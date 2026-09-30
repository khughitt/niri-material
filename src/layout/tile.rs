use core::f64;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use niri_config::utils::MergeWith as _;
use niri_config::{
    Color, CornerRadius, GradientInterpolation, MaterialRef, ResolvedGlass, ResolvedResponse,
};
use niri_ipc::WindowLayout;
use smithay::backend::renderer::element::{Element, Kind};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexProgram};
use smithay::backend::renderer::Texture as _;
use smithay::utils::{Logical, Point, Rectangle, Scale, Size};

use super::focus_ring::{FocusRing, FocusRingRenderElement};
use super::opening_window::{OpenAnimation, OpeningWindowRenderElement};
use super::shadow::Shadow;
use super::{
    HitType, LayoutElement, LayoutElementRenderElement, LayoutElementRenderSnapshot, Options,
    SizeFrac, RESIZE_ANIMATION_THRESHOLD,
};
use crate::animation::{Animation, Clock};
use crate::layout::SizingMode;
use crate::niri_render_elements;
use crate::render_helpers::background_effect::BackgroundEffectElement;
use crate::render_helpers::border::BorderRenderElement;
use crate::render_helpers::clipped_surface::{ClippedSurfaceRenderElement, RoundedCornerDamage};
use crate::render_helpers::damage::ExtraDamage;
use crate::render_helpers::material::optics::{self, OpticFrame};
use crate::render_helpers::material::ring::{self, BeamFrame};
use crate::render_helpers::material::{
    apply_resolved, background_mapping, bevel_depth, glass_signal_inputs, jelly_state,
    material_frame, GlassSignalFingerprint, GlassSignalInputs, InputFingerprint, JellyFingerprint,
    JellyUniforms, MaterialFrame, MaterialRenderConfig, MaterialRenderElement, MaterialState,
    SignalUniforms,
};
use crate::render_helpers::offscreen::{OffscreenBuffer, OffscreenRenderElement};
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::resize::ResizeRenderElement;
use crate::render_helpers::shadow::ShadowRenderElement;
use crate::render_helpers::signal::{solve, EffectiveSignal, FrameInputs, SignalFingerprint};
use crate::render_helpers::snapshot::RenderSnapshot;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::xray::{Xray, XrayPos};
use crate::render_helpers::{RenderCtx, RenderTarget};
use crate::utils::transaction::Transaction;
use crate::utils::{
    baba_is_float_offset, round_logical_in_physical, round_logical_in_physical_max1,
};

/// Toplevel window with decorations.
#[derive(Debug)]
pub struct Tile<W: LayoutElement> {
    /// The toplevel window itself.
    window: W,

    /// The border around the window.
    border: FocusRing,

    /// The focus ring around the window.
    focus_ring: FocusRing,

    /// The shadow around the window.
    shadow: Shadow,

    /// This tile's current sizing mode.
    ///
    /// This will update only when the `window` actually goes maximized or fullscreen, rather than
    /// right away, to avoid black backdrop flicker before the window has had a chance to resize.
    sizing_mode: SizingMode,

    /// The black backdrop for fullscreen windows.
    fullscreen_backdrop: SolidColorBuffer,

    /// Whether the tile should float upon unfullscreening.
    pub(super) restore_to_floating: bool,

    /// The size that the window should assume when going floating.
    ///
    /// This is generally the last size the window had when it was floating. It can be unknown if
    /// the window starts out in the tiling layout or fullscreen.
    pub(super) floating_window_size: Option<Size<i32, Logical>>,

    /// The position that the tile should assume when going floating, relative to the floating
    /// space working area.
    ///
    /// This is generally the last position the tile had when it was floating. It can be unknown if
    /// the window starts out in the tiling layout.
    pub(super) floating_pos: Option<Point<f64, SizeFrac>>,

    /// Currently selected preset width index when this tile is floating.
    pub(super) floating_preset_width_idx: Option<usize>,

    /// Currently selected preset height index when this tile is floating.
    pub(super) floating_preset_height_idx: Option<usize>,

    /// The animation upon opening a window.
    open_animation: Option<OpenAnimation>,

    /// The animation of the window resizing.
    resize_animation: Option<ResizeAnimation>,

    /// The animation of a tile visually moving horizontally.
    move_x_animation: Option<MoveAnimation>,

    /// The animation of a tile visually moving vertically.
    move_y_animation: Option<MoveAnimation>,

    /// The animation of the tile's opacity.
    pub(super) alpha_animation: Option<AlphaAnimation>,

    /// Offset during the initial interactive move rubberband.
    pub(super) interactive_move_offset: Point<f64, Logical>,

    /// Snapshot of the last render for use in the close animation.
    unmap_snapshot: Option<TileRenderSnapshot>,

    /// Extra damage for clipped surface corner radius changes.
    rounded_corner_damage: RoundedCornerDamage,

    /// The view size for the tile's workspace.
    ///
    /// Used as the fullscreen target size.
    view_size: Size<f64, Logical>,

    /// Scale of the output the tile is on (and rounds its sizes to).
    scale: f64,

    /// Clock for driving animations.
    pub(super) clock: Clock,

    /// State for slice-0 material rendering.
    material: Option<MaterialState>,

    signal_crossfade: Option<SignalCrossfade>,
    signal_target: Option<(f32, Option<[f32; 3]>)>,
    signal_frame_cache: RefCell<Option<(EffectiveSignal, FrameInputs)>>,
    signal_render_visible: bool,
    /// Whether this tile was active at the last `update_render_elements`.
    active: bool,
    /// The input-activity gate at the last `update_render_elements` (design 2026-09-18 §3).
    input_active: bool,
    focus_crossfade: Option<FocusCrossfade>,
    focus_beam: Option<FocusBeam>,

    /// Configurable properties of the layout.
    pub(super) options: Rc<Options>,
}

niri_render_elements! {
    TileRenderElement<R> => {
        LayoutElement = LayoutElementRenderElement<R>,
        FocusRing = FocusRingRenderElement,
        SolidColor = SolidColorRenderElement,
        Opening = OpeningWindowRenderElement,
        Resize = ResizeRenderElement,
        Material = MaterialRenderElement,
        Border = BorderRenderElement,
        Shadow = ShadowRenderElement,
        ClippedSurface = ClippedSurfaceRenderElement<R>,
        Offscreen = OffscreenRenderElement,
        ExtraDamage = ExtraDamage,
        BackgroundEffect = BackgroundEffectElement,
    }
}

pub type TileRenderSnapshot =
    RenderSnapshot<TileRenderElement<GlesRenderer>, TileRenderElement<GlesRenderer>>;

/// Whether a material's backdrop blur is actually on, given the global switch.
///
/// `BlurOptions` carries only `passes` and `offset` and drops `off`, so any
/// consumer that re-derived this would silently ignore `blur { off }` — the
/// same reason the background effect gates separately at
/// `src/render_helpers/background_effect.rs:172`.
fn backdrop_blur_enabled(glass: &ResolvedGlass, blur: &niri_config::Blur) -> bool {
    glass.backdrop_blur && !blur.off
}

/// The material a window resolves to under the current config, if any.
///
/// A name the current config does not define is not an error. `reload_config`
/// updates the layout before it recomputes window rules, so a tile is
/// re-resolved against the new config while its window still carries the name
/// resolved against the old one — the case a rename or a removal produces.
/// Config parsing rejects genuinely dangling references, so the only reachable
/// miss is that transient one; dropping the material lets `apply_resolved`
/// clear the state, and the rule recompute that follows re-resolves the tile
/// through `Tile::update_window`.
///
/// The global `blur { off }` switch is applied here, so every consumer
/// downstream reads one already-gated value rather than re-deriving it.
/// The noise and saturation inherit rule lives in their optics, which read
/// the gated value through `OpticFrame`.
fn resolve_material(
    reference: Option<&MaterialRef>,
    options: &Options,
) -> Option<MaterialRenderConfig> {
    let reference = reference?;
    let mut material = options.materials.get(&reference.name).cloned()?;
    let backdrop_blur = backdrop_blur_enabled(&material.glass, &options.blur);
    material.glass.backdrop_blur = backdrop_blur;
    let selected = material.response(reference.response.as_deref());
    material.responses = vec![(String::from("default"), selected)];
    Some(MaterialRenderConfig { material })
}

/// Routes one window render element through the clip-to-geometry path:
/// clipped Wayland surface, radius-rounded blocked-out solid color, or
/// passthrough — the same behavior with and without a material.
fn clip_window_element<R: NiriRenderer>(
    elem: LayoutElementRenderElement<R>,
    scale: Scale<f64>,
    geo: Rectangle<f64, Logical>,
    radius: CornerRadius,
    clip_to_geometry: bool,
    clip_shader: Option<&GlesTexProgram>,
    has_border_shader: bool,
) -> TileRenderElement<R> {
    match elem {
        LayoutElementRenderElement::Wayland(elem) => {
            // If we should clip to geometry, render a clipped window.
            if clip_to_geometry {
                if let Some(shader) = clip_shader {
                    if ClippedSurfaceRenderElement::will_clip(&elem, scale, geo, radius) {
                        return ClippedSurfaceRenderElement::new(
                            elem,
                            scale,
                            geo,
                            shader.clone(),
                            radius,
                        )
                        .into();
                    }
                }
            }

            // Otherwise, render it normally.
            LayoutElementRenderElement::Wayland(elem).into()
        }
        LayoutElementRenderElement::SolidColor(elem) => {
            // In this branch we're rendering a blocked-out window with a solid
            // color. We need to render it with a rounded corner shader even if
            // clip_to_geometry is false, because in this case we're assuming that
            // the unclipped window CSD already has corners rounded to the
            // user-provided radius, so our blocked-out rendering should match that
            // radius.
            if radius != CornerRadius::default() && has_border_shader {
                return BorderRenderElement::new(
                    geo.size,
                    Rectangle::from_size(geo.size),
                    GradientInterpolation::default(),
                    Color::from_color32f(elem.color()),
                    Color::from_color32f(elem.color()),
                    0.,
                    Rectangle::from_size(geo.size),
                    0.,
                    radius,
                    scale.x as f32,
                    1.,
                )
                .with_location(geo.loc)
                .into();
            }

            // Otherwise, render the solid color as is.
            LayoutElementRenderElement::SolidColor(elem).into()
        }
        elem @ LayoutElementRenderElement::BackgroundEffect(_) => {
            // This is only used on popups for now. If subsurface blur is implemented, this
            // will need to be handled somehow.
            error!("background effect clipping is unimplemented");
            elem.into()
        }
    }
}

#[derive(Debug)]
struct ResizeAnimation {
    anim: Animation,
    size_from: Size<f64, Logical>,
    snapshot: LayoutElementRenderSnapshot,
    offscreen: OffscreenBuffer,
    tile_size_from: Size<f64, Logical>,
    // If the resize involved the fullscreen state at some point, this is the progress toward the
    // fullscreen state. Used for things like fullscreen backdrop alpha.
    //
    // Note that this can be set even if this specific resize is between two non-fullscreen states,
    // for example when issuing a new resize during an unfullscreen resize.
    fullscreen_progress: Option<Animation>,
    // Similar to above but for fullscreen-or-maximized.
    expanded_progress: Option<Animation>,
}

#[derive(Debug)]
struct MoveAnimation {
    anim: Animation,
    from: f64,
}

#[derive(Debug)]
pub(super) struct AlphaAnimation {
    pub(super) anim: Animation,
    /// Whether the animation should persist after it's done.
    ///
    /// This is used by things like interactive move which need to animate alpha to
    /// semitransparent, then hold it at semitransparent for a while, until the operation
    /// completes.
    pub(super) hold_after_done: bool,
    offscreen: OffscreenBuffer,
}

#[derive(Debug)]
struct SignalCrossfade {
    anim: Animation,
    level_from: f32,
    level_to: f32,
    /// Straight colors. `None` on one side means that side has no accent;
    /// the other side's color is held for the whole fade and `presence`
    /// carries the fade.
    accent_from: Option<[f32; 3]>,
    accent_to: Option<[f32; 3]>,
    presence_from: f32,
    presence_to: f32,
}

struct MaterialDynamics {
    jelly_fingerprint: JellyFingerprint,
    jelly_uniforms: JellyUniforms,
    signal_fingerprint: SignalFingerprint,
    signal_uniforms: SignalUniforms,
    glass_signal_fingerprint: GlassSignalFingerprint,
    glass_signal: GlassSignalInputs,
    optics: Vec<smithay::backend::renderer::gles::Uniform<'static>>,
}

