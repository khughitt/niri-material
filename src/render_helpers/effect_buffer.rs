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
use crate::render_helpers::blur::{Blur, BlurOptions};

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

    pub fn update_blur_options(&mut self, options: BlurOptions) {
        if self.blur_options == options {
            return;
        }

        self.blur_options = options;

        if let Some(offscreen) = &mut self.offscreen {
            if offscreen.blurred.is_some() {
                offscreen.blurred = None;
                self.commit_counter.increment();
            }
        }
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
                renderer_context_id: renderer.context_id(),
                scale: self.scale,
                damage,
                states: RenderElementStates::default(),
                blurred: None,
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
            self.commit_counter.increment();

            // Original texture changed; reset the blurred texture.
            offscreen.blurred = None;
        }

        // Clear and put the storage back.
        elements.clear();
        self.elements = Elements::Unchanged(elements);

        Ok(())
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
            &offscreen.texture,
            self.blur_options,
        )
        .context("error preparing blur textures")?;

        Ok(())
    }

    pub fn render(&mut self, frame: &mut GlesFrame, blur: bool) -> anyhow::Result<GlesTexture> {
        let offscreen = self.offscreen.as_mut().context("offscreen is missing")?;

        if !blur {
            return Ok(offscreen.texture.clone());
        }

        let texture = if let Some(texture) = &offscreen.blurred {
            texture.clone()
        } else {
            let blur = self.blur.as_mut().context("blur is missing")?;
            let mut guard = frame.renderer();
            let renderer = guard.as_mut();
            let blurred = blur
                .render(renderer, &offscreen.texture, self.blur_options)
                .context("error rendering blur")?;
            offscreen.blurred.insert(blurred).clone()
        };

        Ok(texture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
