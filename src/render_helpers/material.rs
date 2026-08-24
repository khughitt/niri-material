use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use niri_config::{ResolvedGlass, ResolvedMaterial};

use smithay::backend::renderer::element::{Element, Id, Kind, RenderElement, UnderlyingStorage};
use smithay::backend::renderer::gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture, Uniform};
use smithay::backend::renderer::utils::CommitCounter;
use smithay::backend::renderer::Color32F;
use smithay::utils::user_data::UserDataMap;
use smithay::utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Size, Transform};

use super::effect_buffer::EffectBuffer;
use super::offscreen::OffscreenBuffer;
use super::renderer::AsGlesFrame;
use super::shader_element::ShaderRenderElement;
use super::shaders::{ProgramType, Shaders};
use crate::backend::tty::{TtyFrame, TtyRenderer, TtyRendererError};
use crate::render_helpers::RenderTarget;

/// Per-fragment background composition inputs, mirroring `XrayElement`'s
/// two-layer stack for the one workspace the element belongs to.
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundMapping {
    /// Element rect in background-buffer UV, mapped through the workspace
    /// rect (each workspace shows the full buffer).
    pub bg_rect: [f32; 4],
    /// Element rect in backdrop-buffer UV.
    pub backdrop_rect: [f32; 4],
    /// Workspace rect in element UV: inside it the shader samples the
    /// background buffer over the workspace color, outside the backdrop
    /// buffer over the backdrop color. Empty = backdrop everywhere.
    pub ws_rect: [f32; 4],
    /// Premultiplied workspace background color.
    pub ws_color: [f32; 4],
}

/// Selects the workspace containing the element's center and derives the
/// UV mappings. An element straddling a workspace boundary during a
/// workspace switch resolves to its center's workspace; fragments outside
/// that workspace's rect fall back to the backdrop layer.
pub fn background_mapping(
    geo_in_backdrop: Rectangle<f64, Logical>,
    workspaces: &[(Rectangle<f64, Logical>, Color32F)],
    backdrop_size: Size<f64, Logical>,
) -> BackgroundMapping {
    let center = Point::from((
        geo_in_backdrop.loc.x + geo_in_backdrop.size.w / 2.,
        geo_in_backdrop.loc.y + geo_in_backdrop.size.h / 2.,
    ));
    let (bg_rect, ws_rect, ws_color) = match workspaces.iter().find(|(geo, _)| geo.contains(center))
    {
        Some((ws_geo, color)) => (
            [
                ((geo_in_backdrop.loc.x - ws_geo.loc.x) / ws_geo.size.w) as f32,
                ((geo_in_backdrop.loc.y - ws_geo.loc.y) / ws_geo.size.h) as f32,
                (geo_in_backdrop.size.w / ws_geo.size.w) as f32,
                (geo_in_backdrop.size.h / ws_geo.size.h) as f32,
            ],
            uv_rect(*ws_geo, geo_in_backdrop),
            color.components(),
        ),
        None => ([0.; 4], [0.; 4], [0.; 4]),
    };

    BackgroundMapping {
        bg_rect,
        backdrop_rect: [
            (geo_in_backdrop.loc.x / backdrop_size.w) as f32,
            (geo_in_backdrop.loc.y / backdrop_size.h) as f32,
            (geo_in_backdrop.size.w / backdrop_size.w) as f32,
            (geo_in_backdrop.size.h / backdrop_size.h) as f32,
        ],
        ws_rect,
        ws_color,
    }
}

/// Slab constants in logical px, matching the legacy slab mesh so the two
/// implementations stay comparable during the parity pass.
pub const SLAB_DEPTH: f64 = 12.;
pub const SLAB_CORNER_RADIUS: f64 = 28.;

static JELLY_SEED: AtomicUsize = AtomicUsize::new(0);

/// The material element's coordinate frame: the inflated element area plus
/// every rect the shader needs, expressed in element UV.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialFrame {
    /// Element area: union of the window-texture footprint and the slab.
    pub area: Rectangle<f64, Logical>,
    /// Window-texture footprint in element UV (where `niri_tex_win` maps).
    pub geo_rect: [f32; 4],
    /// Slab silhouette in element UV.
    pub slab_rect: [f32; 4],
    /// Element size in logical px.
    pub area_size: [f32; 2],
    /// Chamfer width in logical px: lip + max(|shift-x|, |shift-y|).
    pub chamfer: f32,
}

/// Directional jelly flex derived from animation residuals — a port of the
/// legacy jelly math (`jelly.mjs` in the niri-glass repository). The flex
/// is tanh-saturated toward `max_flex`; `activity` normalizes overall
/// residual magnitude into [0, 1) for the ripple gate.
#[derive(Debug, Clone, PartialEq)]
pub struct JellyState {
    pub move_: [f32; 2],
    pub resize: [f32; 2],
    pub activity: f32,
}