impl SignalCrossfade {
    /// (level, straight accent color, presence) at the current clock.
    fn current(&self) -> (f32, Option<[f32; 3]>, f32) {
        let t = self.anim.clamped_value() as f32;
        let level = self.level_from + (self.level_to - self.level_from) * t;
        let accent = match (self.accent_from, self.accent_to) {
            (Some(a), Some(b)) => Some([0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        };
        let presence = self.presence_from + (self.presence_to - self.presence_from) * t;
        (level, accent, presence)
    }
}

/// Where a new signal crossfade starts: the current point of a running
/// one (so an interrupted fade continues from where it is), else the last
/// settled target with presence 1 when it had an accent, else quiet.
fn crossfade_origin(
    running: Option<&SignalCrossfade>,
    settled: Option<(f32, Option<[f32; 3]>)>,
) -> (f32, Option<[f32; 3]>, f32) {
    running
        .map(SignalCrossfade::current)
        .or(settled.map(|(level, accent)| (level, accent, if accent.is_some() { 1. } else { 0. })))
        .unwrap_or((0., None, 0.))
}

#[derive(Debug)]
struct FocusCrossfade {
    anim: Animation,
    from: f32,
    to: f32,
}

impl FocusCrossfade {
    fn current(&self) -> f32 {
        self.from + (self.to - self.from) * self.anim.clamped_value() as f32
    }
}

/// One beam of the ring's light around the face on focus gain
/// (design 2026-09-19 §2). Only the start and the speed are snapshotted.
/// The run's end is decided in `material_dynamics`, on the perimeter of the
/// face it has just computed for this frame — never on a cached limit, so a
/// window that grows past an old limit keeps its beam.
#[derive(Debug)]
struct FocusBeam {
    /// Start instant on the unadjusted clock.
    started: Duration,
    /// px/s, snapshotted so a reload mid-run does not jump the head.
    speed: f64,
    /// Set on the first geometry evaluation; disables the unrendered timeout.
    rendered: Cell<bool>,
    /// Set by the frame that saw the head past `ring::run_length` (the tail
    /// cleared, or the decay gone dark); the next `advance_animations` drops
    /// the beam.
    done: Cell<bool>,
}

impl FocusBeam {
    fn elapsed(&self, clock: &Clock) -> Duration {
        clock.now_unadjusted().saturating_sub(self.started)
    }

    fn head_px(&self, clock: &Clock) -> f64 {
        self.speed * self.elapsed(clock).as_secs_f64()
    }

    /// A clock set to complete instantly (the fixtures' fast-forward; in
    /// production it mirrors `animations { off }`, which already cuts the
    /// beam) finishes at once; a rendered frame that saw the run complete
    /// finishes it; and a beam that has never rendered expires at
    /// `BEAM_MAX_RUN`. A rendered beam has no wall-clock cap: even a slow
    /// run must finish its lap and tail on the current geometry.
    fn is_done(&self, clock: &Clock) -> bool {
        clock.should_complete_instantly()
            || self.done.get()
            || (!self.rendered.get() && self.elapsed(clock) >= ring::BEAM_MAX_RUN)
    }
}

impl<W: LayoutElement> Tile<W> {
    pub fn new(
        window: W,
        view_size: Size<f64, Logical>,
        scale: f64,
        clock: Clock,
        options: Rc<Options>,
    ) -> Self {
        let rules = window.rules();
        let border_config = options.layout.border.merged_with(&rules.border);
        let focus_ring_config = options.layout.focus_ring.merged_with(&rules.focus_ring);
        let shadow_config = options.layout.shadow.merged_with(&rules.shadow);
        let sizing_mode = window.sizing_mode();
        let material =
            resolve_material(window.rules().material.as_ref(), &options).map(MaterialState::new);

        Self {
            window,
            border: FocusRing::new(border_config.into()),
            focus_ring: FocusRing::new(focus_ring_config),
            shadow: Shadow::new(shadow_config),
            sizing_mode,
            fullscreen_backdrop: SolidColorBuffer::new((0., 0.), [0., 0., 0., 1.]),
            restore_to_floating: false,
            floating_window_size: None,
            floating_pos: None,
            floating_preset_width_idx: None,
            floating_preset_height_idx: None,
            open_animation: None,
            resize_animation: None,
            move_x_animation: None,
            move_y_animation: None,
            alpha_animation: None,
            interactive_move_offset: Point::from((0., 0.)),
            unmap_snapshot: None,
            rounded_corner_damage: Default::default(),
            view_size,
            scale,
            clock,
            material,
            signal_crossfade: None,
            signal_target: Some((0., None)),
            signal_frame_cache: RefCell::new(None),
            signal_render_visible: false,
            active: false,
            input_active: true,
            focus_crossfade: None,
            focus_beam: None,
            options,
        }
    }

    pub fn update_config(
        &mut self,
        view_size: Size<f64, Logical>,
        scale: f64,
        options: Rc<Options>,
    ) {
        // If preset widths or heights changed, clear our stored preset index.
        if self.options.layout.preset_column_widths != options.layout.preset_column_widths {
            self.floating_preset_width_idx = None;
        }
        if self.options.layout.preset_window_heights != options.layout.preset_window_heights {
            self.floating_preset_height_idx = None;
        }

        self.view_size = view_size;
        self.scale = scale;
        self.options = options;
        self.cut_beam_if_forbidden();

        let round_max1 = |logical| round_logical_in_physical_max1(self.scale, logical);

        let rules = self.window.rules();

        let mut border_config = self.options.layout.border.merged_with(&rules.border);
        border_config.width = round_max1(border_config.width);
        self.border.update_config(border_config.into());

        let mut focus_ring_config = self
            .options
            .layout
            .focus_ring
            .merged_with(&rules.focus_ring);
        focus_ring_config.width = round_max1(focus_ring_config.width);
        self.focus_ring.update_config(focus_ring_config);

        let shadow_config = self.options.layout.shadow.merged_with(&rules.shadow);
        self.shadow.update_config(shadow_config);

        self.window.update_config(self.options.blur);
        self.refresh_material();
    }

    pub fn update_shaders(&mut self) {
        self.border.update_shaders();
        self.focus_ring.update_shaders();
        self.shadow.update_shaders();
    }

    /// Re-resolves this tile's material from its window rules and the current config, keeping the
    /// existing state when only parameters changed.
    fn refresh_material(&mut self) {
        let resolved = resolve_material(self.window.rules().material.as_ref(), &self.options);
        apply_resolved(&mut self.material, resolved.as_ref());
        self.cut_beam_if_forbidden();
    }

    /// Crossfaded focus, 0 to 1.
    fn focus_value(&self) -> f32 {
        self.focus_crossfade
            .as_ref()
            .map(FocusCrossfade::current)
            .unwrap_or(if self.active { 1. } else { 0. })
    }

    /// Whether the policy and response allow the ring's beam to move at
    /// all: `focus ring-light`, `motion full`, animations on. This is the
    /// cut rule's test (design §1); it says nothing about the speed, which
    /// is snapshotted per beam.
    fn beam_allowed(&self, response: &ResolvedResponse) -> bool {
        response.focus == niri_config::FocusResponse::RingLight
            && self.options.signal.motion == niri_config::SignalMotionPolicy::Full
            && !self.options.animations.off
    }

    /// Start eligibility: allowed, and the response asks for a beam.
    fn beam_permitted(&self, response: &ResolvedResponse) -> bool {
        self.beam_allowed(response) && response.ring_beam_speed > 0.
    }

    /// Cut rule: a beam in progress ends at rest the moment motion is no
    /// longer allowed (policy, animations, or the response's focus changed).
    /// A reload that only changes `ring-beam-speed`, to zero included,
    /// leaves the running beam on its snapshotted speed.
    fn cut_beam_if_forbidden(&mut self) {
        let allowed = self
            .material
            .as_ref()
            .is_some_and(|material| self.beam_allowed(&material.material().response(None)));
        if !allowed {
            self.focus_beam = None;
        }
    }

    /// Stage 1 plus crossfade bookkeeping. Returns the effective signal and
    /// the crossfaded frame inputs, or `None` when the window has no signal,
    /// no crossfade is running and the tile is neither focused nor fading.
    fn signal_for_frame(
        &mut self,
        response: &ResolvedResponse,
    ) -> Option<(EffectiveSignal, FrameInputs)> {
        use crate::render_helpers::signal::{color_linear, effective, level_value};

        let folded = self.window.signal(self.clock.now_unadjusted());
        let eff = folded
            .as_ref()
            .map(|folded| {
                effective(
                    folded,
                    self.options.signal.motion,
                    response,
                    self.input_active,
                )
            })
            .unwrap_or(EffectiveSignal {
                accent: None,
                level: niri_ipc::SignalLevel::Quiet,
                motion: niri_ipc::SignalMotion::Static,
                impulses: vec![],
            });
        let target = (level_value(eff.level), eff.accent.map(color_linear));

        if self.signal_target != Some(target) {
            let (from_level, from_accent, from_presence) =
                crossfade_origin(self.signal_crossfade.as_ref(), self.signal_target);
            self.signal_crossfade = Some(SignalCrossfade {
                anim: Animation::new(
                    self.clock.clone(),
                    0.,
                    1.,
                    0.,
                    self.options.animations.material_signal.0,
                ),
                level_from: from_level,
                level_to: target.0,
                accent_from: from_accent,
                accent_to: target.1,
                presence_from: from_presence,
                presence_to: if target.1.is_some() { 1. } else { 0. },
            });
            self.signal_target = Some(target);
        }

        let focus = if response.focus == niri_config::FocusResponse::RingLight {
            self.focus_value()
        } else {
            0.
        };
        let focus_fading = self
            .focus_crossfade
            .as_ref()
            .is_some_and(|crossfade| !crossfade.anim.is_done());
        if folded.is_none() && self.signal_crossfade.is_none() && focus <= 0. && !focus_fading {
            return None;
        }

        let (level, accent, presence) = self
            .signal_crossfade
            .as_ref()
            .map(SignalCrossfade::current)
            .unwrap_or((target.0, target.1, if target.1.is_some() { 1. } else { 0. }));
        // The beam's frame values need the geometry; `material_dynamics`
        // fills them on its local copy of these inputs.
        Some((
            eff,
            FrameInputs {
                level,
                accent,
                presence,
                focus,
                beam: BeamFrame::REST,
            },
        ))
    }

    /// What the optics may depend on this frame beyond their configuration.
    fn optic_frame<'a>(
        &'a self,
        material: &MaterialState,
        logical_now: Duration,
    ) -> OpticFrame<'a> {
        OpticFrame {
            logical_now,
            motion: self.options.signal.motion,
            animations_off: self.options.animations.off,
            backdrop_blur: material.material().glass.backdrop_blur,
            blur: &self.options.blur,
            seed: material.jelly_seed()[0],
        }
    }

    /// The per-frame material state for the rendered `frame` and the
    /// `corner_radius` the element gets: jelly, signal, optics. The one place
    /// the beam's frame values are computed and its run is ended, on the
    /// face this frame draws.
    fn material_dynamics(
        &self,
        material: &MaterialState,
        frame: &MaterialFrame,
        corner_radius: CornerRadius,
        motion_residual: Point<f64, Logical>,
        size_residual: (f64, f64),
        window_size: Size<f64, Logical>,
    ) -> MaterialDynamics {
        let glass = &material.material().glass;
        let response = material.material().response(None);
        let now = self.clock.now_unadjusted();
        let time = self.clock.optic_time(now);
        self.clock.record_optic_render(time.logical_now);
        let optics = optics::values(glass, &self.optic_frame(material, time.logical_now));
        let max_flex = 0.25 * bevel_depth(f64::from(frame.chamfer), glass.thickness);
        let mut jelly = jelly_state(
            motion_residual,
            size_residual,
            window_size,
            glass.jelly_flex,
            max_flex,
        );
        let signal = self.signal_frame_cache.borrow().clone();
        let (signal_fingerprint, glass_signal, signal_uniforms) = match &signal {
            Some((effective, inputs)) => {
                let mut inputs = *inputs;
                if let Some(beam) = self
                    .focus_beam
                    .as_ref()
                    .filter(|beam| !beam.is_done(&self.clock))
                {
                    beam.rendered.set(true);
                    // The face the shader draws this frame: the rendered slab,
                    // its chamfer, the element's fitted radius, the jelly resize.
                    let slab = [
                        f64::from(frame.slab_rect[2] * frame.area_size[0]),
                        f64::from(frame.slab_rect[3] * frame.area_size[1]),
                    ];
                    let face = ring::face(
                        slab,
                        f64::from(frame.chamfer),
                        <[f32; 4]>::from(corner_radius).map(f64::from),
                        [f64::from(jelly.resize[0]), f64::from(jelly.resize[1])],
                    );
                    let perimeter = ring::beam_perimeter(&face, response.ring_gap);
                    let run_px = ring::run_length(perimeter, response.ring_beam_decay);
                    // The one place the run ends: on the geometry just computed,
                    // once the tail has cleared or the decay has gone dark.
                    if beam.head_px(&self.clock) >= run_px {
                        beam.done.set(true);
                    } else {
                        inputs.beam = ring::beam_frame(
                            beam.elapsed(&self.clock),
                            beam.speed,
                            perimeter,
                            ring::BEAM_ENVELOPE,
                            ring::HeadNoise {
                                intensity: response.ring_beam_noise,
                                hz: response.ring_beam_noise_hz,
                                // The pane's own seed keeps two windows
                                // focused together out of step; the run's
                                // start keeps one window's successive runs
                                // from replaying the same wander.
                                seed: material.jelly_seed()[0].to_bits()
                                    ^ beam.started.as_millis() as u32,
                            },
                            response.ring_beam_decay,
                        );
                    }
                }
                let frame = solve(effective, now, material.jelly_seed()[0], inputs);
                let glass_signal = glass_signal_inputs(&frame, glass);
                let uniforms = SignalUniforms::from_frame(&frame, &glass_signal, &response);
                (SignalFingerprint::quantize(&frame), glass_signal, uniforms)
            }
            None => (
                SignalFingerprint::default(),
                GlassSignalInputs::quiet(glass),
                SignalUniforms::quiet(&response),
            ),
        };
        jelly.activity = (jelly.activity + glass_signal.activity_add).min(0.999);
        let time = self.clock.now().as_secs_f64();

        MaterialDynamics {
            jelly_fingerprint: JellyFingerprint::quantize(&jelly, time),
            jelly_uniforms: JellyUniforms {
                move_: jelly.move_,
                resize: jelly.resize,
                activity: jelly.activity,
                time: (time % 3600.) as f32,
            },
            signal_fingerprint,
            signal_uniforms,
            glass_signal_fingerprint: GlassSignalFingerprint::quantize(&glass_signal),
            glass_signal,
            optics,
        }
    }

    pub fn update_window(&mut self) {
        let prev_sizing_mode = self.sizing_mode;
        self.sizing_mode = self.window.sizing_mode();

        if let Some(animate_from) = self.window.take_animation_snapshot() {
            let params = if let Some(resize) = self.resize_animation.take() {
                // Compute like in animated_window_size(), but using the snapshot geometry (since
                // the current one is already overwritten).
                let mut size = animate_from.size;

                let val = resize.anim.value();
                let size_from = resize.size_from;
                let tile_size_from = resize.tile_size_from;

                size.w = size_from.w + (size.w - size_from.w) * val;
                size.h = size_from.h + (size.h - size_from.h) * val;

                let mut tile_size = animate_from.size;
                if prev_sizing_mode.is_fullscreen() {
                    tile_size.w = f64::max(tile_size.w, self.view_size.w);
                    tile_size.h = f64::max(tile_size.h, self.view_size.h);
                } else if prev_sizing_mode.is_normal() && !self.border.is_off() {
                    let width = self.border.width();
                    tile_size.w += width * 2.;
                    tile_size.h += width * 2.;
                }

                tile_size.w = tile_size_from.w + (tile_size.w - tile_size_from.w) * val;
                tile_size.h = tile_size_from.h + (tile_size.h - tile_size_from.h) * val;

                let fullscreen_from = resize
                    .fullscreen_progress
                    .map(|anim| anim.clamped_value().clamp(0., 1.))
                    .unwrap_or(if prev_sizing_mode.is_fullscreen() {
                        1.
                    } else {
                        0.
                    });

                let expanded_from = resize
                    .expanded_progress
                    .map(|anim| anim.clamped_value().clamp(0., 1.))
                    .unwrap_or(if prev_sizing_mode.is_normal() { 0. } else { 1. });

                // Also try to reuse the existing offscreen buffer if we have one.
                (
                    size,
                    tile_size,
                    fullscreen_from,
                    expanded_from,
                    resize.offscreen,
                )
            } else {
                let size = animate_from.size;

                // Compute like in tile_size().
                let mut tile_size = size;
                if prev_sizing_mode.is_fullscreen() {
                    tile_size.w = f64::max(tile_size.w, self.view_size.w);
                    tile_size.h = f64::max(tile_size.h, self.view_size.h);
                } else if prev_sizing_mode.is_normal() && !self.border.is_off() {
                    let width = self.border.width();
                    tile_size.w += width * 2.;
                    tile_size.h += width * 2.;
                }

                let fullscreen_from = if prev_sizing_mode.is_fullscreen() {
                    1.
                } else {
                    0.
                };

                let expanded_from = if prev_sizing_mode.is_normal() { 0. } else { 1. };

                (
                    size,
                    tile_size,
                    fullscreen_from,
                    expanded_from,
                    OffscreenBuffer::default(),
                )
            };
            let (size_from, tile_size_from, fullscreen_from, expanded_from, offscreen) = params;

            let change = self.window.size().to_f64().to_point() - size_from.to_point();
            let change = f64::max(change.x.abs(), change.y.abs());
            let tile_change = self.tile_size().to_f64().to_point() - tile_size_from.to_point();
            let tile_change = f64::max(tile_change.x.abs(), tile_change.y.abs());
            let change = f64::max(change, tile_change);
            if change > RESIZE_ANIMATION_THRESHOLD {
                let anim = Animation::new(
                    self.clock.clone(),
                    0.,
                    1.,
                    0.,
                    self.options.animations.window_resize.anim,
                );

                let fullscreen_to = if self.sizing_mode.is_fullscreen() {
                    1.
                } else {
                    0.
                };
                let expanded_to = if self.sizing_mode.is_normal() { 0. } else { 1. };
                let fullscreen_progress = (fullscreen_from != fullscreen_to)
                    .then(|| anim.restarted(fullscreen_from, fullscreen_to, 0.));
                let expanded_progress = (expanded_from != expanded_to)
                    .then(|| anim.restarted(expanded_from, expanded_to, 0.));

                self.resize_animation = Some(ResizeAnimation {
                    anim,
                    size_from,
                    snapshot: animate_from,
                    offscreen,
                    tile_size_from,
                    fullscreen_progress,
                    expanded_progress,
                });
            } else {
                self.resize_animation = None;
            }
        }

        let round_max1 = |logical| round_logical_in_physical_max1(self.scale, logical);

        let rules = self.window.rules();
        let mut border_config = self.options.layout.border.merged_with(&rules.border);
        border_config.width = round_max1(border_config.width);
        self.border.update_config(border_config.into());

        let mut focus_ring_config = self
            .options
            .layout
            .focus_ring
            .merged_with(&rules.focus_ring);
        focus_ring_config.width = round_max1(focus_ring_config.width);
        self.focus_ring.update_config(focus_ring_config);

        let shadow_config = self.options.layout.shadow.merged_with(&rules.shadow);
        self.shadow.update_config(shadow_config);

        let window_size = self.window_size();
        let radius = self
            .window
            .geometry_corner_radius()
            .fit_to(window_size.w as f32, window_size.h as f32);
        self.rounded_corner_damage.set_corner_radius(radius);

        self.refresh_material();
    }

