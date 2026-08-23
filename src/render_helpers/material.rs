use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use niri_config::{ResolvedGlass, ResolvedMaterial};

use smithay::backend::renderer::element::{Element, Id, Kind, RenderElement, UnderlyingStorage};
use smithay::backend::renderer::gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture, Uniform};
use smithay::backend::renderer::utils::CommitCounter;
use smithay::utils::user_data::UserDataMap;
use smithay::utils::{Buffer, Logical, Physical, Rectangle, Scale, Size, Transform};

use super::effect_buffer::EffectBuffer;
use super::offscreen::OffscreenBuffer;
use super::renderer::AsGlesFrame;
use super::shader_element::ShaderRenderElement;
use super::shaders::{ProgramType, Shaders};
use crate::backend::tty::{TtyFrame, TtyRenderer, TtyRendererError};
use crate::render_helpers::RenderTarget;

/// Maps an element rectangle (already in backdrop/background coordinates)
/// to a UV-space rect `[x, y, w, h]` of the background texture.
pub fn bg_uv_rect(elem_geo: Rectangle<f64, Logical>, bg_size: Size<f64, Logical>) -> [f32; 4] {
    [
        (elem_geo.loc.x / bg_size.w) as f32,
        (elem_geo.loc.y / bg_size.h) as f32,
        (elem_geo.size.w / bg_size.w) as f32,
        (elem_geo.size.h / bg_size.h) as f32,
    ]
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
}

#[derive(Debug)]
pub struct MaterialState {
    pub offscreen: OffscreenBuffer,
    id: Id,
    commit: Cell<CommitCounter>,
    material: ResolvedMaterial,
    last_inputs: RefCell<[Option<InputFingerprint>; RenderTarget::COUNT]>,
}