pub fn jelly_state(
    residual: Point<f64, Logical>,
    size_residual: (f64, f64),
    target_size: Size<f64, Logical>,
    flex: f64,
    max_flex: f64,
) -> JellyState {
    // The legacy math works on deltas toward the target.
    let dx = -residual.x;
    let dy = -residual.y;
    let dw = -size_residual.0;
    let dh = -size_residual.1;

    let residual_magnitude = (dx * dx + dy * dy + dw * dw + dh * dh).sqrt();
    let response_distance = (0.5 * target_size.w.hypot(target_size.h)).max(1.);
    let activity = (residual_magnitude / response_distance).tanh() as f32;

    if max_flex <= 0. || flex <= 0. {
        return JellyState {
            move_: [0.; 2],
            resize: [0.; 2],
            activity,
        };
    }

    let raw_x = -flex * dx;
    let raw_y = -flex * dy;
    let raw_mag = raw_x.hypot(raw_y);
    let factor = if raw_mag > 0. {
        max_flex * (raw_mag / max_flex).tanh() / raw_mag
    } else {
        1.
    };

    JellyState {
        move_: [(raw_x * factor) as f32, (raw_y * factor) as f32],
        resize: [
            (max_flex * (-flex * dw / max_flex).tanh()) as f32,
            (max_flex * (-flex * dh / max_flex).tanh()) as f32,
        ],
        activity,
    }
}

/// Jelly values as they reach the shader.
#[derive(Debug, Clone, PartialEq)]
pub struct JellyUniforms {
    pub move_: [f32; 2],
    pub resize: [f32; 2],
    pub activity: f32,
    /// Seconds, wrapped hourly to keep f32 precision; only relative motion
    /// matters to the ripple phase.
    pub time: f32,
}

/// Quantized jelly inputs for damage tracking: 1/1024-px buckets for the
/// shear, 1 ms buckets for the ripple clock while active. At rest the time
/// term is pinned so an idle slab contributes no damage. The clock is
/// wrapped hourly (matching the shader uniform) before quantizing —
/// absolute milliseconds would saturate `i32` after ~24.9 days of uptime
/// and silently freeze jelly damage.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JellyFingerprint {
    move_q: [i32; 2],
    resize_q: [i32; 2],
    activity_q: i32,
    time_q: i32,
}

impl JellyFingerprint {
    pub fn quantize(jelly: &JellyState, time: f64) -> Self {
        let q = |v: f32| (v * 1024.).round() as i32;
        Self {
            move_q: [q(jelly.move_[0]), q(jelly.move_[1])],
            resize_q: [q(jelly.resize[0]), q(jelly.resize[1])],
            activity_q: q(jelly.activity),
            time_q: if jelly.activity > 0. {
                ((time % 3600.) * 1000.).round() as i32
            } else {
                0
            },
        }
    }
}

/// Maps `inner` into `area`-relative UV space.
fn uv_rect(inner: Rectangle<f64, Logical>, area: Rectangle<f64, Logical>) -> [f32; 4] {
    [
        ((inner.loc.x - area.loc.x) / area.size.w) as f32,
        ((inner.loc.y - area.loc.y) / area.size.h) as f32,
        (inner.size.w / area.size.w) as f32,
        (inner.size.h / area.size.h) as f32,
    ]
}

/// Computes the element frame for a window at `win_geo` whose rendered
/// texture covers `tex_geo` (both absolute logical rects).
///
/// The slab is the window rect inflated by `lip` on every side, then slid
/// by (`shift-x`, `shift-y`). The inflation is aligned to physical pixels
/// the way Shadow's is, so the element geometry and damage stay exact.
pub fn material_frame(
    win_geo: Rectangle<f64, Logical>,
    tex_geo: Rectangle<f64, Logical>,
    glass: &ResolvedGlass,
    scale: f64,
) -> MaterialFrame {
    let ceil = |v: f64| (v * scale).ceil() / scale;
    let round = |v: f64| (v * scale).round() / scale;

    let lip = ceil(glass.lip);
    let shift = Point::<f64, Logical>::from((round(glass.shift_x), round(glass.shift_y)));
    let slab = Rectangle::new(
        win_geo.loc - Point::from((lip, lip)) + shift,
        win_geo.size + Size::from((lip * 2., lip * 2.)),
    );
    let area = tex_geo.merge(slab);

    MaterialFrame {
        geo_rect: uv_rect(tex_geo, area),
        slab_rect: uv_rect(slab, area),
        area_size: [area.size.w as f32, area.size.h as f32],
        chamfer: (lip + glass.shift_x.abs().max(glass.shift_y.abs())) as f32,
        area,
    }
}

fn map_normalized_src(
    src: Rectangle<f64, Buffer>,
    texture_src: Rectangle<f64, Buffer>,
) -> Rectangle<f64, Buffer> {
    Rectangle::new(
        (
            texture_src.loc.x + src.loc.x * texture_src.size.w,
            texture_src.loc.y + src.loc.y * texture_src.size.h,
        )
            .into(),
        (
            src.size.w * texture_src.size.w,
            src.size.h * texture_src.size.h,
        )
            .into(),
    )
}