    pub fn advance_animations(&mut self) {
        if let Some(open) = &mut self.open_animation {
            if open.is_done() {
                self.open_animation = None;
            }
        }

        if let Some(resize) = &mut self.resize_animation {
            if resize.anim.is_done() {
                self.resize_animation = None;
            }
        }

        if let Some(move_) = &mut self.move_x_animation {
            if move_.anim.is_done() {
                self.move_x_animation = None;
            }
        }
        if let Some(move_) = &mut self.move_y_animation {
            if move_.anim.is_done() {
                self.move_y_animation = None;
            }
        }

        if let Some(alpha) = &mut self.alpha_animation {
            if !alpha.hold_after_done && alpha.anim.is_done() {
                self.alpha_animation = None;
            }
        }

        if self
            .signal_crossfade
            .as_ref()
            .is_some_and(|crossfade| crossfade.anim.is_done())
        {
            self.signal_crossfade = None;
        }

        if self
            .focus_crossfade
            .as_ref()
            .is_some_and(|crossfade| crossfade.anim.is_done())
        {
            self.focus_crossfade = None;
        }

        if self
            .focus_beam
            .as_ref()
            .is_some_and(|beam| beam.is_done(&self.clock))
        {
            self.focus_beam = None;
        }
    }

    /// Everything that needs another frame. The beam lives here and not in
    /// `are_transitions_ongoing`: it schedules frames on the animation loop
    /// but is not a layout transition, and `are_transitions_ongoing` also
    /// gates the pointer-focus refresh in `Niri::refresh_pointer_contents`,
    /// which a 16–24 s run must not hold up (design 2026-09-19 §2).
    pub fn are_animations_ongoing(&self) -> bool {
        self.are_transitions_ongoing()
            || self.window.rules().baba_is_float == Some(true)
            || self.signal_render_visible
                && self
                    .focus_beam
                    .as_ref()
                    .is_some_and(|beam| !beam.is_done(&self.clock))
    }

    pub fn are_transitions_ongoing(&self) -> bool {
        self.open_animation.is_some()
            || self.resize_animation.is_some()
            || self.move_x_animation.is_some()
            || self.move_y_animation.is_some()
            || self
                .alpha_animation
                .as_ref()
                .is_some_and(|alpha| !alpha.anim.is_done())
            || self.signal_render_visible
                && (self
                    .signal_crossfade
                    .as_ref()
                    .is_some_and(|crossfade| !crossfade.anim.is_done())
                    || self
                        .signal_frame_cache
                        .borrow()
                        .as_ref()
                        .is_some_and(|(eff, _)| eff.has_live_impulses(self.clock.now_unadjusted()))
                    || self
                        .focus_crossfade
                        .as_ref()
                        .is_some_and(|crossfade| !crossfade.anim.is_done()))
    }

    pub fn clear_signal_render_visibility(&mut self) {
        self.signal_render_visible = false;
    }

