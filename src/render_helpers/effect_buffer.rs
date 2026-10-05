use std::mem;

use anyhow::{ensure, Context as _};
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::{Id, RenderElementStates};
use smithay::backend::renderer::gles::{GlesFrame, GlesRenderer, GlesTexture};
use smithay::backend::renderer::utils::CommitCounter;
use smithay::backend::renderer::{
    Bind as _, Color32F, ContextId, FrameContext as _, Offscreen as _, Renderer as _, Texture,
};
use smithay::utils::{Buffer, Logical, Physical, Scale, Size, Transform};

use crate::niri::OutputRenderElements;
use crate::render_helpers::blur::{Blur, BlurOptions, BlurProgram};
use crate::render_helpers::grain::{GrainOptions, GrainProgram};
use crate::render_helpers::shaders::Shaders;

/// Why a cache is being thrown away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Invalidation {
    /// The sharp texture was redrawn.
    SharpDamage,
    /// The agreed backdrop grain changed, including to or from none.
    GrainOptionsChanged,
    /// The global blur passes or offset changed.
    BlurOptionsChanged,
}

/// What an invalidation clears, and whether consumers must be told through
/// the commit counter (design §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cleared {
    grain: bool,
    blurred: bool,
    sharp_pyramid: bool,
    blurred_pyramid: bool,
    publishes: bool,
}