struct PlainWindowSubdraw {
    src: Rectangle<f64, Buffer>,
    dst: Rectangle<i32, Physical>,
    damage: Vec<Rectangle<i32, Physical>>,
    opaque_regions: Vec<Rectangle<i32, Physical>>,
}

fn plain_window_subdraw(
    src: Rectangle<f64, Buffer>,
    dst: Rectangle<i32, Physical>,
    geo_rect: [f32; 4],
    win_src: Rectangle<f64, Buffer>,
    damage: &[Rectangle<i32, Physical>],
    opaque_regions: &[Rectangle<i32, Physical>],
) -> Option<PlainWindowSubdraw> {
    let geo_rect = Rectangle::new(
        Point::from((f64::from(geo_rect[0]), f64::from(geo_rect[1]))),
        Size::from((f64::from(geo_rect[2]), f64::from(geo_rect[3]))),
    );
    let intersection = src.intersection(geo_rect)?;
    if intersection.is_empty() {
        return None;
    }

    let normalized = Rectangle::new(
        Point::from((
            (intersection.loc.x - geo_rect.loc.x) / geo_rect.size.w,
            (intersection.loc.y - geo_rect.loc.y) / geo_rect.size.h,
        )),
        Size::from((
            intersection.size.w / geo_rect.size.w,
            intersection.size.h / geo_rect.size.h,
        )),
    );

    let end = intersection.loc + intersection.size.to_point();
    let map_point = |point: Point<f64, Buffer>| {
        Point::new(
            dst.loc.x
                + (((point.x - src.loc.x) / src.size.w) * f64::from(dst.size.w)).round() as i32,
            dst.loc.y
                + (((point.y - src.loc.y) / src.size.h) * f64::from(dst.size.h)).round() as i32,
        )
    };
    let sub_dst = Rectangle::from_extremities(map_point(intersection.loc), map_point(end));
    if sub_dst.is_empty() {
        return None;
    }

    let mut sub_dst_relative = sub_dst;
    sub_dst_relative.loc -= dst.loc;
    let filter = |regions: &[Rectangle<i32, Physical>]| {
        regions
            .iter()
            .filter_map(|region| {
                region.intersection(sub_dst_relative).map(|mut region| {
                    region.loc -= sub_dst_relative.loc;
                    region
                })
            })
            .collect()
    };

    Some(PlainWindowSubdraw {
        src: map_normalized_src(normalized, win_src),
        dst: sub_dst,
        damage: filter(damage),
        opaque_regions: filter(opaque_regions),
    })
}

/// Everything the material element's pixels depend on that the damage
/// tracker cannot see for itself.
///
/// Element geometry and alpha are deliberately absent: smithay's damage
/// tracker already compares those between frames.
#[derive(Debug, Clone, PartialEq)]
pub struct InputFingerprint {
    /// Commit of the offscreen the window body rendered into.
    pub window: CommitCounter,
    /// Identity of the background buffer being sampled.
    pub background_id: Id,
    /// Commit of the background buffer being sampled.
    pub background: CommitCounter,
    /// Identity of the backdrop buffer being sampled.
    pub backdrop_id: Id,
    /// Commit of the backdrop buffer being sampled.
    pub backdrop: CommitCounter,
    /// Pixel mapping and workspace color used by the shader.
    pub mapping: BackgroundMapping,
    /// Backdrop color composed behind the backdrop buffer.
    pub backdrop_color: [f32; 4],
    pub jelly: JellyFingerprint,
}

#[derive(Debug)]
pub struct MaterialState {
    pub offscreen: OffscreenBuffer,
    id: Id,
    commit: Cell<CommitCounter>,
    material: ResolvedMaterial,
    jelly_seed: [f32; 3],
    last_inputs: RefCell<[Option<InputFingerprint>; RenderTarget::COUNT]>,
}