impl MaterialState {
    pub fn new(material: ResolvedMaterial) -> Self {
        Self {
            offscreen: OffscreenBuffer::default(),
            id: Id::new(),
            commit: Cell::new(CommitCounter::default()),
            material,
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
        area: Rectangle<f64, Logical>,
        scale: f64,
        alpha: f32,
        target: RenderTarget,
        inputs: InputFingerprint,
        win_rect: [f32; 4],
        win_src: Rectangle<f64, Buffer>,
        bg_rect: [f32; 4],
        win_texture: GlesTexture,
        bg: Rc<RefCell<EffectBuffer>>,
    ) -> MaterialRenderElement {
        MaterialRenderElement {
            id: self.id.clone(),
            commit: self.advance_commit(target, inputs),
            area,
            scale,
            alpha,
            glass: self.material.glass,
            win_rect,
            win_src,
            bg_rect,
            win_texture,
            bg,
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
    area: Rectangle<f64, Logical>,
    scale: f64,
    alpha: f32,
    /// Resolved parameters this element renders with.
    glass: ResolvedGlass,
    /// Window sub-rect of the offscreen texture, normalized UV
    /// (`mat_win_rect`) — the buffer may be larger than the window.
    win_rect: [f32; 4],
    /// The same sub-rect in buffer pixels, for the plain-draw fallback.
    win_src: Rectangle<f64, Buffer>,
    bg_rect: [f32; 4],
    win_texture: GlesTexture,
    bg: Rc<RefCell<EffectBuffer>>,
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
        self.area.to_physical_precise_round(scale)
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
        let bg_texture = match self.bg.borrow_mut().render(frame, false) {
            Ok(tex) => Some(tex),
            Err(err) => {
                warn!("material: error rendering background buffer: {err:?}");
                None
            }
        };

        let Some(bg_texture) = bg_texture else {
            return frame.render_texture_from_to(
                &self.win_texture,
                map_normalized_src(src, self.win_src),
                dst,
                damage,
                opaque_regions,
                Transform::Normal,
                self.alpha,
                None,
                &[],
            );
        };

        let g = &self.glass;
        let uniforms: Rc<[Uniform<'static>]> = Rc::new([
            Uniform::new("mat_win_rect", self.win_rect),
            Uniform::new("mat_bg_rect", self.bg_rect),
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
            Uniform::new("mat_jelly_flex", g.jelly_flex as f32),
            Uniform::new("mat_jelly_ripple", g.jelly_ripple as f32),
            Uniform::new("mat_lip", g.lip as f32),
            Uniform::new("mat_shift", [g.shift_x as f32, g.shift_y as f32]),
        ]);
        let textures = HashMap::from([
            (String::from("niri_tex_win"), self.win_texture.clone()),
            (String::from("niri_tex_bg"), bg_texture),
        ]);
        let inner = ShaderRenderElement::new(
            ProgramType::Material,
            self.area.size,
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

    fn fingerprint(window: usize, background: usize, background_id: &Id) -> InputFingerprint {
        InputFingerprint {
            window: commit_after(window),
            background: commit_after(background),
            background_id: background_id.clone(),
        }
    }

    #[test]
    fn unchanged_inputs_do_not_advance_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();

        let first = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id));
        let second = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id));
        let third = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id));

        // A glass window at rest must contribute no damage.
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn window_damage_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();

        let first = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id));
        let second = state.advance_commit(RenderTarget::Output, fingerprint(2, 1, &background_id));

        assert_ne!(first, second);
    }

    #[test]
    fn background_damage_advances_the_commit() {
        let state = MaterialState::new(material("frost"));
        let background_id = Id::new();

        let first = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id));
        let second = state.advance_commit(RenderTarget::Output, fingerprint(1, 2, &background_id));

        assert_ne!(first, second);
    }

    #[test]
    fn stable_targets_converge_on_one_commit() {
        let state = MaterialState::new(material("frost"));
        let output_bg = Id::new();
        let screencast_bg = Id::new();

        state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &output_bg));
        state.advance_commit(RenderTarget::Screencast, fingerprint(1, 1, &screencast_bg));
        let output = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &output_bg));
        let screencast =
            state.advance_commit(RenderTarget::Screencast, fingerprint(1, 1, &screencast_bg));

        assert_eq!(output, screencast);
    }

    #[test]
    fn replacing_a_targets_background_buffer_is_damage() {
        let state = MaterialState::new(material("frost"));
        let first_bg = Id::new();
        let replacement_bg = Id::new();

        let before = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &first_bg));
        let after = state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &replacement_bg));

        assert_ne!(before, after);
    }

    #[test]
    fn parameter_change_advances_the_commit_in_place() {
        let mut slot = Some(MaterialState::new(material("frost")));
        let id_before = slot.as_ref().unwrap().id().clone();
        let output_bg = Id::new();
        let screencast_bg = Id::new();
        slot.as_ref()
            .unwrap()
            .advance_commit(RenderTarget::Output, fingerprint(1, 1, &output_bg));
        slot.as_ref()
            .unwrap()
            .advance_commit(RenderTarget::Screencast, fingerprint(1, 1, &screencast_bg));
        let commit_before = slot
            .as_ref()
            .unwrap()
            .advance_commit(RenderTarget::Output, fingerprint(1, 1, &output_bg));

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
            state.advance_commit(RenderTarget::Output, fingerprint(1, 1, &output_bg)),
            expected
        );
        assert_eq!(
            state.advance_commit(RenderTarget::Screencast, fingerprint(1, 1, &screencast_bg)),
            expected
        );
    }

    #[test]
    fn identical_parameters_are_not_damage() {
        let mut slot = Some(MaterialState::new(material("frost")));
        let background_id = Id::new();
        let commit_before = slot
            .as_ref()
            .unwrap()
            .advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id));

        let rebuilt = apply_resolved(&mut slot, Some(&material("frost")));

        assert!(!rebuilt);
        assert_eq!(
            slot.as_ref()
                .unwrap()
                .advance_commit(RenderTarget::Output, fingerprint(1, 1, &background_id)),
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
    fn bg_uv_rect_identity() {
        // Element covering the whole background maps to the full UV square.
        let geo = Rectangle::new(Point::new(0., 0.), Size::new(1000., 500.));
        let bg = Size::new(1000., 500.);
        assert_eq!(bg_uv_rect(geo, bg), [0., 0., 1., 1.]);
    }

    #[test]
    fn bg_uv_rect_offset_quarter() {
        // A 250x125 element at (500, 250) on a 1000x500 background samples
        // the lower-right quadrant's first quarter.
        let geo = Rectangle::new(Point::new(500., 250.), Size::new(250., 125.));
        let bg = Size::new(1000., 500.);
        assert_eq!(bg_uv_rect(geo, bg), [0.5, 0.5, 0.25, 0.25]);
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
}