const fn cleared_by(cause: Invalidation, had_blurred: bool) -> Cleared {
    match cause {
        // Everything hangs off the sharp texture; grain changes the source
        // every consumer reads, blurred or not.
        Invalidation::SharpDamage | Invalidation::GrainOptionsChanged => Cleared {
            grain: true,
            blurred: true,
            sharp_pyramid: true,
            blurred_pyramid: true,
            publishes: true,
        },
        // Blur options leave the sharp texture and its grain alone; there is
        // nothing to publish unless a blurred texture existed to go stale.
        Invalidation::BlurOptionsChanged => Cleared {
            grain: false,
            blurred: true,
            sharp_pyramid: false,
            blurred_pyramid: true,
            publishes: had_blurred,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrainStatus {
    /// Needs the pass (no grained texture, or a stale one); also where a
    /// failed compile or draw is retried.
    Dirty,
    /// `grain_texture` holds the sharp texture grained with the current options.
    Clean,
    /// The compile or the pass failed since the last invalidation that
    /// cleared grain; the sharp texture stands in until then.
    Failed,
}

/// The grain status after an invalidation: anything that clears grain
/// returns it to `Dirty`, which is also how a failure gets its retry.
const fn next_grain_status(status: GrainStatus, cleared: Cleared) -> GrainStatus {
    if cleared.grain {
        GrainStatus::Dirty
    } else {
        status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrefilterStatus {
    Empty,
    Dirty,
    Clean,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PrefilterSelection {
    low: usize,
    high: usize,
    mix: f32,
}

impl PrefilterSelection {
    const fn level_zero() -> Self {
        Self {
            low: 0,
            high: 0,
            mix: 0.,
        }
    }
}

#[derive(Debug)]
struct PrefilterState {
    level_sizes: Vec<Size<i32, Buffer>>,
    status: PrefilterStatus,
    highest_prepared_level: usize,
}

impl PrefilterState {
    fn new(size: Size<i32, Buffer>) -> Self {
        let mut level_sizes = Vec::new();
        let (mut w, mut h) = (size.w, size.h);
        while w > 1 || h > 1 {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            level_sizes.push(Size::from((w, h)));
        }

        let status = if level_sizes.is_empty() {
            PrefilterStatus::Empty
        } else {
            PrefilterStatus::Dirty
        };
        Self {
            level_sizes,
            status,
            highest_prepared_level: 0,
        }
    }

    fn level_sizes(&self) -> &[Size<i32, Buffer>] {
        &self.level_sizes
    }

    fn needs_prepare(&self) -> bool {
        self.status == PrefilterStatus::Dirty
            && self.highest_prepared_level < self.level_sizes.len()
    }

    fn invalidate(&mut self) {
        self.status = if self.level_sizes.is_empty() {
            PrefilterStatus::Empty
        } else {
            PrefilterStatus::Dirty
        };
        self.highest_prepared_level = 0;
    }

    fn mark_failed(&mut self) {
        self.status = PrefilterStatus::Failed;
        self.highest_prepared_level = 0;
    }

    fn mark_clean(&mut self) {
        self.status = if self.level_sizes.is_empty() {
            PrefilterStatus::Empty
        } else {
            PrefilterStatus::Clean
        };
        self.highest_prepared_level = self.level_sizes.len();
    }

    fn selection(&self, roughness: f64, ior: f64) -> PrefilterSelection {
        let max = self.level_sizes.len();
        if max == 0 || roughness == 0. {
            return PrefilterSelection::level_zero();
        }

        let lod = max as f64 * roughness * (ior * 2. - 2.).clamp(0., 1.);
        let low = lod.floor() as usize;
        let high = (low + 1).min(max);
        PrefilterSelection {
            low,
            high,
            mix: (lod - low as f64) as f32,
        }
    }
}

#[derive(Debug)]
pub struct EffectBuffer {
    /// Id to be used for this effect buffer's elements.
    id: Id,

    /// Size of the effect buffer.
    size: Size<i32, Buffer>,
    /// Scale of the effect buffer.
    scale: Scale<f64>,
    /// Options for blurring.
    blur_options: BlurOptions,
    grain: Option<GrainOptions>,
    grain_program: Option<GrainProgram>,

    /// Elements to be rendered on demand.
    elements: Elements,
    /// Offscreen buffer where elements get rendered.
    offscreen: Option<Offscreen>,
    /// Blurring program, if available.
    blur: Option<Blur>,

    /// Commit counter that takes into account both original and blurred texture changes.
    commit_counter: CommitCounter,
}

#[derive(Debug)]
enum Elements {
    /// Contents remain unchanged.
    Unchanged(
        // Storage to avoid reallocating it every time.
        Vec<OutputRenderElements<GlesRenderer>>,
    ),
    /// New contents, need to check damage and render.
    New(Vec<OutputRenderElements<GlesRenderer>>),
}

#[derive(Debug)]
struct Offscreen {
    /// The texture with the offscreen contents.
    texture: GlesTexture,
    grain_texture: Option<GlesTexture>,
    grain: GrainStatus,
    /// Id of the renderer context that the texture comes from.
    renderer_context_id: ContextId<GlesTexture>,
    /// Scale of the texture.
    scale: Scale<f64>,
    /// Damage tracker for drawing to the texture.
    damage: OutputDamageTracker,
    /// Render element states from the last render into the offscreen.
    states: RenderElementStates,
    /// Rendered blurred version of the texture.
    ///
    /// When texture needs to be reblurred, this field must be reset to `None`.
    blurred: Option<GlesTexture>,
    sharp_prefilter: PrefilterState,
    sharp_prefilter_textures: Vec<GlesTexture>,
    blurred_prefilter: PrefilterState,
    blurred_prefilter_textures: Vec<GlesTexture>,
}

impl Offscreen {
    /// What every consumer reads: the grained texture while grain is set and
    /// prepared, the sharp texture otherwise.
    fn source(&self, grain: Option<GrainOptions>) -> &GlesTexture {
        match (grain, self.grain, &self.grain_texture) {
            (Some(_), GrainStatus::Clean, Some(texture)) => texture,
            _ => &self.texture,
        }
    }

    fn clear(&mut self, cleared: Cleared) {
        self.grain = next_grain_status(self.grain, cleared);
        if cleared.blurred {
            self.blurred = None;
        }
        if cleared.sharp_pyramid {
            self.sharp_prefilter.invalidate();
        }
        if cleared.blurred_pyramid {
            self.blurred_prefilter.invalidate();
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrefilteredTexture {
    pub low: GlesTexture,
    pub high: GlesTexture,
    pub mix: f32,
}

fn prepare_prefilter(
    renderer: &mut GlesRenderer,
    program: &BlurProgram,
    source: &GlesTexture,
    state: &PrefilterState,
    textures: &mut Vec<GlesTexture>,
) -> anyhow::Result<()> {
    let _span = tracy_client::span!("EffectBuffer::prepare_prefilter");

    let reusable = textures.len() == state.level_sizes().len()
        && textures
            .iter()
            .zip(state.level_sizes())
            .all(|(texture, size)| texture.size() == *size)
        && textures.iter_mut().all(GlesTexture::is_unique_reference);
    if !reusable {
        textures.clear();
    }

    for &size in &state.level_sizes()[textures.len()..] {
        trace!(
            "creating material prefilter texture: {} x {}",
            size.w,
            size.h
        );
        textures.push(
            renderer
                .create_buffer(Fourcc::Abgr8888, size)
                .context("error creating prefilter texture")?,
        );
    }

    // ponytail: regenerate the full pyramid after any source damage; generate
    // only through the highest requested level if profiling shows this dominates
    // changed frames, then add per-level damage regions if that still falls short.
    program
        .render_downsample(renderer, source, textures, 1.)
        .context("error downsampling material prefilter")
}

fn prefilter_level(
    source: &GlesTexture,
    textures: &[GlesTexture],
    level: usize,
) -> Option<GlesTexture> {
    if level == 0 {
        Some(source.clone())
    } else {
        textures.get(level - 1).cloned()
    }
}

impl Default for Elements {
    fn default() -> Self {
        Self::Unchanged(Vec::new())
    }
}

impl EffectBuffer {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            size: Size::default(),
            scale: Scale::from(1.),
            blur_options: BlurOptions::default(),
            grain: None,
            grain_program: None,
            elements: Elements::default(),
            offscreen: None,
            blur: None,
            commit_counter: CommitCounter::default(),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn commit(&self) -> CommitCounter {
        self.commit_counter
    }

    pub fn logical_size(&self) -> Size<f64, Logical> {
        self.size.to_f64().to_logical(self.scale, Transform::Normal)
    }

    pub fn scale(&self) -> Scale<f64> {
        self.scale
    }

    pub fn render_element_states(&self) -> Option<&RenderElementStates> {
        self.offscreen.as_ref().map(|o| &o.states)
    }

    pub fn update_size(&mut self, size: Size<i32, Physical>, scale: Scale<f64>) {
        self.size = size.to_logical(1).to_buffer(1, Transform::Normal);
        self.scale = scale;
    }

    fn invalidate(&mut self, cause: Invalidation) {
        let Some(offscreen) = &mut self.offscreen else {
            return;
        };
        let cleared = cleared_by(cause, offscreen.blurred.is_some());
        offscreen.clear(cleared);
        if cleared.publishes {
            self.commit_counter.increment();
        }
    }

    pub fn update_grain_options(&mut self, options: Option<GrainOptions>) {
        if self.grain == options {
            return;
        }
        self.grain = options;
        self.invalidate(Invalidation::GrainOptionsChanged);
    }

    pub fn update_blur_options(&mut self, options: BlurOptions) {
        if self.blur_options == options {
            return;
        }
        self.blur_options = options;
        self.invalidate(Invalidation::BlurOptionsChanged);
    }

    pub fn elements(&mut self) -> &mut Vec<OutputRenderElements<GlesRenderer>> {
        // Assume we're going to insert new elements, switch to New.
        match mem::take(&mut self.elements) {
            Elements::Unchanged(elements) | Elements::New(elements) => {
                self.elements = Elements::New(elements);
            }
        }
        let Elements::New(elements) = &mut self.elements else {
            unreachable!();
        };
        elements
    }

    pub fn prepare(&mut self, renderer: &mut GlesRenderer, blur: bool) -> bool {
        if let Err(err) = self.prepare_offscreen(renderer) {
            warn!("error preparing offscreen: {err:?}");
            return false;
        };

        self.prepare_grain(renderer);

        if blur {
            if let Err(err) = self.prepare_blur(renderer) {
                warn!("error preparing blur: {err:?}");
                return false;
            }
        }

        true
    }

    fn prepare_offscreen(&mut self, renderer: &mut GlesRenderer) -> anyhow::Result<()> {
        let _span = tracy_client::span!("EffectBuffer::prepare_offscreen");

        // Check if we need to create or recreate the texture.
        let size_string;
        let mut reason = "";
        if let Some(Offscreen {
            texture,
            renderer_context_id,
            ..
        }) = &mut self.offscreen
        {
            let old_size = texture.size();
            if old_size != self.size {
                size_string = format!(
                    "size changed from {} × {} to {} × {}",
                    old_size.w, old_size.h, self.size.w, self.size.h
                );
                reason = &size_string;

                self.offscreen = None;
            } else if !texture.is_unique_reference() {
                reason = "not unique";

                self.offscreen = None;
            } else if *renderer_context_id != renderer.context_id() {
                reason = "renderer id changed";

                self.offscreen = None;
            }
        } else {
            reason = "first render";
        }

        let offscreen = if let Some(offscreen) = &mut self.offscreen {
            offscreen
        } else {
            trace!("creating new offscreen texture: {reason}");
            let span = tracy_client::span!("creating effect offscreen texture");
            span.emit_text(reason);

            let texture: GlesTexture = renderer
                .create_buffer(Fourcc::Abgr8888, self.size)
                .context("error creating texture")?;

            let buffer_size = self.size.to_logical(1, Transform::Normal).to_physical(1);
            let damage = OutputDamageTracker::new(buffer_size, self.scale, Transform::Normal);

            self.offscreen.insert(Offscreen {
                texture,
                grain_texture: None,
                grain: GrainStatus::Dirty,
                renderer_context_id: renderer.context_id(),
                scale: self.scale,
                damage,
                states: RenderElementStates::default(),
                blurred: None,
                sharp_prefilter: PrefilterState::new(self.size),
                sharp_prefilter_textures: Vec::new(),
                blurred_prefilter: PrefilterState::new(self.size),
                blurred_prefilter_textures: Vec::new(),
            })
        };

        // Recreate the damage tracker if the scale changes. We already recreate it for buffer size
        // changes, and transform is always Normal.
        if offscreen.scale != self.scale {
            offscreen.scale = self.scale;

            trace!("recreating damage tracker due to scale change");
            let buffer_size = self.size.to_logical(1, Transform::Normal).to_physical(1);
            offscreen.damage = OutputDamageTracker::new(buffer_size, self.scale, Transform::Normal);

            self.commit_counter.increment();
            offscreen.blurred = None;
            offscreen.grain = GrainStatus::Dirty;
            offscreen.sharp_prefilter.invalidate();
            offscreen.blurred_prefilter.invalidate();
        }

        // Render the elements if any.
        let mut elements = match mem::take(&mut self.elements) {
            Elements::New(elements) => elements,
            x @ Elements::Unchanged(_) => {
                // No redrawing necessary.
                self.elements = x;
                return Ok(());
            }
        };

        let res = {
            let mut target = renderer
                .bind(&mut offscreen.texture)
                .context("error binding texture")?;
            offscreen
                .damage
                .render_output(renderer, &mut target, 1, &elements, Color32F::TRANSPARENT)
                .context("error rendering")?
        };

        offscreen.states = res.states;

        if res.damage.is_some() {
            let _span = tracy_client::span!("EffectBuffer::sharp_damage");
            // Original texture changed; everything derived from it is stale.
            let cleared = cleared_by(Invalidation::SharpDamage, true);
            offscreen.clear(cleared);
            self.commit_counter.increment();
        }

        // Clear and put the storage back.
        elements.clear();
        self.elements = Elements::Unchanged(elements);

        Ok(())
    }

    /// Runs the grain pass when grain is set and the grained texture is
    /// stale. A failure is logged once per invalidation and the sharp
    /// texture stands in until the next one (the prefilter's pattern).
    fn prepare_grain(&mut self, renderer: &mut GlesRenderer) {
        let Some(options) = self.grain else {
            return;
        };
        let Some(offscreen) = self.offscreen.as_mut() else {
            return;
        };
        if offscreen.grain != GrainStatus::Dirty {
            return;
        }
        let _span = tracy_client::span!("EffectBuffer::prepare_grain");
        // The program is the buffer's, compiled on first need and after a
        // renderer change; a failed compile is retried the next time grain
        // is Dirty, which the invalidation table decides.
        if self
            .grain_program
            .as_ref()
            .is_some_and(|p| p.context_id() != renderer.context_id())
        {
            self.grain_program = None;
        }
        let program = match &self.grain_program {
            Some(program) => program.clone(),
            None => match GrainProgram::compile(renderer) {
                Ok(program) => self.grain_program.insert(program).clone(),
                Err(err) => {
                    offscreen.grain = GrainStatus::Failed;
                    warn!("backdrop grain shader failed to compile; the sharp texture stands in until the next damage: {err:?}");
                    return;
                }
            },
        };
        let result = (|| -> anyhow::Result<()> {
            let size = offscreen.texture.size();
            let reusable = offscreen
                .grain_texture
                .as_mut()
                .is_some_and(|t| t.size() == size && t.is_unique_reference());
            if !reusable {
                offscreen.grain_texture = Some(
                    renderer
                        .create_buffer(Fourcc::Abgr8888, size)
                        .context("error creating grain texture")?,
                );
            }
            let target = offscreen.grain_texture.as_ref().expect("just ensured");
            program.render(renderer, &offscreen.texture, target, options)
        })();
        match result {
            Ok(()) => offscreen.grain = GrainStatus::Clean,
            Err(err) => {
                offscreen.grain = GrainStatus::Failed;
                warn!("backdrop grain pass failed; the sharp texture stands in until the next damage: {err:?}");
            }
        }
    }

    fn prepare_blur(&mut self, renderer: &mut GlesRenderer) -> anyhow::Result<()> {
        let offscreen = self.offscreen.as_mut().context("missing offscreen")?;
        if offscreen.blurred.is_some() {
            // Already rendered.
            return Ok(());
        }

        if let Some(blur) = &self.blur {
            if blur.context_id() != renderer.context_id() {
                debug!("recreating blur: renderer changed");
                self.blur = None;
            }
        }

        let blur = if let Some(blur) = &mut self.blur {
            blur
        } else {
            let Some(blur) = Blur::new(renderer) else {
                // Missing blur shader.
                return Ok(());
            };
            self.blur.insert(blur)
        };

        ensure!(
            offscreen.renderer_context_id == renderer.context_id(),
            "wrong renderer context id"
        );

        blur.prepare_textures(
            |fourcc, size| renderer.create_buffer(fourcc, size),
            offscreen.source(self.grain),
            self.blur_options,
        )
        .context("error preparing blur textures")?;

        Ok(())
    }

    pub fn render(&mut self, frame: &mut GlesFrame, blur: bool) -> anyhow::Result<GlesTexture> {
        let offscreen = self.offscreen.as_mut().context("offscreen is missing")?;

        if !blur {
            return Ok(offscreen.source(self.grain).clone());
        }

        let texture = if let Some(texture) = &offscreen.blurred {
            texture.clone()
        } else {
            let blur = self.blur.as_mut().context("blur is missing")?;
            let mut guard = frame.renderer();
            let renderer = guard.as_mut();
            let blurred = blur
                .render(renderer, offscreen.source(self.grain), self.blur_options)
                .context("error rendering blur")?;
            offscreen.blurred_prefilter.invalidate();
            offscreen.blurred.insert(blurred).clone()
        };

        Ok(texture)
    }

    pub fn render_prefiltered(
        &mut self,
        frame: &mut GlesFrame,
        blur: bool,
        roughness: f64,
        ior: f64,
    ) -> Option<PrefilteredTexture> {
        let selection = {
            let offscreen = self.offscreen.as_ref()?;
            let state = if blur {
                &offscreen.blurred_prefilter
            } else {
                &offscreen.sharp_prefilter
            };
            state.selection(roughness, ior)
        };

        let needs_pyramid = selection.low > 0 || selection.mix > 0.;
        if needs_pyramid {
            let offscreen = self.offscreen.as_ref()?;
            let state = if blur {
                &offscreen.blurred_prefilter
            } else {
                &offscreen.sharp_prefilter
            };
            if state.status == PrefilterStatus::Failed {
                return None;
            }
        }

        let source = match self.render(frame, blur) {
            Ok(source) => source,
            Err(err) => {
                if needs_pyramid {
                    self.record_prefilter_failure(blur, &err);
                } else {
                    warn!("material source failed: {err:?}");
                }
                return None;
            }
        };

        if !needs_pyramid {
            return Some(PrefilteredTexture {
                low: source.clone(),
                high: source,
                mix: 0.,
            });
        }

        let program = Shaders::get_from_frame(frame).blur.clone();
        let mut guard = frame.renderer();
        let renderer = guard.as_mut();
        let offscreen = self.offscreen.as_mut()?;
        let (state, textures, label) = if blur {
            (
                &mut offscreen.blurred_prefilter,
                &mut offscreen.blurred_prefilter_textures,
                "blurred",
            )
        } else {
            (
                &mut offscreen.sharp_prefilter,
                &mut offscreen.sharp_prefilter_textures,
                "sharp",
            )
        };

        if state.needs_prepare() {
            let _span = if blur {
                tracy_client::span!("EffectBuffer::prepare_blurred_prefilter")
            } else {
                tracy_client::span!("EffectBuffer::prepare_sharp_prefilter")
            };
            let result = program
                .context("blur downsample program is missing")
                .and_then(|program| {
                    prepare_prefilter(renderer, &program, &source, state, textures)
                });
            if let Err(err) = result {
                state.mark_failed();
                warn!("{label} material prefilter failed: {err:?}");
                return None;
            }
            state.mark_clean();
        }

        let low = prefilter_level(&source, textures, selection.low)?;
        let high = if selection.mix == 0. {
            low.clone()
        } else {
            prefilter_level(&source, textures, selection.high)?
        };
        Some(PrefilteredTexture {
            low,
            high,
            mix: selection.mix,
        })
    }

    fn record_prefilter_failure(&mut self, blur: bool, err: &anyhow::Error) {
        let Some(offscreen) = self.offscreen.as_mut() else {
            return;
        };
        let (state, label) = if blur {
            (&mut offscreen.blurred_prefilter, "blurred")
        } else {
            (&mut offscreen.sharp_prefilter, "sharp")
        };
        if state.status != PrefilterStatus::Failed {
            state.mark_failed();
            warn!("{label} material prefilter failed: {err:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niri_config::NoiseType;
    #[test]
    fn each_invalidation_clears_what_depends_on_it_and_publishes_damage() {
        use Invalidation::*;
        let all = Cleared {
            grain: true,
            blurred: true,
            sharp_pyramid: true,
            blurred_pyramid: true,
            publishes: true,
        };
        assert_eq!(cleared_by(SharpDamage, false), all);
        assert_eq!(cleared_by(SharpDamage, true), all);
        // Grain changes the source every consumer reads, blurred or not.
        assert_eq!(cleared_by(GrainOptionsChanged, false), all);
        assert_eq!(cleared_by(GrainOptionsChanged, true), all);
        // Blur options leave the sharp texture and its grain alone, and
        // publish only when a blurred texture existed to be stale.
        let blur_only = Cleared {
            grain: false,
            blurred: true,
            sharp_pyramid: false,
            blurred_pyramid: true,
            publishes: false,
        };
        assert_eq!(cleared_by(BlurOptionsChanged, false), blur_only);
        assert_eq!(
            cleared_by(BlurOptionsChanged, true),
            Cleared {
                publishes: true,
                ..blur_only
            }
        );
    }

    #[test]
    fn a_failed_grain_pass_retries_only_after_an_invalidation_that_clears_grain() {
        use Invalidation::*;
        assert_eq!(
            next_grain_status(GrainStatus::Failed, cleared_by(BlurOptionsChanged, true)),
            GrainStatus::Failed
        );
        assert_eq!(
            next_grain_status(GrainStatus::Failed, cleared_by(SharpDamage, true)),
            GrainStatus::Dirty
        );
        assert_eq!(
            next_grain_status(GrainStatus::Failed, cleared_by(GrainOptionsChanged, false)),
            GrainStatus::Dirty
        );
        assert_eq!(
            next_grain_status(GrainStatus::Clean, cleared_by(BlurOptionsChanged, true)),
            GrainStatus::Clean
        );
        assert_eq!(
            next_grain_status(GrainStatus::Clean, cleared_by(SharpDamage, false)),
            GrainStatus::Dirty
        );
    }

    #[test]
    fn grain_options_change_publishes_only_with_an_offscreen_and_never_for_equal_options() {
        let mut buffer = EffectBuffer::new();
        let before = buffer.commit();
        let grain = Some(GrainOptions {
            amount: 0.3,
            kind: NoiseType::Fine,
        });
        buffer.update_grain_options(grain);
        assert_eq!(
            buffer.commit(),
            before,
            "no offscreen yet, nothing to publish"
        );
        buffer.update_grain_options(grain);
        assert_eq!(buffer.commit(), before, "equal options are a no-op");
        assert_eq!(buffer.grain, grain);
    }

    #[test]
    fn failed_grain_draw_uses_the_sharp_source_until_invalidation() {
        crate::render_helpers::grain::tests::with_renderer(|renderer| {
            let mut buffer = EffectBuffer::new();
            buffer.update_size((8, 8).into(), Scale::from(1.));
            buffer.elements();
            buffer.update_grain_options(Some(GrainOptions {
                amount: 0.3,
                kind: NoiseType::Fine,
            }));
            assert!(buffer.prepare(renderer, false));
            crate::render_helpers::grain::tests::delete_program(
                renderer,
                buffer.grain_program.as_ref().unwrap(),
            );
            buffer.update_grain_options(Some(GrainOptions {
                amount: 0.4,
                kind: NoiseType::Fine,
            }));
            assert!(buffer.prepare(renderer, false));
            let offscreen = buffer.offscreen.as_ref().unwrap();
            assert_eq!(offscreen.grain, GrainStatus::Failed);
            assert_eq!(
                offscreen.source(buffer.grain).tex_id(),
                offscreen.texture.tex_id()
            );
            assert!(buffer.prepare(renderer, false));
            assert_eq!(
                buffer.offscreen.as_ref().unwrap().grain,
                GrainStatus::Failed
            );
            buffer.update_grain_options(Some(GrainOptions {
                amount: 0.5,
                kind: NoiseType::Fine,
            }));
            assert_eq!(buffer.offscreen.as_ref().unwrap().grain, GrainStatus::Dirty);
        });
    }

    #[test]
    fn prefilter_levels_reach_one_by_one_and_stay_below_the_source() {
        for w in 1..=64 {
            for h in 1..=64 {
                let state = PrefilterState::new(Size::from((w, h)));
                let pixels: i32 = state.level_sizes().iter().map(|s| s.w * s.h).sum();
                assert!(pixels < w * h || w * h == 1, "{w} x {h}: {pixels}");
                if w * h == 1 {
                    assert!(state.level_sizes().is_empty());
                } else {
                    assert_eq!(state.level_sizes().last(), Some(&Size::from((1, 1))));
                }
            }
        }
    }

    #[test]
    fn prefilter_selection_matches_reference_formula() {
        let state = PrefilterState::new(Size::from((1920, 1080)));
        assert_eq!(state.selection(0., 1.5), PrefilterSelection::level_zero());
        let selected = state.selection(0.08, 1.5);
        assert_eq!((selected.low, selected.high), (0, 1));
        assert!((selected.mix - 0.8).abs() < f32::EPSILON);
        assert_eq!(
            state.selection(1., 1.),
            PrefilterSelection {
                low: 0,
                high: 1,
                mix: 0.,
            }
        );
    }

    #[test]
    fn prefilter_failure_retries_only_after_invalidation() {
        let mut state = PrefilterState::new(Size::from((1920, 1080)));
        assert!(state.needs_prepare());
        state.mark_failed();
        assert!(!state.needs_prepare());
        state.invalidate();
        assert!(state.needs_prepare());
        state.mark_clean();
        assert!(!state.needs_prepare());
    }
}