impl MaterialState {
    pub fn new(material: ResolvedMaterial) -> Self {
        let s = (JELLY_SEED.fetch_add(1, Ordering::Relaxed) % 4096) as f64;
        let jelly_seed = [
            ((s * 0.754_877_66).fract() * 17.) as f32,
            ((s * 0.569_840_29).fract() * 17.) as f32,
            ((s * 0.438_289_97).fract() * 17.) as f32,
        ];
        Self {
            offscreen: OffscreenBuffer::default(),
            id: Id::new(),
            commit: Cell::new(CommitCounter::default()),
            material,
            jelly_seed,
            last_inputs: RefCell::new(std::array::from_fn(|_| None)),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn material(&self) -> &ResolvedMaterial {
        &self.material
    }

    pub fn has_program(renderer: &mut GlesRenderer) -> bool {
        Shaders::get(renderer).material.is_some()
    }

    /// Advances the element commit counter if and only if a pixel input
    /// changed, and returns the counter the next element should carry.
    pub fn advance_commit(&self, target: RenderTarget, inputs: InputFingerprint) -> CommitCounter {
        let last_inputs = &mut self.last_inputs.borrow_mut()[target as usize];
        if last_inputs.as_ref() != Some(&inputs) {
            *last_inputs = Some(inputs);
            self.bump();
        }
        self.commit.get()
    }

    fn bump(&self) {
        let mut commit = self.commit.get();
        commit.increment();
        self.commit.set(commit);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn element(
        &self,
        frame: MaterialFrame,
        mapping: BackgroundMapping,
        jelly: JellyUniforms,
        scale: f64,
        alpha: f32,
        target: RenderTarget,
        inputs: InputFingerprint,
        win_rect: [f32; 4],
        win_src: Rectangle<f64, Buffer>,
        win_texture: GlesTexture,
        bg: Rc<RefCell<EffectBuffer>>,
        backdrop: Rc<RefCell<EffectBuffer>>,
        backdrop_color: [f32; 4],
    ) -> MaterialRenderElement {
        MaterialRenderElement {
            id: self.id.clone(),
            commit: self.advance_commit(target, inputs),
            frame,
            jelly,
            jelly_seed: self.jelly_seed,
            scale,
            alpha,
            glass: self.material.glass,
            win_rect,
            win_src,
            mapping,
            win_texture,
            bg,
            backdrop,
            backdrop_color,
        }
    }
}

/// Applies a newly resolved material to a tile's material slot.
///
/// Returns whether the slot's identity changed — that is, whether the
/// `MaterialState` was created, replaced or dropped, as opposed to updated in
/// place. A parameter-only edit keeps the offscreen buffer and element `Id`
/// and registers as damage; only a change of definition name rebuilds.
pub fn apply_resolved(
    slot: &mut Option<MaterialState>,
    resolved: Option<&ResolvedMaterial>,
) -> bool {
    match (slot.as_mut(), resolved) {
        (None, None) => false,
        (Some(_), None) => {
            *slot = None;
            true
        }
        (Some(state), Some(resolved)) if state.material.name == resolved.name => {
            if state.material.glass != resolved.glass {
                state.material.glass = resolved.glass;
                state.bump();
            }
            false
        }
        (_, Some(resolved)) => {
            *slot = Some(MaterialState::new(resolved.clone()));
            true
        }
    }
}

#[derive(Debug)]
pub struct MaterialRenderElement {
    id: Id,
    commit: CommitCounter,
    /// The element's coordinate frame (area, UV rects, chamfer).
    frame: MaterialFrame,
    jelly: JellyUniforms,
    jelly_seed: [f32; 3],
    scale: f64,
    alpha: f32,
    /// Resolved parameters this element renders with.
    glass: ResolvedGlass,
    /// Window sub-rect of the offscreen texture, normalized UV
    /// (`mat_win_rect`) — the buffer may be larger than the window.
    win_rect: [f32; 4],
    /// The same sub-rect in buffer pixels, for the plain-draw fallback.
    win_src: Rectangle<f64, Buffer>,
    mapping: BackgroundMapping,
    win_texture: GlesTexture,
    bg: Rc<RefCell<EffectBuffer>>,
    backdrop: Rc<RefCell<EffectBuffer>>,
    backdrop_color: [f32; 4],
}

impl Element for MaterialRenderElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_size(Size::from((1., 1.)))
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.frame.area.to_physical_precise_round(scale)
    }

    fn alpha(&self) -> f32 {
        self.alpha
    }

    fn kind(&self) -> Kind {
        Kind::Unspecified
    }
}

impl RenderElement<GlesRenderer> for MaterialRenderElement {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), GlesError> {
        let bg_texture = self.bg.borrow_mut().render(frame, false).ok();
        let backdrop_texture = self.backdrop.borrow_mut().render(frame, false).ok();
        let (Some(bg_texture), Some(backdrop_texture)) = (bg_texture, backdrop_texture) else {
            warn!("material: error rendering background/backdrop buffer");
            let Some(subdraw) = plain_window_subdraw(
                src,
                dst,
                self.frame.geo_rect,
                self.win_src,
                damage,
                opaque_regions,
            ) else {
                return Ok(());
            };
            return frame.render_texture_from_to(
                &self.win_texture,
                subdraw.src,
                subdraw.dst,
                &subdraw.damage,
                &subdraw.opaque_regions,
                Transform::Normal,
                self.alpha,
                None,
                &[],
            );
        };