    pub fn update_render_elements(
        &mut self,
        is_active: bool,
        input_active: bool,
        visible: bool,
        view_rect: Rectangle<f64, Logical>,
    ) {
        self.input_active = input_active;
        let response = self
            .material
            .as_ref()
            .map(|material| material.material().response(None));
        let lights_focus = response
            .as_ref()
            .is_some_and(|response| response.focus == niri_config::FocusResponse::RingLight);
        if self.active != is_active {
            let from = self.focus_value();
            self.active = is_active;
            self.focus_crossfade = lights_focus.then(|| FocusCrossfade {
                anim: Animation::new(
                    self.clock.clone(),
                    0.,
                    1.,
                    0.,
                    self.options.animations.material_signal.0,
                ),
                from,
                to: if is_active { 1. } else { 0. },
            });
            // Start rule (design 2026-09-19 §2): every gain starts a beam,
            // replacing a running one; loss ends it.
            if is_active {
                if response.as_ref().is_some_and(|r| self.beam_permitted(r)) {
                    self.focus_beam = Some(FocusBeam {
                        started: self.clock.now_unadjusted(),
                        speed: response.as_ref().unwrap().ring_beam_speed,
                        rendered: Cell::new(false),
                        done: Cell::new(false),
                    });
                }
            } else {
                self.focus_beam = None;
            }
        }
        let signal = response
            .as_ref()
            .and_then(|response| self.signal_for_frame(response));
        *self.signal_frame_cache.get_mut() = signal;
        self.signal_render_visible = visible
            && self.material.as_ref().is_some_and(|material| {
                crate::render_helpers::signal::slab_in_view(
                    Point::default(),
                    self.tile_size(),
                    material.material().glass.bevel,
                    view_rect,
                )
            });

        let rules = self.window.rules();
        let animated_tile_size = self.animated_tile_size();
        let expanded_progress = self.expanded_progress();

        let draw_border_with_background = rules
            .draw_border_with_background
            .unwrap_or_else(|| !self.window.has_ssd());
        let border_width = self.visual_border_width().unwrap_or(0.);

        // Do the inverse of tile_size() in order to handle the unfullscreen animation for windows
        // that were smaller than the fullscreen size, and therefore their animated_window_size() is
        // currently much smaller than the tile size.
        let mut border_window_size = animated_tile_size;
        border_window_size.w -= border_width * 2.;
        border_window_size.h -= border_width * 2.;

        // FIXME: this takes into account the animation from normal sizing mode to
        // maximized/fullscreen, but it doesn't take into account the corner radius animation from
        // the window itself.
        //
        // Currently, an easy way to see the problem is to start from a window with a nonzero
        // radius, then go from windowed fullscreen (that forces 0 radius) to regular fullscreen.
        // At the start of the animation, windowed fullscreen becomes false, but the window hasn't
        // animated to the normal fullscreen yet, so the radius here jumps to its nonzero value,
        // even though it should remain zero throughout.
        //
        // Later, when windows get the surface shape protocol with radii, this issue will happen
        // when that changes between animated commits.
        let radius = self
            .window
            .geometry_corner_radius()
            .expanded_by(border_width as f32)
            .scaled_by(1. - expanded_progress as f32);
        self.border.update_render_elements(
            border_window_size,
            is_active,
            !draw_border_with_background,
            self.window.is_urgent(),
            Rectangle::new(
                view_rect.loc - Point::from((border_width, border_width)),
                view_rect.size,
            ),
            radius,
            self.scale,
            1. - expanded_progress as f32,
        );

        let radius = if self.visual_border_width().is_some() {
            radius
        } else {
            self.window
                .geometry_corner_radius()
                .scaled_by(1. - expanded_progress as f32)
        };
        self.shadow.update_render_elements(
            animated_tile_size,
            is_active,
            radius,
            self.scale,
            1. - expanded_progress as f32,
        );

        let draw_focus_ring_with_background = if self.border.is_off() {
            draw_border_with_background
        } else {
            false
        };
        let radius = radius.expanded_by(self.focus_ring.width() as f32);
        self.focus_ring.update_render_elements(
            animated_tile_size,
            is_active,
            !draw_focus_ring_with_background,
            self.window.is_urgent(),
            view_rect,
            radius,
            self.scale,
            1. - expanded_progress as f32,
        );

        self.fullscreen_backdrop.resize(animated_tile_size);
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn render_offset(&self) -> Point<f64, Logical> {
        let mut offset = Point::from((0., 0.));

        if let Some(move_) = &self.move_x_animation {
            offset.x += move_.from * move_.anim.value();
        }
        if let Some(move_) = &self.move_y_animation {
            offset.y += move_.from * move_.anim.value();
        }

        offset += self.interactive_move_offset;

        offset
    }

    /// Decaying move-animation residual: the tile's current render
    /// position minus its target, excluding the non-decaying
    /// interactive-move grab offset.
    pub fn animation_residual(&self) -> Point<f64, Logical> {
        let mut offset = Point::from((0., 0.));
        if let Some(move_) = &self.move_x_animation {
            offset.x += move_.from * move_.anim.value();
        }
        if let Some(move_) = &self.move_y_animation {
            offset.y += move_.from * move_.anim.value();
        }
        offset
    }

    pub fn start_open_animation(&mut self) {
        self.open_animation = Some(OpenAnimation::new(Animation::new(
            self.clock.clone(),
            0.,
            1.,
            0.,
            self.options.animations.window_open.anim,
        )));
    }

    pub fn resize_animation(&self) -> Option<&Animation> {
        self.resize_animation.as_ref().map(|resize| &resize.anim)
    }

    pub fn animate_move_from(&mut self, from: Point<f64, Logical>) {
        self.animate_move_x_from(from.x);
        self.animate_move_y_from(from.y);
    }

    pub fn animate_move_x_from(&mut self, from: f64) {
        self.animate_move_x_from_with_config(from, self.options.animations.window_movement.0);
    }

    pub fn animate_move_x_from_with_config(&mut self, from: f64, config: niri_config::Animation) {
        let current_offset = self.render_offset().x;

        // Preserve the previous config if ongoing.
        let anim = self.move_x_animation.take().map(|move_| move_.anim);
        let anim = anim
            .map(|anim| anim.restarted(1., 0., 0.))
            .unwrap_or_else(|| Animation::new(self.clock.clone(), 1., 0., 0., config));

        self.move_x_animation = Some(MoveAnimation {
            anim,
            from: from + current_offset,
        });
    }

    pub fn animate_move_y_from(&mut self, from: f64) {
        self.animate_move_y_from_with_config(from, self.options.animations.window_movement.0);
    }

    pub fn animate_move_y_from_with_config(&mut self, from: f64, config: niri_config::Animation) {
        let current_offset = self.render_offset().y;

        // Preserve the previous config if ongoing.
        let anim = self.move_y_animation.take().map(|move_| move_.anim);
        let anim = anim
            .map(|anim| anim.restarted(1., 0., 0.))
            .unwrap_or_else(|| Animation::new(self.clock.clone(), 1., 0., 0., config));

        self.move_y_animation = Some(MoveAnimation {
            anim,
            from: from + current_offset,
        });
    }

    pub fn offset_move_y_anim_current(&mut self, offset: f64) {
        if let Some(move_) = self.move_y_animation.as_mut() {
            // If the anim is almost done, there's little point trying to offset it; we can let
            // things jump. If it turns out like a bad idea, we could restart the anim instead.
            let value = move_.anim.value();
            if value > 0.001 {
                move_.from += offset / value;
            }
        }
    }

    pub fn stop_move_animations(&mut self) {
        self.move_x_animation = None;
        self.move_y_animation = None;
    }

    pub fn animate_alpha(&mut self, from: f64, to: f64, config: niri_config::Animation) {
        let from = from.clamp(0., 1.);
        let to = to.clamp(0., 1.);

        let (current, offscreen) = if let Some(alpha) = self.alpha_animation.take() {
            (alpha.anim.clamped_value(), alpha.offscreen)
        } else {
            (from, OffscreenBuffer::default())
        };

        self.alpha_animation = Some(AlphaAnimation {
            anim: Animation::new(self.clock.clone(), current, to, 0., config),
            hold_after_done: false,
            offscreen,
        });
    }

    pub fn ensure_alpha_animates_to_1(&mut self) {
        if let Some(alpha) = &self.alpha_animation {
            if alpha.anim.to() != 1. {
                // Cancel animation instead of starting a new one because the user likely wants to
                // see the tile right away.
                self.alpha_animation = None;
            }
        }
    }

    pub fn hold_alpha_animation_after_done(&mut self) {
        if let Some(alpha) = &mut self.alpha_animation {
            alpha.hold_after_done = true;
        }
    }

    pub fn window(&self) -> &W {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut W {
        &mut self.window
    }

    pub fn sizing_mode(&self) -> SizingMode {
        self.sizing_mode
    }

    fn fullscreen_progress(&self) -> f64 {
        if let Some(resize) = &self.resize_animation {
            if let Some(anim) = &resize.fullscreen_progress {
                return anim.clamped_value().clamp(0., 1.);
            }
        }

        if self.sizing_mode.is_fullscreen() {
            1.
        } else {
            0.
        }
    }

    fn expanded_progress(&self) -> f64 {
        if let Some(resize) = &self.resize_animation {
            if let Some(anim) = &resize.expanded_progress {
                return anim.clamped_value().clamp(0., 1.);
            }
        }

        if self.sizing_mode.is_normal() {
            0.
        } else {
            1.
        }
    }

    /// Returns `None` if the border is hidden and `Some(width)` if it should be shown.
    pub fn effective_border_width(&self) -> Option<f64> {
        if !self.sizing_mode.is_normal() {
            return None;
        }

        if self.border.is_off() {
            return None;
        }

        Some(self.border.width())
    }

    fn visual_border_width(&self) -> Option<f64> {
        if self.border.is_off() {
            return None;
        }

        let expanded_progress = self.expanded_progress();

        // Only hide the border when fully expanded to avoid jarring border appearance.
        if expanded_progress == 1. {
            return None;
        }

        // FIXME: would be cool to, like, gradually resize the border from full width to 0 during
        // fullscreening, but the rest of the code isn't quite ready for that yet. It needs to
        // handle things like computing intermediate tile size when an animated resize starts during
        // an animated unfullscreen resize.
        Some(self.border.width())
    }

    /// Returns the location of the window's visual geometry within this Tile.
    pub fn window_loc(&self) -> Point<f64, Logical> {
        let mut loc = Point::from((0., 0.));

        let window_size = self.animated_window_size();
        let target_size = self.animated_tile_size();

        // Center the window within its tile.
        //
        // - Without borders, the sizes match, so this difference is zero.
        // - Borders always match from all sides, so this difference is pre-rounded to physical.
        // - In fullscreen, if the window is smaller than the tile, then it gets centered, otherwise
        //   the tile size matches the window.
        // - During animations, the window remains centered within the tile; this is important for
        //   the to/from fullscreen animation.
        loc.x += (target_size.w - window_size.w) / 2.;
        loc.y += (target_size.h - window_size.h) / 2.;

        // Round to physical pixels.
        loc = loc
            .to_physical_precise_round(self.scale)
            .to_logical(self.scale);

        loc
    }

    pub fn tile_size(&self) -> Size<f64, Logical> {
        let mut size = self.window_size();

        if self.sizing_mode.is_fullscreen() {
            // Normally we'd just return the fullscreen size here, but this makes things a bit
            // nicer if a fullscreen window is bigger than the fullscreen size for some reason.
            size.w = f64::max(size.w, self.view_size.w);
            size.h = f64::max(size.h, self.view_size.h);
            return size;
        }

        if let Some(width) = self.effective_border_width() {
            size.w += width * 2.;
            size.h += width * 2.;
        }

        size
    }

    pub fn tile_expected_or_current_size(&self) -> Size<f64, Logical> {
        let mut size = self.window_expected_or_current_size();

        if self.sizing_mode.is_fullscreen() {
            // Normally we'd just return the fullscreen size here, but this makes things a bit
            // nicer if a fullscreen window is bigger than the fullscreen size for some reason.
            size.w = f64::max(size.w, self.view_size.w);
            size.h = f64::max(size.h, self.view_size.h);
            return size;
        }

        if let Some(width) = self.effective_border_width() {
            size.w += width * 2.;
            size.h += width * 2.;
        }

        size
    }

    pub fn window_size(&self) -> Size<f64, Logical> {
        let mut size = self.window.size().to_f64();
        size = size
            .to_physical_precise_round(self.scale)
            .to_logical(self.scale);
        size
    }

    pub fn window_expected_or_current_size(&self) -> Size<f64, Logical> {
        let size = self.window.expected_size();
        let mut size = size.unwrap_or_else(|| self.window.size()).to_f64();
        size = size
            .to_physical_precise_round(self.scale)
            .to_logical(self.scale);
        size
    }

    pub fn animated_window_size(&self) -> Size<f64, Logical> {
        let mut size = self.window_size();

        if let Some(resize) = &self.resize_animation {
            let val = resize.anim.value();
            let size_from = resize.size_from.to_f64();

            size.w = f64::max(1., size_from.w + (size.w - size_from.w) * val);
            size.h = f64::max(1., size_from.h + (size.h - size_from.h) * val);
            size = size
                .to_physical_precise_round(self.scale)
                .to_logical(self.scale);
        }

        size
    }

    pub fn animated_tile_size(&self) -> Size<f64, Logical> {
        let mut size = self.tile_size();

        if let Some(resize) = &self.resize_animation {
            let val = resize.anim.value();
            let size_from = resize.tile_size_from.to_f64();

            size.w = f64::max(1., size_from.w + (size.w - size_from.w) * val);
            size.h = f64::max(1., size_from.h + (size.h - size_from.h) * val);
            size = size
                .to_physical_precise_round(self.scale)
                .to_logical(self.scale);
        }

        size
    }

    pub fn buf_loc(&self) -> Point<f64, Logical> {
        let mut loc = Point::from((0., 0.));
        loc += self.window_loc();
        loc += self.window.buf_loc().to_f64();
        loc
    }

    /// Returns a partially-filled [`WindowLayout`].
    ///
    /// Only the sizing properties that a [`Tile`] can fill are filled.
    pub fn ipc_layout_template(&self) -> WindowLayout {
        WindowLayout {
            pos_in_scrolling_layout: None,
            tile_size: self.tile_size().into(),
            window_size: self.window().size().into(),
            tile_pos_in_workspace_view: None,
            window_offset_in_tile: self.window_loc().into(),
        }
    }

    fn is_in_input_region(&self, mut point: Point<f64, Logical>) -> bool {
        point -= self.window_loc().to_f64();
        self.window.is_in_input_region(point)
    }

    fn is_in_activation_region(&self, point: Point<f64, Logical>) -> bool {
        let activation_region = Rectangle::from_size(self.tile_size());
        activation_region.contains(point)
    }

    pub fn hit(&self, point: Point<f64, Logical>) -> Option<HitType> {
        let offset = self.bob_offset();
        let point = point - offset;

        if self.is_in_input_region(point) {
            let win_pos = self.buf_loc() + offset;
            Some(HitType::Input { win_pos })
        } else if self.is_in_activation_region(point) {
            Some(HitType::Activate {
                is_tab_indicator: false,
            })
        } else {
            None
        }
    }

    pub fn request_tile_size(
        &mut self,
        mut size: Size<f64, Logical>,
        animate: bool,
        transaction: Option<Transaction>,
    ) {
        // Can't go through effective_border_width() because we might be fullscreen.
        if !self.border.is_off() {
            let width = self.border.width();
            size.w = f64::max(1., size.w - width * 2.);
            size.h = f64::max(1., size.h - width * 2.);
        }

        // The size request has to be i32 unfortunately, due to Wayland. We floor here instead of
        // round to avoid situations where proportionally-sized columns don't fit on the screen
        // exactly.
        self.window.request_size(
            size.to_i32_floor(),
            SizingMode::Normal,
            animate,
            transaction,
        );
    }

    pub fn tile_width_for_window_width(&self, size: f64) -> f64 {
        if self.border.is_off() {
            size
        } else {
            size + self.border.width() * 2.
        }
    }

    pub fn tile_height_for_window_height(&self, size: f64) -> f64 {
        if self.border.is_off() {
            size
        } else {
            size + self.border.width() * 2.
        }
    }

    pub fn window_width_for_tile_width(&self, size: f64) -> f64 {
        if self.border.is_off() {
            size
        } else {
            size - self.border.width() * 2.
        }
    }

    pub fn window_height_for_tile_height(&self, size: f64) -> f64 {
        if self.border.is_off() {
            size
        } else {
            size - self.border.width() * 2.
        }
    }

    pub fn request_maximized(
        &mut self,
        size: Size<f64, Logical>,
        animate: bool,
        transaction: Option<Transaction>,
    ) {
        self.window.request_size(
            size.to_i32_round(),
            SizingMode::Maximized,
            animate,
            transaction,
        );
    }

    pub fn request_fullscreen(&mut self, animate: bool, transaction: Option<Transaction>) {
        self.window.request_size(
            self.view_size.to_i32_round(),
            SizingMode::Fullscreen,
            animate,
            transaction,
        );
    }

    pub fn min_size_nonfullscreen(&self) -> Size<f64, Logical> {
        let mut size = self.window.min_size().to_f64();

        // Can't go through effective_border_width() because we might be fullscreen.
        if !self.border.is_off() {
            let width = self.border.width();

            size.w = f64::max(1., size.w);
            size.h = f64::max(1., size.h);

            size.w += width * 2.;
            size.h += width * 2.;
        }

        size
    }

    pub fn max_size_nonfullscreen(&self) -> Size<f64, Logical> {
        let mut size = self.window.max_size().to_f64();

        // Can't go through effective_border_width() because we might be fullscreen.
        if !self.border.is_off() {
            let width = self.border.width();

            if size.w > 0. {
                size.w += width * 2.;
            }
            if size.h > 0. {
                size.h += width * 2.;
            }
        }

        size
    }

    pub fn bob_offset(&self) -> Point<f64, Logical> {
        if self.window.rules().baba_is_float != Some(true) {
            return Point::from((0., 0.));
        }

        let y = baba_is_float_offset(self.clock.now(), self.view_size.h);
        let y = round_logical_in_physical(self.scale, y);
        Point::from((0., y))
    }

    fn render_inner<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        location: Point<f64, Logical>,
        mut xray_pos: XrayPos,
        focus_ring: bool,
        motion_residual: Point<f64, Logical>,
        push: &mut dyn FnMut(TileRenderElement<R>),
    ) {
        let _span = tracy_client::span!("Tile::render_inner");

        let scale = Scale::from(self.scale);
        let fullscreen_progress = self.fullscreen_progress();
        let expanded_progress = self.expanded_progress();
        let material_ready = self.material.is_some() && ctx.xray.is_some();
        let win_alpha = if self.window.is_ignoring_opacity_window_rule() {
            1.
        } else {
            let alpha = self.window.rules().opacity.unwrap_or(1.).clamp(0., 1.);

            // Interpolate towards alpha = 1. at fullscreen.
            let p = fullscreen_progress as f32;
            alpha * (1. - p) + 1. * p
        };

        // This is here rather than in render_offset() because render_offset() is currently assumed
        // by the code to be temporary. So, for example, interactive move will try to "grab" the
        // tile at its current render offset and reset the render offset to zero by cancelling the
        // tile move animations. On the other hand, bob_offset() is not resettable, so adding it in
        // render_offset() would cause obvious animation glitches.
        //
        // This isn't to say that adding it here is perfect; indeed, it kind of breaks view_rect
        // passed to update_render_elements(). But, it works well enough for what it is.
        let bob_offset = self.bob_offset();
        let location = location + bob_offset;
        xray_pos = xray_pos.offset(bob_offset);

        let window_loc = self.window_loc();
        let window_size = self.window_size();
        let animated_window_size = self.animated_window_size();
        let size_residual = (
            animated_window_size.w - window_size.w,
            animated_window_size.h - window_size.h,
        );
        let window_render_loc = location + window_loc;
        let area = Rectangle::new(window_render_loc, animated_window_size);
        xray_pos = xray_pos.offset(window_loc);

        let rules = self.window.rules();

        // Clip to geometry including during the fullscreen animation to help with buggy clients
        // that submit a full-sized buffer before acking the fullscreen state (Firefox).
        let clip_to_geometry = fullscreen_progress < 1. && rules.clip_to_geometry == Some(true);
        let radius = self
            .window
            .geometry_corner_radius()
            .scaled_by(1. - expanded_progress as f32);

        // Popups go on top, whether it's resize or not.
        self.window.render_popups(
            ctx.r(),
            window_render_loc,
            scale,
            win_alpha,
            xray_pos,
            &mut |elem| push(elem.into()),
        );

        // If we're resizing, try to render a shader, or a fallback.
        let mut pushed_resize = false;
        if let Some(resize) = &self.resize_animation {
            if ResizeRenderElement::has_shader(ctx.renderer) {
                let mut ctx = ctx.as_gles();

                if let Some(texture_from) = resize.snapshot.texture(ctx.r(), scale) {
                    let mut window_elements = Vec::new();
                    self.window.render_normal(
                        ctx.r(),
                        Point::from((0., 0.)),
                        scale,
                        1.,
                        &mut |elem| window_elements.push(elem),
                    );

                    let current = resize
                        .offscreen
                        .render(ctx.renderer, scale, &window_elements)
                        .map_err(|err| warn!("error rendering window to texture: {err:?}"))
                        .ok();

                    // Clip blocked-out resizes unconditionally because they use solid color render
                    // elements.
                    let clip_to_geometry =
                        if ctx.target.should_block_out(resize.snapshot.block_out_from)
                            && ctx.target.should_block_out(rules.block_out_from)
                        {
                            true
                        } else {
                            clip_to_geometry
                        };

                    if let Some((elem_current, _sync_point, data)) = current {
                        let texture_current = elem_current.texture().clone();
                        // The offset and size are computed in physical pixels and converted to
                        // logical with the same `scale`, so converting them back with rounding
                        // inside the geometry() call gives us the same physical result back.
                        let texture_current_geo = elem_current.geometry(scale);

                        let elem = ResizeRenderElement::new(
                            area,
                            scale,
                            texture_from.clone(),
                            resize.snapshot.size,
                            (texture_current, texture_current_geo),
                            window_size,
                            resize.anim.value() as f32,
                            resize.anim.clamped_value().clamp(0., 1.) as f32,
                            radius,
                            clip_to_geometry,
                            win_alpha,
                        );

                        let mut data = Some(data);
                        let mut done = false;
                        if material_ready {
                            let material = self.material.as_ref().unwrap();
                            let xray = ctx.xray.unwrap();
                            let bg = xray.background[ctx.target as usize].clone();
                            let backdrop = xray.backdrop[ctx.target as usize].clone();

                            let backdrop_blur = material.material().glass.backdrop_blur;
                            if MaterialState::has_program(ctx.renderer)
                                && bg.borrow_mut().prepare(ctx.renderer, backdrop_blur)
                                && backdrop.borrow_mut().prepare(ctx.renderer, backdrop_blur)
                            {
                                let material_elem = elem.clone().with_alpha(1.);
                                match material.offscreen.render(
                                    ctx.renderer,
                                    scale,
                                    std::slice::from_ref(&material_elem),
                                ) {
                                    Ok((offscreen_elem, _sync, mat_data)) => {
                                        let offscreen_geo = offscreen_elem
                                            .geometry(scale)
                                            .to_f64()
                                            .to_logical(scale);
                                        let tex_geo =
                                            Rectangle::new(offscreen_geo.loc, offscreen_geo.size);
                                        let win_geo =
                                            Rectangle::new(window_render_loc, animated_window_size);
                                        let frame = material_frame(
                                            win_geo,
                                            tex_geo,
                                            &material.material().glass,
                                            self.scale,
                                        );
                                        let corner_radius =
                                            radius.fit_to(area.size.w as f32, area.size.h as f32);
                                        let dynamics = self.material_dynamics(
                                            material,
                                            &frame,
                                            corner_radius,
                                            motion_residual,
                                            size_residual,
                                            window_size,
                                        );

                                        let win_texture = offscreen_elem.texture().clone();
                                        let win_src = offscreen_elem.src();
                                        let tex_size = win_texture.size().to_f64();
                                        let win_rect = [
                                            (win_src.loc.x / tex_size.w) as f32,
                                            (win_src.loc.y / tex_size.h) as f32,
                                            (win_src.size.w / tex_size.w) as f32,
                                            (win_src.size.h / tex_size.h) as f32,
                                        ];

                                        let rel = frame.area.loc - window_render_loc;
                                        let geo_in_backdrop = Rectangle::new(
                                            (xray_pos.pos_in_backdrop + rel).upscale(xray_pos.zoom),
                                            frame.area.size.upscale(xray_pos.zoom),
                                        );
                                        let backdrop_size = backdrop.borrow().logical_size();
                                        let mapping = background_mapping(
                                            geo_in_backdrop,
                                            &xray.workspaces,
                                            backdrop_size,
                                        );
                                        let backdrop_color = xray.backdrop_color.components();
                                        let (background_id, background) = {
                                            let bg = bg.borrow();
                                            (bg.id().clone(), bg.commit())
                                        };
                                        let (backdrop_id, backdrop_commit) = {
                                            let backdrop = backdrop.borrow();
                                            (backdrop.id().clone(), backdrop.commit())
                                        };
                                        let inputs = InputFingerprint {
                                            window: offscreen_elem.current_commit(),
                                            background_id,
                                            background,
                                            backdrop_id,
                                            backdrop: backdrop_commit,
                                            mapping: mapping.clone(),
                                            backdrop_color,
                                            corner_radius,
                                            jelly: dynamics.jelly_fingerprint,
                                            signal: dynamics.signal_fingerprint,
                                            glass_signal: dynamics.glass_signal_fingerprint,
                                            optics: dynamics.optics.clone(),
                                        };

                                        let mat_elem = material.element(
                                            frame,
                                            mapping,
                                            dynamics.jelly_uniforms,
                                            dynamics.signal_uniforms,
                                            dynamics.glass_signal,
                                            dynamics.optics,
                                            self.scale,
                                            win_alpha,
                                            ctx.target,
                                            inputs,
                                            win_rect,
                                            win_src,
                                            win_texture,
                                            bg,
                                            backdrop,
                                            backdrop_color,
                                        );

                                        let render_data = data.as_mut().unwrap();
                                        render_data.id = mat_elem.id().clone();
                                        render_data.states.states.extend(mat_data.states.states);
                                        self.window.set_offscreen_data(data.take());
                                        push(mat_elem.into());
                                        pushed_resize = true;
                                        done = true;
                                    }
                                    Err(err) => {
                                        warn!("material: error rendering resize element to offscreen: {err:?}");
                                    }
                                }
                            }
                        }

                        if !done {
                            let mut data = data.unwrap();
                            // We're drawing the resize shader, not the offscreen directly.
                            data.id = elem.id().clone();

                            // This is not a problem for split popups as the code will look for them
                            // by original id when it doesn't find them
                            // on the offscreen.
                            self.window.set_offscreen_data(Some(data));
                            push(elem.into());
                            pushed_resize = true;
                        }
                    }
                }
            }

            if !pushed_resize {
                let fallback_buffer = SolidColorBuffer::new(area.size, [1., 0., 0., 1.]);
                let elem = SolidColorRenderElement::from_buffer(
                    &fallback_buffer,
                    area.loc,
                    win_alpha,
                    Kind::Unspecified,
                );
                push(elem.into());
                pushed_resize = true;
            }
        }

        // If we're not resizing, render the window itself.
        let has_border_shader = BorderRenderElement::has_shader(ctx.renderer);
        if !pushed_resize {
            let mut pushed_material = false;
            if material_ready {
                let material = self.material.as_ref().unwrap();
                let mut ctx = ctx.as_gles();

                if MaterialState::has_program(ctx.renderer) {
                    let clip_shader = ClippedSurfaceRenderElement::shader(ctx.renderer).cloned();
                    let clip_geo = Rectangle::from_size(window_size);
                    let clip_radius = radius.fit_to(window_size.w as f32, window_size.h as f32);
                    let mut window_elements: Vec<TileRenderElement<GlesRenderer>> = Vec::new();
                    self.window.render_normal(
                        ctx.r(),
                        Point::from((0., 0.)),
                        scale,
                        1.,
                        &mut |elem| {
                            window_elements.push(clip_window_element(
                                elem,
                                scale,
                                clip_geo,
                                clip_radius,
                                clip_to_geometry,
                                clip_shader.as_ref(),
                                has_border_shader,
                            ))
                        },
                    );
                    if clip_to_geometry && clip_shader.is_some() {
                        window_elements.push(self.rounded_corner_damage.render(clip_geo).into());
                    }

                    let xray = ctx.xray.unwrap();
                    let bg = xray.background[ctx.target as usize].clone();
                    let backdrop = xray.backdrop[ctx.target as usize].clone();
                    let backdrop_blur = material.material().glass.backdrop_blur;
                    if bg.borrow_mut().prepare(ctx.renderer, backdrop_blur)
                        && backdrop.borrow_mut().prepare(ctx.renderer, backdrop_blur)
                    {
                        match material
                            .offscreen
                            .render(ctx.renderer, scale, &window_elements)
                        {
                            Ok((offscreen_elem, _sync, mut data)) => {
                                let offscreen_geo =
                                    offscreen_elem.geometry(scale).to_f64().to_logical(scale);
                                let tex_geo = Rectangle::new(
                                    window_render_loc + offscreen_geo.loc,
                                    offscreen_geo.size,
                                );
                                let win_geo =
                                    Rectangle::new(window_render_loc, animated_window_size);
                                let frame = material_frame(
                                    win_geo,
                                    tex_geo,
                                    &material.material().glass,
                                    self.scale,
                                );
                                let dynamics = self.material_dynamics(
                                    material,
                                    &frame,
                                    clip_radius,
                                    motion_residual,
                                    size_residual,
                                    window_size,
                                );

                                let win_texture = offscreen_elem.texture().clone();
                                let win_src = offscreen_elem.src();
                                let tex_size = win_texture.size().to_f64();
                                let win_rect = [
                                    (win_src.loc.x / tex_size.w) as f32,
                                    (win_src.loc.y / tex_size.h) as f32,
                                    (win_src.size.w / tex_size.w) as f32,
                                    (win_src.size.h / tex_size.h) as f32,
                                ];

                                let rel = frame.area.loc - window_render_loc;
                                let geo_in_backdrop = Rectangle::new(
                                    (xray_pos.pos_in_backdrop + rel).upscale(xray_pos.zoom),
                                    frame.area.size.upscale(xray_pos.zoom),
                                );
                                let backdrop_size = backdrop.borrow().logical_size();
                                let mapping = background_mapping(
                                    geo_in_backdrop,
                                    &xray.workspaces,
                                    backdrop_size,
                                );
                                let backdrop_color = xray.backdrop_color.components();
                                let (background_id, background) = {
                                    let bg = bg.borrow();
                                    (bg.id().clone(), bg.commit())
                                };
                                let (backdrop_id, backdrop_commit) = {
                                    let backdrop = backdrop.borrow();
                                    (backdrop.id().clone(), backdrop.commit())
                                };
                                let inputs = InputFingerprint {
                                    window: offscreen_elem.current_commit(),
                                    background_id,
                                    background,
                                    backdrop_id,
                                    backdrop: backdrop_commit,
                                    mapping: mapping.clone(),
                                    backdrop_color,
                                    corner_radius: clip_radius,
                                    jelly: dynamics.jelly_fingerprint,
                                    signal: dynamics.signal_fingerprint,
                                    glass_signal: dynamics.glass_signal_fingerprint,
                                    optics: dynamics.optics.clone(),
                                };

                                let elem = material.element(
                                    frame,
                                    mapping,
                                    dynamics.jelly_uniforms,
                                    dynamics.signal_uniforms,
                                    dynamics.glass_signal,
                                    dynamics.optics,
                                    self.scale,
                                    win_alpha,
                                    ctx.target,
                                    inputs,
                                    win_rect,
                                    win_src,
                                    win_texture,
                                    bg,
                                    backdrop,
                                    backdrop_color,
                                );

                                data.id = elem.id().clone();
                                self.window.set_offscreen_data(Some(data));
                                push(elem.into());
                                pushed_material = true;
                            }
                            Err(err) => {
                                warn!("material: error rendering window to offscreen: {err:?}");
                            }
                        }
                    }
                }
            }

            if !pushed_material {
                let geo = Rectangle::new(window_render_loc, window_size);
                let radius = radius.fit_to(window_size.w as f32, window_size.h as f32);

                let clip_shader = ClippedSurfaceRenderElement::shader(ctx.renderer).cloned();

                if clip_to_geometry && clip_shader.is_some() {
                    let damage = self.rounded_corner_damage.render(geo);
                    push(damage.into());
                }

                self.window.render_normal(
                    ctx.r(),
                    window_render_loc,
                    scale,
                    win_alpha,
                    &mut |elem| {
                        push(clip_window_element(
                            elem,
                            scale,
                            geo,
                            radius,
                            clip_to_geometry,
                            clip_shader.as_ref(),
                            has_border_shader,
                        ))
                    },
                );
            }
        }

        if fullscreen_progress > 0. {
            let alpha = fullscreen_progress as f32;

            // During the un/fullscreen animation, render a border element in order to use the
            // animated corner radius.
            if fullscreen_progress < 1. && has_border_shader {
                let border_width = self.visual_border_width().unwrap_or(0.);
                let radius = self
                    .window
                    .geometry_corner_radius()
                    .expanded_by(border_width as f32)
                    .scaled_by(1. - expanded_progress as f32);

                let size = self.fullscreen_backdrop.size();
                let color = self.fullscreen_backdrop.color();
                let elem = BorderRenderElement::new(
                    size,
                    Rectangle::from_size(size),
                    GradientInterpolation::default(),
                    Color::from_color32f(color),
                    Color::from_color32f(color),
                    0.,
                    Rectangle::from_size(size),
                    0.,
                    radius,
                    scale.x as f32,
                    alpha,
                )
                .with_location(location);
                push(elem.into());
            } else {
                let elem = SolidColorRenderElement::from_buffer(
                    &self.fullscreen_backdrop,
                    location,
                    alpha,
                    Kind::Unspecified,
                );
                push(elem.into());
            }
        }

        if let Some(width) = self.visual_border_width() {
            self.border.render(
                ctx.renderer,
                location + Point::from((width, width)),
                &mut |elem| push(elem.into()),
            );
        }

        // Hide the focus ring when maximized/fullscreened. It's not normally visible anyway due to
        // being outside the monitor or obscured by a solid colored bar, but it is visible under
        // semitransparent bars in maximized state (which is a bit weird) and in the overview (also
        // a bit weird).
        if focus_ring && expanded_progress < 1. {
            self.focus_ring
                .render(ctx.renderer, location, &mut |elem| push(elem.into()));
        }

        if expanded_progress < 1. {
            self.shadow
                .render(ctx.renderer, location, &mut |elem| push(elem.into()));
        }

        let surface_anim_scale = animated_window_size / window_size;
        self.window.render_background_effect(
            ctx.as_gles(),
            area,
            self.scale,
            clip_to_geometry,
            surface_anim_scale,
            radius,
            xray_pos,
            &mut |elem| push(elem.into()),
        );
    }

    pub fn render<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        location: Point<f64, Logical>,
        xray_pos: XrayPos,
        focus_ring: bool,
        motion_residual: Point<f64, Logical>,
        push: &mut dyn FnMut(TileRenderElement<R>),
    ) {
        let _span = tracy_client::span!("Tile::render");

        if let Some(ticks) = &ctx.signal_ticks {
            if let Some(deadline) =
                self.tick_deadline(location, ticks.view.get(), self.clock.now_unadjusted())
            {
                ticks.report(deadline);
            }
        }

        let scale = Scale::from(self.scale);

        let tile_alpha = self
            .alpha_animation
            .as_ref()
            .map_or(1., |alpha| alpha.anim.clamped_value()) as f32;

        let mut pushed = false;
        self.window().set_offscreen_data(None);

        if let Some(open) = &self.open_animation {
            let mut ctx = ctx.as_gles();
            let mut elements = Vec::new();
            self.render_inner(
                ctx.r(),
                Point::new(0., 0.),
                xray_pos,
                focus_ring,
                motion_residual,
                &mut |elem| elements.push(elem),
            );
            match open.render(
                ctx.renderer,
                &elements,
                self.animated_tile_size(),
                location,
                scale,
                tile_alpha,
            ) {
                Ok((elem, data)) => {
                    self.window().set_offscreen_data(Some(data));
                    push(elem.into());
                    pushed = true;
                }
                Err(err) => {
                    warn!("error rendering window opening animation: {err:?}");
                }
            }
        } else if let Some(alpha) = &self.alpha_animation {
            let mut ctx = ctx.as_gles();
            let mut elements = Vec::new();
            self.render_inner(
                ctx.r(),
                Point::new(0., 0.),
                xray_pos,
                focus_ring,
                motion_residual,
                &mut |elem| elements.push(elem),
            );
            match alpha.offscreen.render(ctx.renderer, scale, &elements) {
                Ok((elem, _sync, data)) => {
                    let offset = elem.offset();
                    let elem = elem.with_alpha(tile_alpha).with_offset(location + offset);

                    self.window().set_offscreen_data(Some(data));
                    push(elem.into());
                    pushed = true;
                }
                Err(err) => {
                    warn!("error rendering tile to offscreen for alpha animation: {err:?}");
                }
            }
        }

        if !pushed {
            self.render_inner(
                ctx,
                location,
                xray_pos,
                focus_ring,
                motion_residual,
                &mut |elem| push(elem),
            );
        }
    }