        let g = &self.glass;
        let f = &self.frame;
        let uniforms: Rc<[Uniform<'static>]> = Rc::new([
            Uniform::new("mat_win_rect", self.win_rect),
            Uniform::new("mat_geo_rect", f.geo_rect),
            Uniform::new("mat_slab_rect", f.slab_rect),
            Uniform::new("mat_area_size", f.area_size),
            Uniform::new("mat_chamfer", f.chamfer),
            Uniform::new("mat_jelly_move", self.jelly.move_),
            Uniform::new("mat_jelly_resize", self.jelly.resize),
            Uniform::new("mat_jelly_activity", self.jelly.activity),
            Uniform::new("mat_jelly_time", self.jelly.time),
            Uniform::new("mat_jelly_seed", self.jelly_seed),
            Uniform::new("mat_bg_rect", self.mapping.bg_rect),
            Uniform::new("mat_backdrop_rect", self.mapping.backdrop_rect),
            Uniform::new("mat_ws_rect", self.mapping.ws_rect),
            Uniform::new("mat_ws_color", self.mapping.ws_color),
            Uniform::new("mat_backdrop_color", self.backdrop_color),
            Uniform::new("mat_ior", g.ior as f32),
            Uniform::new("mat_thickness", g.thickness as f32),
            Uniform::new(
                "mat_attenuation_color",
                g.attenuation_color.to_array_unpremul(),
            ),
            Uniform::new("mat_attenuation_distance", g.attenuation_distance as f32),
            Uniform::new("mat_chromatic_aberration", g.chromatic_aberration as f32),
            Uniform::new("mat_distortion", g.distortion as f32),
            Uniform::new("mat_distortion_scale", g.distortion_scale as f32),
            Uniform::new("mat_samples", f32::from(g.samples)),
            Uniform::new("mat_anisotropic_blur", g.anisotropic_blur as f32),
            Uniform::new("mat_jelly_ripple", g.jelly_ripple as f32),
        ]);
        let textures = HashMap::from([
            (String::from("niri_tex_win"), self.win_texture.clone()),
            (String::from("niri_tex_bg"), bg_texture),
            (String::from("niri_tex_backdrop"), backdrop_texture),
        ]);
        let inner = ShaderRenderElement::new(
            ProgramType::Material,
            self.frame.area.size,
            None,
            self.scale as f32,
            self.alpha,
            uniforms,
            textures,
            Kind::Unspecified,
        );
        RenderElement::<GlesRenderer>::draw(&inner, frame, src, dst, damage, opaque_regions, cache)
    }

    fn underlying_storage(&self, _renderer: &mut GlesRenderer) -> Option<UnderlyingStorage<'_>> {
        None
    }
}

impl<'render> RenderElement<TtyRenderer<'render>> for MaterialRenderElement {
    fn draw(
        &self,
        frame: &mut TtyFrame<'_, '_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), TtyRendererError<'render>> {
        let frame = frame.as_gles_frame();
        RenderElement::<GlesRenderer>::draw(self, frame, src, dst, damage, opaque_regions, cache)?;
        Ok(())
    }

    fn underlying_storage(
        &self,
        _renderer: &mut TtyRenderer<'render>,
    ) -> Option<UnderlyingStorage<'_>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niri_config::{ResolvedGlass, ResolvedMaterial};
    use smithay::backend::renderer::Color32F;
    use smithay::utils::Point;

    use crate::render_helpers::RenderTarget;

    fn material(name: &str) -> ResolvedMaterial {
        ResolvedMaterial {
            name: String::from(name),
            glass: ResolvedGlass::default(),
        }
    }

    fn commit_after(n: usize) -> CommitCounter {
        let mut c = CommitCounter::default();
        for _ in 0..n {
            c.increment();
        }
        c
    }

    fn fingerprint(
        window: usize,
        background: usize,
        background_id: &Id,
        backdrop_id: &Id,
    ) -> InputFingerprint {
        fingerprint2(window, background, background_id, 1, backdrop_id)
    }

    fn fingerprint2(
        window: usize,
        background: usize,
        background_id: &Id,
        backdrop: usize,
        backdrop_id: &Id,
    ) -> InputFingerprint {
        InputFingerprint {
            window: commit_after(window),
            background: commit_after(background),
            background_id: background_id.clone(),
            backdrop: commit_after(backdrop),
            backdrop_id: backdrop_id.clone(),
            mapping: BackgroundMapping {
                bg_rect: [0.; 4],
                backdrop_rect: [0.; 4],
                ws_rect: [0.; 4],
                ws_color: [0.; 4],
            },
            backdrop_color: [0.; 4],
            jelly: JellyFingerprint::default(),
        }
    }

    #[test]
    fn jelly_at_rest_is_neutral() {
        let j = jelly_state(
            Point::from((0., 0.)),
            (0., 0.),
            Size::new(800., 600.),
            0.004,
            3.,
        );
        assert_eq!(j.move_, [0., 0.]);
        assert_eq!(j.resize, [0., 0.]);
        assert_eq!(j.activity, 0.);
    }

    #[test]
    fn jelly_move_trails_the_residual() {
        // The tile sits 100 px left of its target (residual −100): the
        // front face must trail right→left, i.e. lag toward the old
        // position: negative x.
        let j = jelly_state(
            Point::from((-100., 0.)),
            (0., 0.),
            Size::new(800., 600.),
            0.004,
            3.,
        );
        assert!(j.move_[0] < 0.);
        assert_eq!(j.move_[1], 0.);
        assert!(j.activity > 0.);
    }

    #[test]
    fn jelly_move_saturates_at_max_flex() {
        let j = jelly_state(
            Point::from((100_000., 0.)),
            (0., 0.),
            Size::new(800., 600.),
            0.004,
            3.,
        );
        assert!(j.move_[0] <= 3.0 + 1e-4);
        assert!(j.move_[0] > 2.9);
    }

    #[test]
    fn jelly_zero_max_flex_still_reports_activity() {
        let j = jelly_state(
            Point::from((50., 0.)),
            (0., 0.),
            Size::new(800., 600.),
            0.004,
            0.,
        );
        assert_eq!(j.move_, [0., 0.]);
        assert!(j.activity > 0.);
    }

    #[test]
    fn jelly_fingerprint_ignores_time_at_rest() {
        let a = JellyFingerprint::quantize(
            &JellyState {
                move_: [0.; 2],
                resize: [0.; 2],
                activity: 0.,
            },
            1.0,
        );
        let b = JellyFingerprint::quantize(
            &JellyState {
                move_: [0.; 2],
                resize: [0.; 2],
                activity: 0.,
            },
            2.0,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn jelly_fingerprint_tracks_time_while_active() {
        let j = JellyState {
            move_: [1., 0.],
            resize: [0.; 2],
            activity: 0.5,
        };
        let a = JellyFingerprint::quantize(&j, 1.0);
        let b = JellyFingerprint::quantize(&j, 1.05);
        assert_ne!(a, b);
    }

    #[test]
    fn unchanged_inputs_do_not_advance_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let first = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &background_id, &backdrop_id),
        );
        let second = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &background_id, &backdrop_id),
        );
        let third = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &background_id, &backdrop_id),
        );

        // A glass window at rest must contribute no damage.
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn window_damage_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let first = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &background_id, &backdrop_id),
        );
        let second = state.advance_commit(
            RenderTarget::Output,
            fingerprint(2, 1, &background_id, &backdrop_id),
        );

        assert_ne!(first, second);
    }

    #[test]
    fn background_damage_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let first = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &background_id, &backdrop_id),
        );
        let second = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 2, &background_id, &backdrop_id),
        );

        assert_ne!(first, second);
    }

    #[test]
    fn stable_targets_converge_on_one_commit() {
        let state = MaterialState::new(material("frost"));
        let output_bg = Id::new();
        let screencast_bg = Id::new();
        let backdrop_id = Id::new();

        state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &output_bg, &backdrop_id),
        );
        state.advance_commit(
            RenderTarget::Screencast,
            fingerprint(1, 1, &screencast_bg, &backdrop_id),
        );
        let output = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &output_bg, &backdrop_id),
        );
        let screencast = state.advance_commit(
            RenderTarget::Screencast,
            fingerprint(1, 1, &screencast_bg, &backdrop_id),
        );

        assert_eq!(output, screencast);
    }

    #[test]
    fn replacing_a_targets_background_buffer_is_damage() {
        let state = MaterialState::new(material("frost"));
        let first_bg = Id::new();
        let replacement_bg = Id::new();
        let backdrop_id = Id::new();

        let before = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &first_bg, &backdrop_id),
        );
        let after = state.advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &replacement_bg, &backdrop_id),
        );

        assert_ne!(before, after);
    }

    #[test]
    fn parameter_change_advances_the_commit_in_place() {
        let mut slot = Some(MaterialState::new(material("frost")));
        let id_before = slot.as_ref().unwrap().id().clone();
        let output_bg = Id::new();
        let screencast_bg = Id::new();
        let backdrop_id = Id::new();
        slot.as_ref().unwrap().advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &output_bg, &backdrop_id),
        );
        slot.as_ref().unwrap().advance_commit(
            RenderTarget::Screencast,
            fingerprint(1, 1, &screencast_bg, &backdrop_id),
        );
        let commit_before = slot.as_ref().unwrap().advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &output_bg, &backdrop_id),
        );

        let mut changed = material("frost");
        changed.glass.ior = 1.6;
        let rebuilt = apply_resolved(&mut slot, Some(&changed));

        let state = slot.as_ref().unwrap();
        assert!(!rebuilt);
        assert_eq!(state.id(), &id_before);
        assert_eq!(state.material().glass.ior, 1.6);

        let mut expected = commit_before;
        expected.increment();
        assert_eq!(
            state.advance_commit(
                RenderTarget::Output,
                fingerprint(1, 1, &output_bg, &backdrop_id)
            ),
            expected
        );
        assert_eq!(
            state.advance_commit(
                RenderTarget::Screencast,
                fingerprint(1, 1, &screencast_bg, &backdrop_id)
            ),
            expected
        );
    }

    #[test]
    fn identical_parameters_are_not_damage() {
        let mut slot = Some(MaterialState::new(material("frost")));
        let background_id = Id::new();
        let backdrop_id = Id::new();
        let commit_before = slot.as_ref().unwrap().advance_commit(
            RenderTarget::Output,
            fingerprint(1, 1, &background_id, &backdrop_id),
        );

        let rebuilt = apply_resolved(&mut slot, Some(&material("frost")));

        assert!(!rebuilt);
        assert_eq!(
            slot.as_ref().unwrap().advance_commit(
                RenderTarget::Output,
                fingerprint(1, 1, &background_id, &backdrop_id)
            ),
            commit_before
        );
    }

    #[test]
    fn name_change_rebuilds_the_state() {
        let mut slot = Some(MaterialState::new(material("frost")));
        let id_before = slot.as_ref().unwrap().id().clone();

        let rebuilt = apply_resolved(&mut slot, Some(&material("clear")));

        let state = slot.as_ref().unwrap();
        assert!(rebuilt);
        assert_ne!(state.id(), &id_before);
        assert_eq!(state.material().name, "clear");
    }

    #[test]
    fn losing_the_material_clears_the_state() {
        let mut slot = Some(MaterialState::new(material("frost")));

        let rebuilt = apply_resolved(&mut slot, None);

        assert!(rebuilt);
        assert!(slot.is_none());
    }

    #[test]
    fn no_material_keeps_empty_slot() {
        let mut slot = None;

        let rebuilt = apply_resolved(&mut slot, None);

        assert!(!rebuilt);
        assert!(slot.is_none());
    }

    #[test]
    fn gaining_a_material_builds_the_state() {
        let mut slot = None;

        let rebuilt = apply_resolved(&mut slot, Some(&material("frost")));

        assert!(rebuilt);
        assert_eq!(slot.as_ref().unwrap().material().name, "frost");
    }

    #[test]
    fn background_mapping_at_rest_is_identity_like() {
        let ws = Rectangle::new(Point::new(0., 0.), Size::new(1920., 1080.));
        let color = Color32F::new(0.1, 0.2, 0.3, 1.);
        let geo = Rectangle::new(Point::new(192., 108.), Size::new(384., 216.));

        let m = background_mapping(geo, &[(ws, color)], Size::new(1920., 1080.));

        assert_eq!(m.bg_rect, [0.1, 0.1, 0.2, 0.2]);
        assert_eq!(m.backdrop_rect, [0.1, 0.1, 0.2, 0.2]);
        assert_eq!(m.ws_rect, [-0.5, -0.5, 5., 5.]);
        assert_eq!(m.ws_color, color.components());
    }

    #[test]
    fn background_mapping_overview_scales_through_the_workspace() {
        let ws = Rectangle::new(Point::new(480., 270.), Size::new(960., 540.));
        let color = Color32F::new(0., 0., 0., 1.);
        let geo = Rectangle::new(Point::new(720., 405.), Size::new(96., 54.));

        let m = background_mapping(geo, &[(ws, color)], Size::new(1920., 1080.));

        assert_eq!(m.bg_rect, [0.25, 0.25, 0.1, 0.1]);
    }

    #[test]
    fn background_mapping_without_a_containing_workspace_is_backdrop_only() {
        let geo = Rectangle::new(Point::new(0., 0.), Size::new(100., 100.));

        let m = background_mapping(geo, &[], Size::new(1920., 1080.));

        assert_eq!(m.ws_rect, [0.; 4]);
        assert_eq!(m.ws_color, [0.; 4]);
    }

    #[test]
    fn backdrop_commit_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let a = state.advance_commit(
            RenderTarget::Output,
            fingerprint2(1, 1, &background_id, 1, &backdrop_id),
        );
        let b = state.advance_commit(
            RenderTarget::Output,
            fingerprint2(1, 1, &background_id, 2, &backdrop_id),
        );

        assert_ne!(a, b);
    }

    #[test]
    fn workspace_color_change_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let mut f = fingerprint2(1, 1, &background_id, 1, &backdrop_id);
        let a = state.advance_commit(RenderTarget::Output, f.clone());
        f.mapping.ws_color = [1., 0., 0., 1.];
        let b = state.advance_commit(RenderTarget::Output, f);

        assert_ne!(a, b);
    }

    #[test]
    fn background_mapping_change_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();
        let backdrop_id = Id::new();

        let mut f = fingerprint2(1, 1, &background_id, 1, &backdrop_id);
        let a = state.advance_commit(RenderTarget::Output, f.clone());
        f.mapping.bg_rect[0] = 0.25;
        let b = state.advance_commit(RenderTarget::Output, f);

        assert_ne!(a, b);
    }

    #[test]
    fn material_frame_inflates_by_lip_and_shift() {
        let mut glass = ResolvedGlass::default();
        glass.lip = 10.;
        glass.shift_x = 0.;
        glass.shift_y = 0.;
        let win = Rectangle::new(Point::new(0., 0.), Size::new(200., 100.));

        let frame = material_frame(win, win, &glass, 1.);

        // Slab = window + 10 on every side; area = union = the slab.
        assert_eq!(
            frame.area,
            Rectangle::new(Point::new(-10., -10.), Size::new(220., 120.))
        );
        assert_eq!(frame.slab_rect, [0., 0., 1., 1.]);
        assert_eq!(
            frame.geo_rect,
            [10. / 220., 10. / 120., 200. / 220., 100. / 120.]
        );
        assert_eq!(frame.area_size, [220., 120.]);
        assert_eq!(frame.chamfer, 10.);
    }

    #[test]
    fn material_frame_shift_slides_the_slab() {
        let mut glass = ResolvedGlass::default();
        glass.lip = 6.;
        glass.shift_x = 6.;
        glass.shift_y = 6.;
        let win = Rectangle::new(Point::new(100., 50.), Size::new(200., 100.));

        let frame = material_frame(win, win, &glass, 1.);

        // lip 6 + shift 6: the slab's top-left lands on the window's
        // top-left; the lip is fully on the right/bottom sides.
        assert_eq!(
            frame.area,
            Rectangle::new(Point::new(100., 50.), Size::new(212., 112.))
        );
        // chamfer = lip + max(|sx|, |sy|) = 12
        assert_eq!(frame.chamfer, 12.);
    }

    #[test]
    fn material_frame_aligns_to_physical_pixels() {
        let mut glass = ResolvedGlass::default();
        glass.lip = 5.3;
        glass.shift_x = 0.;
        glass.shift_y = 0.;
        let win = Rectangle::new(Point::new(0., 0.), Size::new(100., 100.));

        let frame = material_frame(win, win, &glass, 2.);

        // ceil(5.3 * 2) / 2 = 5.5
        assert_eq!(frame.area.loc, Point::new(-5.5, -5.5));
    }

    #[test]
    fn material_frame_merges_an_oversized_texture_footprint() {
        let mut glass = ResolvedGlass::default();
        glass.lip = 6.;
        glass.shift_x = 0.;
        glass.shift_y = 0.;
        let win = Rectangle::new(Point::new(0., 0.), Size::new(200., 100.));
        // CSD shadows: the texture extends 20 px past the window.
        let tex = Rectangle::new(Point::new(-20., -20.), Size::new(240., 140.));

        let frame = material_frame(win, tex, &glass, 1.);

        assert_eq!(frame.area, tex);
        // The slab is smaller than the area here.
        assert_eq!(
            frame.slab_rect,
            [14. / 240., 14. / 140., 212. / 240., 112. / 140.]
        );
    }

    #[test]
    fn normalized_src_maps_into_texture_subrectangle() {
        let src = Rectangle::new(Point::new(0.25, 0.25), Size::new(0.5, 0.4));
        let texture_src = Rectangle::new(Point::new(10., 20.), Size::new(200., 100.));

        assert_eq!(
            map_normalized_src(src, texture_src),
            Rectangle::new(Point::new(60., 45.), Size::new(100., 40.))
        );
    }

    #[test]
    fn cropped_plain_window_fallback_maps_source_destination_and_regions() {
        let src = Rectangle::new(Point::new(0.5, 0.125), Size::new(0.375, 0.75));
        let dst = Rectangle::new(Point::new(100, 200), Size::new(400, 600));
        let win_src = Rectangle::new(Point::new(10., 20.), Size::new(800., 600.));
        let damage = [Rectangle::new(Point::new(0, 0), Size::new(400, 200))];
        let opaque = [Rectangle::new(Point::new(0, 0), Size::new(400, 600))];

        let subdraw = plain_window_subdraw(
            src,
            dst,
            [0.125, 0.25, 0.75, 0.5],
            win_src,
            &damage,
            &opaque,
        )
        .unwrap();

        assert_eq!(
            subdraw.src,
            Rectangle::new(Point::new(410., 20.), Size::new(400., 600.))
        );
        assert_eq!(
            subdraw.dst,
            Rectangle::new(Point::new(100, 300), Size::new(400, 400))
        );
        assert_eq!(
            subdraw.damage,
            [Rectangle::new(Point::new(0, 0), Size::new(400, 100))]
        );
        assert_eq!(
            subdraw.opaque_regions,
            [Rectangle::new(Point::new(0, 0), Size::new(400, 400))]
        );
    }

    #[test]
    fn cropped_plain_window_fallback_skips_when_window_is_outside_crop() {
        let src = Rectangle::new(Point::new(0., 0.), Size::new(0.25, 1.));
        let dst = Rectangle::new(Point::new(0, 0), Size::new(250, 1000));
        let win_src = Rectangle::new(Point::new(0., 0.), Size::new(500., 500.));

        assert!(plain_window_subdraw(src, dst, [0.5, 0., 0.5, 1.], win_src, &[], &[],).is_none());
    }
}