    /// Whether the input-activity gate changes what this tile renders: its
    /// window's signal has sustained motion while input is active. The gate
    /// touches nothing else (design 2026-09-18 §3), so a tile without such a
    /// signal needs no redraw when the gate flips.
    pub fn attention_gated(&self) -> bool {
        use crate::render_helpers::signal::effective;

        let Some(material) = &self.material else {
            return false;
        };
        let response = material.material().response(None);
        self.window
            .signal(self.clock.now_unadjusted())
            .is_some_and(|folded| {
                effective(&folded, self.options.signal.motion, &response, true).is_sustained()
            })
    }

    /// Next instant this tile needs a redraw for its material: the earliest
    /// of any optic's own change and the sustained-signal bucket boundary,
    /// while the slab band is in view. The focus beam runs on the animation
    /// loop and has no arm here (design 2026-09-18 §2).
    ///
    /// The optic deadline does not sit behind the signal frame cache: an
    /// unfocused, signal-free window has no cache, and an animated optic
    /// must keep it redrawing all the same (design §3).
    pub fn tick_deadline(
        &self,
        location: Point<f64, Logical>,
        view: Rectangle<f64, Logical>,
        now: Duration,
    ) -> Option<Duration> {
        use crate::render_helpers::signal::{slab_in_view, tick_deadline};

        let material = self.material.as_ref()?;
        let glass = &material.material().glass;
        if !slab_in_view(location, self.tile_size(), glass.bevel, view) {
            return None;
        }
        let time = self.clock.optic_time(now);
        let optic =
            optics::next_change(glass, &self.optic_frame(material, time.logical_now), &time);
        let signal = self
            .signal_frame_cache
            .borrow()
            .as_ref()
            .and_then(|(eff, _)| tick_deadline(eff, true, now));
        match (optic, signal) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    pub fn store_unmap_snapshot_if_empty(
        &mut self,
        renderer: &mut GlesRenderer,
        xray: Option<&mut Xray>,
        xray_has_blocked_out_layers: bool,
        xray_pos: XrayPos,
        motion_residual: Point<f64, Logical>,
    ) {
        if self.unmap_snapshot.is_some() {
            return;
        }

        self.unmap_snapshot = Some(self.render_snapshot(
            renderer,
            xray,
            xray_has_blocked_out_layers,
            xray_pos,
            motion_residual,
        ));
    }

    fn render_snapshot(
        &self,
        renderer: &mut GlesRenderer,
        mut xray: Option<&mut Xray>,
        xray_has_blocked_out_layers: bool,
        xray_pos: XrayPos,
        motion_residual: Point<f64, Logical>,
    ) -> TileRenderSnapshot {
        let _span = tracy_client::span!("Tile::render_snapshot");

        let mut contents = Vec::new();
        self.render(
            RenderCtx {
                target: RenderTarget::Output,
                renderer,
                xray: xray.as_deref(),
                signal_ticks: None,
            },
            Point::from((0., 0.)),
            xray_pos,
            false,
            motion_residual,
            &mut |elem| contents.push(elem),
        );

        let mut contents_with_blocked_out_bg = None;

        // Do a bit of pointer surgery on Xray.
        //
        // The idea is to avoid the combinatorial combination of rendering snapshots for target
        // (Output, Screencast) × Xray target (Output, Screencast, ScreenCapture).
        //
        // Our main goals:
        // - Everything must look unblocked for RenderTarget::Output.
        // - If anything is potentially blocked-out, it must not show up on any screen capture.
        //
        // Right above we rendered a fully-unblocked snapshot for the Output, so that's covered.
        //
        // Next, *only if Xray has any blocked-out surfaces* (which is a rare case), we will render
        // a snapshot where the window itself is unblocked, but the Xray background is blocked. To
        // do this, we swap the Output target buffers in Xray with the Screencast target buffers
        // (which were prepared for us higher up the stack).
        //
        // Finally, we render a fully blocked-out snapshot. If Xray has blocked-out surfaces, then
        // Xray's Screencast buffers are already filled-in, but if not, then we swap in the Output
        // buffers, to avoid an extra render. This is safe since we know there are no blocked
        // surfaces there.
        let output_idx = RenderTarget::Output as usize;
        let screencast_idx = RenderTarget::Screencast as usize;
        let mut screencast_background = None;
        let mut screencast_backdrop = None;
        let mut output_background = None;
        let mut output_backdrop = None;
        if let Some(xray) = &mut xray {
            screencast_background = Some(Rc::clone(&xray.background[screencast_idx]));
            screencast_backdrop = Some(Rc::clone(&xray.backdrop[screencast_idx]));
            output_background = Some(Rc::clone(&xray.background[output_idx]));
            output_backdrop = Some(Rc::clone(&xray.backdrop[output_idx]));

            if xray_has_blocked_out_layers {
                xray.background[output_idx] = screencast_background.clone().unwrap();
                xray.backdrop[output_idx] = screencast_backdrop.clone().unwrap();

                let mut contents = Vec::new();
                self.render(
                    RenderCtx {
                        target: RenderTarget::Output,
                        renderer,
                        xray: Some(xray),
                        signal_ticks: None,
                    },
                    Point::from((0., 0.)),
                    xray_pos,
                    false,
                    motion_residual,
                    &mut |elem| contents.push(elem),
                );
                contents_with_blocked_out_bg = Some(contents);
            } else {
                xray.background[screencast_idx] = output_background.clone().unwrap();
                xray.backdrop[screencast_idx] = output_backdrop.clone().unwrap();
            }
        }

        // A bit of a hack to render blocked out as for screencast, but I think it's fine here.
        let mut blocked_out_contents = Vec::new();
        self.render(
            RenderCtx {
                target: RenderTarget::Screencast,
                renderer,
                xray: xray.as_deref(),
                signal_ticks: None,
            },
            Point::from((0., 0.)),
            xray_pos,
            false,
            motion_residual,
            &mut |elem| blocked_out_contents.push(elem),
        );

        // Put everything back to normal.
        if let Some(xray) = &mut xray {
            if xray_has_blocked_out_layers {
                xray.background[output_idx] = output_background.take().unwrap();
                xray.backdrop[output_idx] = output_backdrop.take().unwrap();
            } else {
                xray.background[screencast_idx] = screencast_background.take().unwrap();
                xray.backdrop[screencast_idx] = screencast_backdrop.take().unwrap();
            }
        }

        RenderSnapshot {
            contents,
            contents_with_blocked_out_bg,
            blocked_out_contents,
            block_out_from: self.window.rules().block_out_from,
            size: self.animated_tile_size(),
            texture: Default::default(),
            texture_with_blocked_out_bg: Default::default(),
            blocked_out_texture: Default::default(),
        }
    }

    pub fn take_unmap_snapshot(&mut self) -> Option<TileRenderSnapshot> {
        self.unmap_snapshot.take()
    }

    pub fn border(&self) -> &FocusRing {
        &self.border
    }

    pub fn focus_ring(&self) -> &FocusRing {
        &self.focus_ring
    }

    pub fn options(&self) -> &Rc<Options> {
        &self.options
    }

    pub fn material(&self) -> Option<&MaterialState> {
        self.material.as_ref()
    }

    #[cfg(test)]
    pub fn view_size(&self) -> Size<f64, Logical> {
        self.view_size
    }

    #[cfg(test)]
    pub fn verify_invariants(&self) {
        use approx::assert_abs_diff_eq;

        assert_eq!(self.sizing_mode, self.window.sizing_mode());

        let scale = self.scale;
        let size = self.tile_size();
        let rounded = size.to_physical_precise_round(scale).to_logical(scale);
        assert_abs_diff_eq!(size.w, rounded.w, epsilon = 1e-5);
        assert_abs_diff_eq!(size.h, rounded.h, epsilon = 1e-5);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::animation::Clock;
    use crate::layout::tests::{TestWindow, TestWindowParams};
    use crate::render_helpers::material::optics::{self, OpticFrame};
    use crate::window::ResolvedWindowRules;

    fn options_for(glass: niri_config::ResolvedGlass, blur: niri_config::Blur) -> Options {
        let material = niri_config::ResolvedMaterial {
            name: String::from("frost"),
            glass,
            responses: vec![(
                String::from("default"),
                niri_config::ResolvedResponse::default(),
            )],
        };
        Options {
            materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
            blur,
            ..Default::default()
        }
    }

    fn options_with(name: &str, backdrop_blur: bool, blur_off: bool) -> Options {
        assert_eq!(name, "frost", "the fixture defines one material");
        options_for(
            niri_config::ResolvedGlass {
                backdrop_blur,
                ..Default::default()
            },
            niri_config::Blur {
                off: blur_off,
                ..Default::default()
            },
        )
    }

    /// The post-stage uniforms a tile would upload for the material "frost"
    /// under `options`: what the renderer sees, after the inherit rule.
    fn post_uniforms(options: &Options) -> Vec<smithay::backend::renderer::gles::Uniform<'static>> {
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        let resolved = resolve_material(Some(&reference), options).unwrap();
        let frame = OpticFrame {
            logical_now: Duration::ZERO,
            motion: options.signal.motion,
            animations_off: options.animations.off,
            backdrop_blur: resolved.material.glass.backdrop_blur,
            blur: &options.blur,
            seed: 0.,
        };
        optics::values(&resolved.material.glass, &frame)
    }

    fn uniform_f32(
        uniforms: &[smithay::backend::renderer::gles::Uniform<'static>],
        name: &str,
    ) -> f32 {
        match uniforms.iter().find(|u| u.name == name).unwrap().value {
            smithay::backend::renderer::gles::UniformValue::_1f(v) => v,
            ref other => panic!("{name}: {other:?}"),
        }
    }

    #[test]
    fn backdrop_blur_survives_resolution_when_global_blur_is_on() {
        let options = options_with("frost", true, false);
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        let resolved = resolve_material(Some(&reference), &options).unwrap();
        assert!(resolved.material.glass.backdrop_blur);
    }

    #[test]
    fn backdrop_blur_is_off_when_the_global_switch_is_off() {
        // `blur { off }` means off. BlurOptions carries only passes and offset,
        // so nothing downstream would honor `off` if it were not applied here.
        let options = options_with("frost", true, true);
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        let resolved = resolve_material(Some(&reference), &options).unwrap();
        assert!(!resolved.material.glass.backdrop_blur);
    }

    #[test]
    fn backdrop_blur_stays_off_when_the_material_opts_out() {
        let options = options_with("frost", false, false);
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        let resolved = resolve_material(Some(&reference), &options).unwrap();
        assert!(!resolved.material.glass.backdrop_blur);
    }

    #[test]
    fn material_postprocess_follows_effective_backdrop_blur() {
        for (material_blur, global_off, expected) in [
            (true, false, (0.07, 0.8)),
            (false, false, (0.0, 1.0)),
            (true, true, (0.0, 1.0)),
        ] {
            let mut options = options_with("frost", material_blur, global_off);
            options.blur.noise = 0.07;
            options.blur.saturation = 0.8;

            let uniforms = post_uniforms(&options);
            assert_eq!(
                (
                    uniform_f32(&uniforms, "mat_noise"),
                    uniform_f32(&uniforms, "mat_saturation")
                ),
                expected
            );
        }
    }

    #[test]
    fn written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur() {
        // Non-neutral globals so "inherited" and "neutral" are distinguishable
        // from "written", and so an explicit neutral value is visibly a choice.
        let global = niri_config::Blur {
            noise: 0.02,
            saturation: 1.5,
            ..Default::default()
        };
        let global_off = niri_config::Blur {
            off: true,
            ..global
        };
        // (written noise, written saturation, backdrop-blur, blur block, expected pair)
        for (noise, saturation, backdrop_blur, blur, expected) in [
            // both written: the pair survives every switch
            (Some(0.3), Some(0.5), true, global, (0.3, 0.5)),
            (Some(0.3), Some(0.5), false, global, (0.3, 0.5)),
            (Some(0.3), Some(0.5), true, global_off, (0.3, 0.5)),
            // both omitted: today's inheritance
            (None, None, true, global, (0.02, 1.5)),
            (None, None, false, global, (0., 1.)),
            (None, None, true, global_off, (0., 1.)),
            // one written, one omitted: each side decided on its own
            (Some(0.3), None, true, global, (0.3, 1.5)),
            (Some(0.3), None, false, global, (0.3, 1.)),
            (None, Some(0.5), true, global, (0.02, 0.5)),
            (None, Some(0.5), false, global, (0., 0.5)),
            // explicit neutral beats non-neutral globals
            (Some(0.), Some(1.), true, global, (0., 1.)),
        ] {
            let options = options_for(
                niri_config::ResolvedGlass {
                    noise: niri_config::ResolvedNoise {
                        amount: noise,
                        ..Default::default()
                    },
                    saturation: niri_config::ResolvedSaturation { amount: saturation },
                    backdrop_blur,
                    ..Default::default()
                },
                blur,
            );
            let uniforms = post_uniforms(&options);
            assert_eq!(
                (
                    uniform_f32(&uniforms, "mat_noise"),
                    uniform_f32(&uniforms, "mat_saturation")
                ),
                expected,
                "noise {noise:?} saturation {saturation:?} backdrop {backdrop_blur} off {}",
                blur.off
            );
        }
    }

    #[test]
    fn noise_type_reaches_the_render_config_regardless_of_blur() {
        let global_off = niri_config::Blur {
            off: true,
            ..Default::default()
        };
        for (noise_type, backdrop_blur, blur) in [
            (
                niri_config::NoiseType::Fine,
                false,
                niri_config::Blur::default(),
            ),
            (niri_config::NoiseType::Lightness, true, global_off),
            (
                niri_config::NoiseType::White,
                true,
                niri_config::Blur::default(),
            ),
        ] {
            let options = options_for(
                niri_config::ResolvedGlass {
                    noise: niri_config::ResolvedNoise {
                        amount: Some(0.3),
                        kind: noise_type,
                    },
                    backdrop_blur,
                    ..Default::default()
                },
                blur,
            );
            let uniforms = post_uniforms(&options);
            assert_eq!(
                uniform_f32(&uniforms, "mat_noise_type"),
                noise_type as u8 as f32
            );
            assert_eq!(uniform_f32(&uniforms, "mat_noise"), 0.3);
        }
    }

    #[test]
    fn a_name_the_config_does_not_define_resolves_to_nothing() {
        // Reachable only between a reload's layout update and its rule
        // recompute. Panicking here took the whole compositor down when a
        // live material was renamed or removed.
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        assert!(resolve_material(Some(&reference), &Options::default()).is_none());
    }

    #[test]
    fn signal_crossfade_carries_presence_and_straight_color() {
        use crate::animation::{Animation, Clock};

        // `Clock` is shared by clone (`src/animation/clock.rs`): advancing the
        // test's handle advances the animation's.
        let mut clock = Clock::with_time(Duration::ZERO);
        let config = niri_config::animations::MaterialSignalAnim::default().0; // 400 ms ease-out-cubic
        let anim = Animation::new(clock.clone(), 0., 1., 0., config);
        clock.set_unadjusted(Duration::from_millis(200));
        let t = anim.clamped_value() as f32;
        assert!(t > 0. && t < 1., "mid-fade: {t}");

        // Arrival: the color is the arriving color throughout, presence rises with t.
        let arriving = SignalCrossfade {
            anim,
            level_from: 0.,
            level_to: 1.,
            accent_from: None,
            accent_to: Some([1., 0.5, 0.]),
            presence_from: 0.,
            presence_to: 1.,
        };
        let (_, accent, presence) = arriving.current();
        assert_eq!(accent, Some([1., 0.5, 0.]));
        assert!((presence - t).abs() < 1e-6);

        // Expiry: the color holds the last live color, presence falls.
        let expiring = SignalCrossfade {
            accent_from: Some([1., 0.5, 0.]),
            accent_to: None,
            presence_from: 1.,
            presence_to: 0.,
            ..arriving
        };
        let (_, accent, presence) = expiring.current();
        assert_eq!(accent, Some([1., 0.5, 0.]));
        assert!((presence - (1. - t)).abs() < 1e-6);

        // Live to live: straight interpolation by t, presence stays 1.
        let changing = SignalCrossfade {
            accent_from: Some([1., 0., 0.]),
            accent_to: Some([0., 0., 1.]),
            presence_from: 1.,
            presence_to: 1.,
            ..expiring
        };
        let (_, accent, presence) = changing.current();
        assert_eq!(accent, Some([1. - t, 0., t]));
        assert_eq!(presence, 1.);

        // Interrupted mid-fade: the next crossfade starts from the current point,
        // not from the settled target.
        let origin = crossfade_origin(Some(&changing), Some((0., None)));
        assert_eq!(origin, changing.current());
        assert_eq!(
            crossfade_origin(None, Some((0.5, Some([1., 0., 0.])))),
            (0.5, Some([1., 0., 0.]), 1.),
            "settled accent has presence 1"
        );
        assert_eq!(crossfade_origin(None, None), (0., None, 0.));
    }

    /// A tile carrying the material "frost" with the given glass, whose only
    /// response selects `focus`. Everything else is stock: `motion "full"`,
    /// animations on.
    fn material_tile(
        glass: niri_config::ResolvedGlass,
        focus: niri_config::FocusResponse,
        clock: Clock,
    ) -> Tile<TestWindow> {
        beam_tile(
            glass,
            niri_config::ResolvedResponse {
                focus,
                ..Default::default()
            },
            clock,
        )
    }

    fn focus_tile(focus: niri_config::FocusResponse, clock: Clock) -> Tile<TestWindow> {
        material_tile(niri_config::ResolvedGlass::default(), focus, clock)
    }

    #[test]
    fn focus_none_never_crossfades_the_filament() {
        // `focus "none"` has to cost a focus change nothing: no crossfade is
        // started, so the tile never reports itself as transitioning and the
        // output never wakes for it.
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::None, clock);
        let view = Rectangle::from_size(Size::from((1280., 720.)));

        for is_active in [true, false, true] {
            tile.update_render_elements(is_active, true, true, view);
            assert!(tile.focus_crossfade.is_none(), "active {is_active}");
            assert!(!tile.are_transitions_ongoing(), "active {is_active}");
        }
    }

    #[test]
    fn static_optics_report_no_deadline_without_a_signal_cache() {
        // An unfocused tile whose response lights nothing and that carries no
        // signal has no signal frame cache. Optic deadlines are still
        // evaluated; with only static optics registered they are None, and
        // the call must not short-circuit on the missing cache.
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::None, clock);
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(false, true, true, view);
        assert!(tile.signal_frame_cache.borrow().is_none());
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::ZERO),
            None
        );
    }

    #[test]
    fn an_unfocused_signal_free_aurora_tile_reports_its_next_bucket() {
        // The scheduling gate of the optics design §3: no focus filament, no
        // signal, so no signal frame cache, and still a deadline from the
        // optic while the slab band is in view.
        let mut clock = Clock::with_time(Duration::ZERO);
        let lit = niri_config::ResolvedGlass {
            aurora: niri_config::ResolvedAurora {
                amount: 0.5,
                drift_hz: 4.,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut tile = material_tile(lit, niri_config::FocusResponse::None, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(false, true, true, view);
        assert!(!tile.active);
        assert!(tile.signal_frame_cache.borrow().is_none());
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::ZERO),
            Some(Duration::from_millis(250))
        );
        let far = Point::from((10_000., 10_000.));
        assert_eq!(tile.tick_deadline(far, view, Duration::ZERO), None);

        clock.set_unadjusted(Duration::from_millis(100));
        let held = render_dynamics(&tile, 1280., 720.).optics;
        clock.set_optic_active(false, Duration::from_millis(100));
        clock.set_unadjusted(Duration::from_secs(1));
        assert_eq!(render_dynamics(&tile, 1280., 720.).optics, held);
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::from_secs(1)),
            None
        );
        assert_eq!(tile.tick_deadline(far, view, Duration::from_secs(1)), None);
        clock.set_optic_active(true, Duration::from_secs(1));
        assert_eq!(render_dynamics(&tile, 1280., 720.).optics, held);
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::from_secs(1)),
            Some(Duration::from_millis(1_150))
        );

        let pinned = niri_config::ResolvedGlass {
            aurora: niri_config::ResolvedAurora {
                amount: 0.5,
                drift_hz: 0.,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut tile = material_tile(pinned, niri_config::FocusResponse::None, clock);
        tile.update_render_elements(false, true, true, view);
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::ZERO),
            None
        );
    }

    /// A tile carrying the material "frost" with the given glass and
    /// response. Everything else is stock: `motion "full"`, animations on.
    fn beam_tile(
        glass: niri_config::ResolvedGlass,
        response: niri_config::ResolvedResponse,
        clock: Clock,
    ) -> Tile<TestWindow> {
        let material = niri_config::ResolvedMaterial {
            name: String::from("frost"),
            glass,
            responses: vec![(String::from("default"), response)],
        };
        let options = Options {
            materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
            ..Default::default()
        };
        let mut params = TestWindowParams::new(1);
        params.rules = Some(ResolvedWindowRules {
            material: Some(MaterialRef {
                name: String::from("frost"),
                response: None,
            }),
            ..Default::default()
        });
        Tile::new(
            TestWindow::new(params),
            Size::from((1280., 720.)),
            1.,
            clock,
            Rc::new(options),
        )
    }

    /// A flat slab: no bevel, no offset, so the face is the window itself.
    fn flat_glass() -> niri_config::ResolvedGlass {
        niri_config::ResolvedGlass {
            bevel: 0.,
            offset_x: 0.,
            offset_y: 0.,
            ..Default::default()
        }
    }

    fn beam_of(tile: &Tile<TestWindow>) -> Option<(Duration, f64, bool)> {
        tile.focus_beam
            .as_ref()
            .map(|b| (b.started, b.speed, b.done.get()))
    }

    /// The frame a render of a `w × h` window at (0, 0) builds, exactly as
    /// `render_inner` does with the window texture covering the window.
    fn frame_for(tile: &Tile<TestWindow>, w: f64, h: f64) -> MaterialFrame {
        let geo = Rectangle::from_size(Size::from((w, h)));
        let glass = &tile.material.as_ref().unwrap().material().glass;
        material_frame(geo, geo, glass, 1.)
    }

    /// What a render of a `w × h` window computes this frame: the dynamics
    /// `render_inner` hands the element, with zero radii and zero residuals.
    fn render_dynamics(tile: &Tile<TestWindow>, w: f64, h: f64) -> MaterialDynamics {
        let material = tile.material.as_ref().unwrap();
        let frame = frame_for(tile, w, h);
        tile.material_dynamics(
            material,
            &frame,
            CornerRadius::default(),
            Point::default(),
            (0., 0.),
            Size::from((w, h)),
        )
    }

    /// The face of that render and its beam perimeter and tail, from the same
    /// frame values `material_dynamics` reads.
    fn geometry_of(tile: &Tile<TestWindow>, w: f64, h: f64) -> (ring::Face, f64, f64) {
        let frame = frame_for(tile, w, h);
        let slab = [
            f64::from(frame.slab_rect[2] * frame.area_size[0]),
            f64::from(frame.slab_rect[3] * frame.area_size[1]),
        ];
        let face = ring::face(slab, f64::from(frame.chamfer), [0.; 4], [0., 0.]);
        let gap = tile
            .material
            .as_ref()
            .unwrap()
            .material()
            .response(None)
            .ring_gap;
        let p = ring::beam_perimeter(&face, gap);
        (face, p, ring::tail_length(p))
    }

    /// The beam part of the focus uniform: head, env, decay.
    fn beam_uniforms(dynamics: &MaterialDynamics) -> [f32; 3] {
        let f = dynamics.signal_uniforms.focus;
        [f[1], f[2], f[3]]
    }

    fn secs(t: f64) -> Duration {
        Duration::from_secs_f64(t)
    }

    #[test]
    fn settled_focus_reports_no_deadline_and_no_transition() {
        // Already focused, crossfade done, beam done: the three settled
        // gates of the design (§2).
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock);
        tile.active = true;
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        assert!(tile.focus_crossfade.is_none());
        assert_eq!(beam_of(&tile), None, "no change of focus, no beam");
        assert!(tile.signal_frame_cache.borrow().is_some());
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::ZERO),
            None
        );
        assert!(!tile.are_transitions_ongoing());
    }

    #[test]
    fn every_focus_gain_starts_a_beam_and_loss_ends_it() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));

        tile.update_render_elements(false, true, true, view);
        assert_eq!(beam_of(&tile), None);

        clock.set_unadjusted(Duration::from_millis(100));
        tile.update_render_elements(true, true, true, view);
        assert_eq!(
            beam_of(&tile),
            Some((Duration::from_millis(100), 300., false))
        );
        assert!(tile.are_animations_ongoing());

        // A second gain a second later (another tile took focus, then this
        // one got it back) starts a fresh beam at the new instant.
        clock.set_unadjusted(Duration::from_millis(1100));
        tile.update_render_elements(false, true, true, view);
        tile.update_render_elements(true, true, true, view);
        assert_eq!(
            beam_of(&tile),
            Some((Duration::from_millis(1100), 300., false))
        );

        // Loss ends it.
        clock.set_unadjusted(Duration::from_millis(1500));
        tile.update_render_elements(false, true, true, view);
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn the_beam_ends_only_when_a_rendered_frame_sees_the_tail_clear() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        let (_, p, l) = geometry_of(&tile, 400., 300.);
        // slab 412 × 312, chamfer 12, gap 8: a 372 × 272 line
        assert_eq!((p, l), (1288., 322.));
        let run = (p + l) / 300.;

        let first = render_dynamics(&tile, 400., 300.);
        assert!(tile.focus_beam.as_ref().unwrap().rendered.get());
        assert_eq!(beam_uniforms(&first), [0., 0., 1.], "launch: head at 0");

        clock.set_unadjusted(secs(run - 0.1));
        tile.update_render_elements(true, true, true, view);
        let near_end = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!(tile.are_animations_ongoing());
        assert!((beam_uniforms(&near_end)[0] - 300. * (run - 0.1) as f32).abs() < 1e-2);

        // Past the run, `advance_animations` alone changes nothing: no
        // frame has seen the new time against the geometry.
        clock.set_unadjusted(secs(run + 0.1));
        tile.advance_animations();
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        tile.update_render_elements(true, true, true, view);
        let ended = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&ended), [0., 0., 0.], "rest uniforms");
        assert!(!tile.are_animations_ongoing());
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);

        // Settled: the same fingerprint later, and no deadline.
        clock.set_unadjusted(secs(run + 5.));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let later = render_dynamics(&tile, 400., 300.);
        assert_eq!(later.signal_fingerprint, ended.signal_fingerprint);
        assert_eq!(beam_uniforms(&later), [0., 0., 0.]);
        assert_eq!(
            tile.tick_deadline(Point::default(), view, secs(run + 5.)),
            None
        );
    }

    #[test]
    fn growth_past_an_old_limit_keeps_the_beam() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        let (_, p1, l1) = geometry_of(&tile, 400., 300.);
        let (_, p2, l2) = geometry_of(&tile, 800., 600.);
        assert!(p2 + l2 > p1 + l1 + 200.);
        render_dynamics(&tile, 400., 300.);

        // One px past the old limit, the window has grown before any frame
        // evaluates: the beam stays, its head where elapsed time puts it.
        let t = (p1 + l1 + 1.) / 300.;
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        assert!(beam_of(&tile).is_some());
        tile.update_render_elements(true, true, true, view);
        let grown = render_dynamics(&tile, 800., 600.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!((beam_uniforms(&grown)[0] - (300. * t) as f32).abs() < 1e-2);
        assert!(tile.are_animations_ongoing());

        // The shrink: past the old limit, under the new one, back to the
        // small geometry — the fresh evaluation ends the run.
        let t = (p1 + l1 + 100.) / 300.;
        assert!(300. * t < p2 + l2);
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let still = render_dynamics(&tile, 800., 600.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!((beam_uniforms(&still)[0] - (300. * t) as f32).abs() < 1e-2);
        let shrunk = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&shrunk), [0., 0., 0.]);
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn a_resize_during_the_tail_phase_behaves_the_same() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        let (_, p1, l1) = geometry_of(&tile, 400., 300.);
        let (_, p2, l2) = geometry_of(&tile, 800., 600.);
        render_dynamics(&tile, 400., 300.);

        // Mid-drain on the small face: the head is past P₁, env is 0.
        let t = (p1 + 50.) / 300.;
        clock.set_unadjusted(secs(t));
        tile.update_render_elements(true, true, true, view);
        let draining = render_dynamics(&tile, 400., 300.);
        assert_eq!(
            beam_uniforms(&draining)[1],
            0.,
            "env is 0 through the drain"
        );
        assert!((beam_uniforms(&draining)[0] - (300. * t) as f32).abs() < 1e-2);

        // The window grows: the beam persists, the head keeps counting
        // from the same start, and the run now ends on the new geometry.
        // The head is never repositioned (design 2026-09-19 §2), so on the
        // longer line `elapsed < P₂ / speed` again: the head re-enters the
        // new lap, `env` returns to 1 and the head visibly reappears.
        let t = (p1 + l1 + 1.) / 300.;
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        assert!(beam_of(&tile).is_some());
        tile.update_render_elements(true, true, true, view);
        let grown = render_dynamics(&tile, 800., 600.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!((beam_uniforms(&grown)[0] - (300. * t) as f32).abs() < 1e-2);
        assert_eq!(beam_uniforms(&grown)[2], 1., "plateau decay");
        assert!(tile.are_animations_ongoing());

        let t = (p2 + l2) / 300. + 0.1;
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let ended = render_dynamics(&tile, 800., 600.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&ended), [0., 0., 0.]);
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn a_beam_nobody_renders_ends_at_the_backstop() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        assert!(beam_of(&tile).is_some());

        clock.set_unadjusted(ring::BEAM_MAX_RUN - Duration::from_secs(1));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        assert!(beam_of(&tile).is_some());
        assert!(!tile.focus_beam.as_ref().unwrap().rendered.get());
        assert!(tile.are_animations_ongoing());

        clock.set_unadjusted(ring::BEAM_MAX_RUN + Duration::from_secs(1));
        assert!(!tile.are_animations_ongoing());
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }

    /// The slow fixture of the design's timeout cases: a flat 1800 × 1200
    /// slab, gap 0, zero radii and zero jelly (`P = 6000`, `L = 1200`) at
    /// 50 px/s: a 120 s lap and a 144 s run.
    fn slow_tile(clock: Clock) -> Tile<TestWindow> {
        let response = niri_config::ResolvedResponse {
            ring_gap: 0.,
            ring_beam_speed: 50.,
            ..Default::default()
        };
        let tile = beam_tile(flat_glass(), response, clock);
        let (face, p, l) = geometry_of(&tile, 1800., 1200.);
        assert_eq!(face.half, [900., 600.]);
        assert_eq!((p, l), (6000., 1200.));
        tile
    }

    #[test]
    fn a_rendered_slow_beam_outlives_the_unrendered_backstop() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = slow_tile(clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        assert_eq!(beam_of(&tile), Some((Duration::ZERO, 50., false)));

        clock.set_unadjusted(secs(10.));
        tile.update_render_elements(true, true, true, view);
        let early = render_dynamics(&tile, 1800., 1200.);
        assert!(tile.focus_beam.as_ref().unwrap().rendered.get());
        assert_eq!(beam_uniforms(&early), [500., 1., 1.]);

        for t in [60., 119.] {
            clock.set_unadjusted(secs(t));
            tile.advance_animations();
            tile.update_render_elements(true, true, true, view);
            let mid = render_dynamics(&tile, 1800., 1200.);
            assert_eq!(beam_uniforms(&mid)[0], (50. * t) as f32);
            assert!(beam_of(&tile).is_some());
        }

        // Past the unrendered backstop: a rendered beam has no wall-clock
        // cap, and its tail is still draining.
        clock.set_unadjusted(secs(121.));
        tile.advance_animations();
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        tile.update_render_elements(true, true, true, view);
        let draining = render_dynamics(&tile, 1800., 1200.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert_eq!(beam_uniforms(&draining), [6050., 0., 1.]);
        assert!(tile.are_animations_ongoing());

        clock.set_unadjusted(secs(122.));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let later = render_dynamics(&tile, 1800., 1200.);
        assert_ne!(
            later.signal_fingerprint, draining.signal_fingerprint,
            "the tail still advances"
        );
        assert_eq!(beam_uniforms(&later), [6100., 0., 1.]);

        clock.set_unadjusted(secs(144.));
        tile.advance_animations();
        assert!(beam_of(&tile).is_some());
        tile.update_render_elements(true, true, true, view);
        let ended = render_dynamics(&tile, 1800., 1200.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&ended), [0., 0., 0.]);
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn hiding_a_rendered_beam_does_not_rearm_the_backstop() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = slow_tile(clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);

        clock.set_unadjusted(secs(10.));
        tile.update_render_elements(true, true, true, view);
        render_dynamics(&tile, 1800., 1200.);
        assert!(tile.focus_beam.as_ref().unwrap().rendered.get());

        // Hidden past 120 s without a render: the beam is retained, its
        // redraws suppressed by visibility, the backstop not re-armed.
        for t in [30., 121.] {
            clock.set_unadjusted(secs(t));
            tile.advance_animations();
            tile.update_render_elements(true, true, false, view);
            assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
            assert!(tile.focus_beam.as_ref().unwrap().rendered.get());
            assert!(!tile.are_animations_ongoing(), "hidden at {t} s");
        }

        // Revealed at 121 s: the tail is draining.
        tile.update_render_elements(true, true, true, view);
        assert!(tile.are_animations_ongoing());
        let revealed = render_dynamics(&tile, 1800., 1200.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert_eq!(beam_uniforms(&revealed), [6050., 0., 1.]);

        // Hidden again and revealed after 144 s instead: fresh geometry ends
        // the beam and the frame returns rest.
        clock.set_unadjusted(secs(130.));
        tile.advance_animations();
        tile.update_render_elements(true, true, false, view);
        clock.set_unadjusted(secs(150.));
        tile.advance_animations();
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        tile.update_render_elements(true, true, true, view);
        let ended = render_dynamics(&tile, 1800., 1200.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&ended), [0., 0., 0.]);
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn the_beam_is_skipped_under_reduced_off_animations_off_and_zero_speed() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let cases: [(niri_config::SignalMotionPolicy, bool, f64); 4] = [
            (niri_config::SignalMotionPolicy::Reduced, false, 300.),
            (niri_config::SignalMotionPolicy::Off, false, 300.),
            (niri_config::SignalMotionPolicy::Full, true, 300.),
            (niri_config::SignalMotionPolicy::Full, false, 0.),
        ];
        for (policy, animations_off, speed) in cases {
            let mut clock = Clock::with_time(Duration::ZERO);
            let response = niri_config::ResolvedResponse {
                ring_beam_speed: speed,
                ..Default::default()
            };
            let mut tile = beam_tile(Default::default(), response, clock.clone());
            let mut options = (*tile.options).clone();
            options.signal.motion = policy;
            options.animations.off = animations_off;
            tile.options = Rc::new(options);
            tile.update_render_elements(true, true, true, view);
            let label = format!("{policy:?} animations_off={animations_off} speed={speed}");
            assert_eq!(beam_of(&tile), None, "{label}");
            clock.set_unadjusted(secs(1.));
            tile.update_render_elements(true, true, true, view);
            let dynamics = render_dynamics(&tile, 400., 300.);
            assert_eq!(beam_uniforms(&dynamics), [0., 0., 0.], "{label}");
        }
    }

    #[test]
    fn only_sustained_attention_is_gated_by_input() {
        // The idle gate redraws only tiles it changes: a window whose signal
        // moves while input is active. No signal, a static one, or motion
        // "off" render the same either way.
        let signal = |motion| crate::window::signal::Folded {
            level: niri_ipc::SignalLevel::Demand,
            motion,
            accent: None,
            tag: None,
            sources: vec![String::from("demo")],
            impulses: vec![],
        };
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock);
        assert!(!tile.attention_gated(), "no signal");

        tile.window()
            .set_signal(Some(signal(niri_ipc::SignalMotion::Static)));
        assert!(!tile.attention_gated(), "static signal");

        tile.window()
            .set_signal(Some(signal(niri_ipc::SignalMotion::Pulse)));
        assert!(tile.attention_gated(), "pulsing signal");

        let mut options = (*tile.options).clone();
        options.signal.motion = niri_config::SignalMotionPolicy::Off;
        tile.options = Rc::new(options);
        assert!(!tile.attention_gated(), "motion off");
    }

    #[test]
    fn idle_input_freezes_attention_but_not_the_beam() {
        let clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock);
        tile.window()
            .set_signal(Some(crate::window::signal::Folded {
                level: niri_ipc::SignalLevel::Demand,
                motion: niri_ipc::SignalMotion::Pulse,
                accent: None,
                tag: None,
                sources: vec![String::from("demo")],
                impulses: vec![],
            }));
        let view = Rectangle::from_size(Size::from((1280., 720.)));

        tile.update_render_elements(true, true, true, view);
        let (eff, _) = tile.signal_frame_cache.borrow().clone().unwrap();
        assert_eq!(eff.motion, niri_ipc::SignalMotion::Pulse);
        assert!(tile
            .tick_deadline(Point::default(), view, Duration::ZERO)
            .is_some());

        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        tile.window()
            .set_signal(Some(crate::window::signal::Folded {
                level: niri_ipc::SignalLevel::Demand,
                motion: niri_ipc::SignalMotion::Pulse,
                accent: None,
                tag: None,
                sources: vec![String::from("demo")],
                impulses: vec![],
            }));
        tile.update_render_elements(true, false, true, view);
        assert!(
            beam_of(&tile).is_some(),
            "the beam is finite and not gated on idle"
        );
        let (eff, _) = tile.signal_frame_cache.borrow().clone().unwrap();
        assert_eq!(eff.motion, niri_ipc::SignalMotion::Static);
        assert_eq!(
            eff.level,
            niri_ipc::SignalLevel::Demand,
            "level is untouched"
        );
        assert_eq!(
            tile.tick_deadline(Point::default(), view, Duration::ZERO),
            None,
            "frozen attention reports no bucket deadline"
        );
        clock.set_unadjusted(secs(1.));
        tile.update_render_elements(true, false, true, view);
        let dynamics = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_uniforms(&dynamics), [300., 1., 1.], "the beam runs");
    }

    /// `frost` with the given response, everything else stock.
    fn frost_options(response: niri_config::ResolvedResponse) -> Options {
        let material = niri_config::ResolvedMaterial {
            name: String::from("frost"),
            glass: niri_config::ResolvedGlass::default(),
            responses: vec![(String::from("default"), response)],
        };
        Options {
            materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
            ..Default::default()
        }
    }

    #[test]
    fn a_speed_reload_keeps_the_running_beam_and_a_focus_none_reload_cuts_it() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let size = Size::from((1280., 720.));
        tile.update_render_elements(true, true, true, view);
        assert_eq!(beam_of(&tile).map(|b| b.1), Some(300.));

        // A reload that changes `ring-beam-speed` leaves the running beam on
        // its snapshotted speed; the new value applies to the next beam.
        let faster = niri_config::ResolvedResponse {
            ring_beam_speed: 900.,
            ..Default::default()
        };
        tile.update_config(size, 1., Rc::new(frost_options(faster)));
        assert_eq!(beam_of(&tile).map(|b| b.1), Some(300.));

        // Even a reload to zero is not a cut: the beam finishes.
        let none = niri_config::ResolvedResponse {
            ring_beam_speed: 0.,
            ..Default::default()
        };
        tile.update_config(size, 1., Rc::new(frost_options(none)));
        assert_eq!(beam_of(&tile).map(|b| b.1), Some(300.));

        // ...and with zero configured, the next gain starts nothing.
        clock.set_unadjusted(Duration::from_millis(2000));
        tile.update_render_elements(false, true, true, view);
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        assert_eq!(beam_of(&tile), None);

        // A reload to `focus "none"` cuts a running beam at once.
        let mut tile = focus_tile(
            niri_config::FocusResponse::RingLight,
            Clock::with_time(Duration::ZERO),
        );
        tile.update_render_elements(true, true, true, view);
        assert!(beam_of(&tile).is_some());
        let lights_off = niri_config::ResolvedResponse {
            focus: niri_config::FocusResponse::None,
            ..Default::default()
        };
        tile.update_config(size, 1., Rc::new(frost_options(lights_off)));
        assert_eq!(beam_of(&tile), None);

        // So does a policy change to reduced.
        let mut tile = focus_tile(
            niri_config::FocusResponse::RingLight,
            Clock::with_time(Duration::ZERO),
        );
        tile.update_render_elements(true, true, true, view);
        assert!(beam_of(&tile).is_some());
        let mut options = (*tile.options).clone();
        options.signal.motion = niri_config::SignalMotionPolicy::Reduced;
        tile.update_config(size, 1., Rc::new(options));
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn env_is_zero_through_the_drain_and_the_head_keeps_moving() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        let (_, p, l) = geometry_of(&tile, 400., 300.);
        render_dynamics(&tile, 400., 300.);

        let t = p / 300. + 0.5;
        assert!(t + 0.1 < (p + l) / 300., "still within the drain");
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let drain = render_dynamics(&tile, 400., 300.);
        let [head, env, decay] = beam_uniforms(&drain);
        assert_eq!(env, 0.);
        assert!((head - (p + 150.) as f32).abs() < 1e-2, "{head}");
        assert_eq!(decay, 1.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!(tile.are_animations_ongoing());

        clock.set_unadjusted(secs(t + 0.1));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let next = render_dynamics(&tile, 400., 300.);
        let [head2, env2, _] = beam_uniforms(&next);
        assert_eq!(env2, 0.);
        assert!(head2 > head + 29., "{head2} vs {head}");
        assert_ne!(next.signal_fingerprint, drain.signal_fingerprint);
        // Only the head moved: the rest of the frame is settled.
        assert_eq!(
            next.signal_uniforms.focus[0],
            drain.signal_uniforms.focus[0]
        );
        assert_eq!(next.signal_uniforms.accent, drain.signal_uniforms.accent);
        assert_eq!(next.signal_uniforms.level, drain.signal_uniforms.level);
    }

    #[test]
    fn a_flat_slab_carries_the_beam() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), Default::default(), clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let (face, p, _) = geometry_of(&tile, 400., 300.);
        assert_eq!(frame_for(&tile, 400., 300.).chamfer, 0.);
        assert_eq!(face.half, [200., 150.], "the face is the slab");
        assert_eq!(p, 4. * (192. + 142.));

        tile.update_render_elements(true, true, true, view);
        clock.set_unadjusted(secs(1.));
        tile.update_render_elements(true, true, true, view);
        let dynamics = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_uniforms(&dynamics), [300., 1., 1.], "a live beam");
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
    }

    #[test]
    fn the_head_wander_reaches_the_uniform_and_stops_with_the_beam() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let response = niri_config::ResolvedResponse {
            ring_beam_noise: 0.6,
            ring_beam_noise_hz: 5.,
            ..Default::default()
        };
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), response, clock.clone());
        let (_, p, l) = geometry_of(&tile, 400., 300.);
        tile.update_render_elements(true, true, true, view);

        // Through the plateau the head amplitude leaves 1, while the head
        // position and the shared decay stay exactly where they were.
        let mut wandered = false;
        let mut t = 0.4;
        while t < (p / 300.) - 0.4 {
            clock.set_unadjusted(secs(t));
            tile.update_render_elements(true, true, true, view);
            let [head, env, decay] = beam_uniforms(&render_dynamics(&tile, 400., 300.));
            assert!((head - (300. * t) as f32).abs() < 1e-2, "head at {t}");
            assert_eq!(decay, 1., "plateau decay at {t}");
            assert!((0.4..=1.6).contains(&env), "env {env} out of band at {t}");
            wandered |= (env - 1.).abs() > 0.05;
            t += 0.05;
        }
        assert!(wandered, "the head never left its envelope amplitude");

        // Past the lap the head amplitude is exactly zero while the tail
        // drains, and the settled frame is the rest uniform: the wander
        // never puts the quiet ring back on a clock.
        clock.set_unadjusted(secs((p / 300.) + 0.5));
        tile.update_render_elements(true, true, true, view);
        assert_eq!(beam_uniforms(&render_dynamics(&tile, 400., 300.))[1], 0.);
        clock.set_unadjusted(secs(((p + l) / 300.) + 0.5));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        assert_eq!(
            beam_uniforms(&render_dynamics(&tile, 400., 300.)),
            [0., 0., 0.]
        );
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }

    #[test]
    fn the_wander_and_decay_are_skipped_with_the_beam_under_reduced_and_off() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        for (policy, animations_off) in [
            (niri_config::SignalMotionPolicy::Reduced, false),
            (niri_config::SignalMotionPolicy::Off, false),
            (niri_config::SignalMotionPolicy::Full, true),
        ] {
            let mut clock = Clock::with_time(Duration::ZERO);
            let response = niri_config::ResolvedResponse {
                ring_beam_noise: 1.,
                ring_beam_noise_hz: 20.,
                ring_beam_decay: 600.,
                ..Default::default()
            };
            let mut tile = beam_tile(flat_glass(), response, clock.clone());
            let mut options = (*tile.options).clone();
            options.signal.motion = policy;
            options.animations.off = animations_off;
            tile.options = Rc::new(options);
            tile.update_render_elements(true, true, true, view);
            let label = format!("{policy:?} animations_off={animations_off}");
            assert_eq!(beam_of(&tile), None, "{label}");
            for t in [0.1, 0.5, 1., 2.] {
                clock.set_unadjusted(secs(t));
                tile.update_render_elements(true, true, true, view);
                let dynamics = render_dynamics(&tile, 400., 300.);
                assert_eq!(beam_uniforms(&dynamics), [0., 0., 0.], "{label} at {t}");
            }
        }
    }

    #[test]
    fn a_decay_shorter_than_the_lap_ends_the_run_where_the_comet_goes_dark() {
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let response = niri_config::ResolvedResponse {
            ring_beam_decay: 600.,
            ..Default::default()
        };
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = beam_tile(flat_glass(), response, clock.clone());
        let (_, p, _) = geometry_of(&tile, 400., 300.);
        assert!(p > 600., "the decay must fall inside the lap: P = {p}");
        tile.update_render_elements(true, true, true, view);
        render_dynamics(&tile, 400., 300.);

        // Halfway to dark: a quarter of the brightness, the head on its path.
        clock.set_unadjusted(secs(1.));
        tile.update_render_elements(true, true, true, view);
        let [head, _, decay] = beam_uniforms(&render_dynamics(&tile, 400., 300.));
        assert!((head - 300.).abs() < 1e-2);
        assert!((decay - 0.25).abs() < 1e-6, "decay {decay}");
        assert!(tile.are_animations_ongoing());

        // 600 px at 300 px/s is dark at 2 s, long before the lap closes:
        // the first frame past it ends the run, so the redraws stop there.
        clock.set_unadjusted(secs(2.1));
        tile.update_render_elements(true, true, true, view);
        let ended = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&ended), [0., 0., 0.], "rest uniforms");
        assert!(!tile.are_animations_ongoing());
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);

        // The resting ring is the same constant it is without the knob.
        clock.set_unadjusted(secs(10.));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let later = render_dynamics(&tile, 400., 300.);
        assert_eq!(later.signal_fingerprint, ended.signal_fingerprint);
        assert_eq!(tile.tick_deadline(Point::default(), view, secs(10.)), None);
    }

    #[test]
    fn the_beam_schedules_frames_without_holding_layout_transitions() {
        // `are_transitions_ongoing` also gates the pointer-focus refresh in
        // `Niri::refresh_pointer_contents`; a beam run must not hold it.
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        tile.update_render_elements(true, true, true, view);
        let (_, p, l) = geometry_of(&tile, 400., 300.);
        render_dynamics(&tile, 400., 300.);

        // Mid-run, the focus crossfade long settled.
        clock.set_unadjusted(secs(2.));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        assert!(tile.focus_crossfade.is_none());
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!(tile.are_animations_ongoing(), "the beam wants frames");
        assert!(
            !tile.are_transitions_ongoing(),
            "but is not a layout transition"
        );

        // At rest, neither.
        clock.set_unadjusted(secs((p + l) / 300. + 0.1));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert!(!tile.are_animations_ongoing());
        assert!(!tile.are_transitions_ongoing());
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
        assert!(!tile.are_animations_ongoing());
    }

    #[test]
    fn a_gap_reload_mid_run_changes_the_perimeter_but_not_the_head() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut tile = focus_tile(niri_config::FocusResponse::RingLight, clock.clone());
        let view = Rectangle::from_size(Size::from((1280., 720.)));
        let size = Size::from((1280., 720.));
        tile.update_render_elements(true, true, true, view);
        let (_, p1, l1) = geometry_of(&tile, 400., 300.);
        render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile), Some((Duration::ZERO, 300., false)));

        // Reload to a wider gap: the beam is the same one, on a shorter line.
        let wider = niri_config::ResolvedResponse {
            ring_gap: 16.,
            ..Default::default()
        };
        tile.update_config(size, 1., Rc::new(frost_options(wider)));
        assert_eq!(beam_of(&tile), Some((Duration::ZERO, 300., false)));
        let (_, p2, l2) = geometry_of(&tile, 400., 300.);
        assert!(p2 < p1, "a wider gap shortens the line: {p2} vs {p1}");
        assert!(p2 + l2 + 50. < p1 + l1);

        // Under both limits: running, the head at `speed · elapsed`.
        let t = (p2 + l2 - 50.) / 300.;
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        tile.update_render_elements(true, true, true, view);
        let running = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        assert!((beam_uniforms(&running)[0] - (300. * t) as f32).abs() < 1e-2);

        // Past the new limit, still under the old one: the fresh geometry
        // ends the run where the old perimeter would have kept it going.
        let t = (p2 + l2 + 50.) / 300.;
        assert!(300. * t < p1 + l1);
        clock.set_unadjusted(secs(t));
        tile.advance_animations();
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(false));
        tile.update_render_elements(true, true, true, view);
        let ended = render_dynamics(&tile, 400., 300.);
        assert_eq!(beam_of(&tile).map(|b| b.2), Some(true));
        assert_eq!(beam_uniforms(&ended), [0., 0., 0.]);
        tile.advance_animations();
        assert_eq!(beam_of(&tile), None);
    }
}
