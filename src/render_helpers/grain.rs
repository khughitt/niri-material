//! The backdrop grain pass (`noise site=backdrop`, design
//! `2026-10-05-noise-placement-design.md` §4): one full-quad draw that grains
//! an output's sharp effect-buffer texture into a sibling texture, which the
//! blur, both prefilter pyramids and the direct sample then read.

use std::rc::Rc;

use anyhow::{ensure, Context as _};
use niri_config::{BackdropGrain, NoiseType, NOISE_LAYERS};
use smithay::backend::renderer::gles::{ffi, link_program, GlesRenderer, GlesTexture};
use smithay::backend::renderer::{ContextId, Renderer as _, Texture as _};
use smithay::gpu_span_location;

use crate::render_helpers::shaders::grain_source;

/// One backdrop layer, as the pass needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrainLayer {
    pub amount: f32,
    pub kind: NoiseType,
    pub scale: f32,
}

/// The agreed backdrop layers, in backdrop-list order (slot `k` seeds with
/// `NOISE_SEED_k`). Equality over the whole array is the effect buffer's
/// change detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrainOptions {
    pub layers: [Option<GrainLayer>; NOISE_LAYERS],
}

impl From<BackdropGrain> for GrainOptions {
    fn from(grain: BackdropGrain) -> Self {
        Self {
            layers: grain.layers.map(|layer| {
                layer.map(|layer| GrainLayer {
                    amount: layer.amount as f32,
                    kind: layer.kind,
                    scale: layer.scale as f32,
                })
            }),
        }
    }
}

impl GrainOptions {
    /// Amounts, kinds and scales, one vec4 component per slot; an empty
    /// slot is amount 0 and scale 1, which `noise_source` skips.
    pub fn uniforms(&self) -> [[f32; 4]; 3] {
        let mut out = [[0.; 4], [0.; 4], [1.; 4]];
        for (k, layer) in self.layers.iter().enumerate() {
            if let Some(layer) = layer {
                out[0][k] = layer.amount;
                out[1][k] = layer.kind as u8 as f32;
                out[2][k] = layer.scale;
            }
        }
        out
    }

    /// One layer at scale 1, for tests.
    #[cfg(test)]
    pub(crate) fn one(amount: f32, kind: NoiseType) -> Self {
        let mut layers = [None; NOISE_LAYERS];
        layers[0] = Some(GrainLayer {
            amount,
            kind,
            scale: 1.,
        });
        Self { layers }
    }
}

#[derive(Debug, Clone)]
pub struct GrainProgram(Rc<GrainProgramInner>);

#[derive(Debug)]
struct GrainProgramInner {
    program: ffi::types::GLuint,
    uniform_tex: ffi::types::GLint,
    uniform_amount: ffi::types::GLint,
    uniform_kind: ffi::types::GLint,
    uniform_scale: ffi::types::GLint,
    attrib_vert: ffi::types::GLint,
    context_id: ContextId<GlesTexture>,
}

impl GrainProgram {
    pub fn compile(renderer: &mut GlesRenderer) -> anyhow::Result<Self> {
        // The EGL share group owns one successful program. Buffers borrow
        // it, so reconnecting outputs cannot orphan per-buffer GL objects.
        // EGL frees the program with the owning context/share group; failed
        // compiles never enter this cache and remain retryable.
        if let Some(program) = renderer.egl_context().user_data().get::<Self>() {
            return Ok(program.clone());
        }
        let source = grain_source();
        let context_id = renderer.context_id();
        let program = renderer
            .with_context(move |gl| unsafe {
                let program = link_program(gl, include_str!("shaders/blur.vert"), &source)
                    .context("error compiling grain shader")?;
                Ok::<_, anyhow::Error>(Self(Rc::new(GrainProgramInner {
                    program,
                    uniform_tex: gl.GetUniformLocation(program, c"tex".as_ptr()),
                    uniform_amount: gl.GetUniformLocation(program, c"mat_noise".as_ptr()),
                    uniform_kind: gl.GetUniformLocation(program, c"mat_noise_type".as_ptr()),
                    uniform_scale: gl.GetUniformLocation(program, c"mat_noise_scale".as_ptr()),
                    attrib_vert: gl.GetAttribLocation(program, c"vert".as_ptr()),
                    context_id,
                })))
            })
            .context("error making GL context current")??;
        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| program.clone());
        Ok(program)
    }

    /// The renderer context the program was linked in; a buffer drops the
    /// program when its renderer changes, as `Blur` does.
    pub fn context_id(&self) -> ContextId<GlesTexture> {
        self.0.context_id.clone()
    }

    /// Grains `source` into `target`, same size, one texel to one texel.
    pub fn render(
        &self,
        renderer: &mut GlesRenderer,
        source: &GlesTexture,
        target: &GlesTexture,
        options: GrainOptions,
    ) -> anyhow::Result<()> {
        let _span = tracy_client::span!("Grain::render");
        let size = target.size();
        ensure!(
            source.size() == size,
            "grain target size {size:?} differs from the source {:?}",
            source.size()
        );
        let p = &self.0;
        renderer.with_profiled_context(gpu_span_location!("Grain::render"), |gl| unsafe {
            while gl.GetError() != ffi::NO_ERROR {}
            gl.Disable(ffi::BLEND);
            gl.Disable(ffi::SCISSOR_TEST);
            gl.ActiveTexture(ffi::TEXTURE0);

            let mut fbo = 0;
            gl.GenFramebuffers(1, &mut fbo);
            let result = (|| -> anyhow::Result<()> {
                gl.BindFramebuffer(ffi::DRAW_FRAMEBUFFER, fbo);
                gl.FramebufferTexture2D(
                    ffi::DRAW_FRAMEBUFFER,
                    ffi::COLOR_ATTACHMENT0,
                    ffi::TEXTURE_2D,
                    target.tex_id(),
                    0,
                );
                let status = gl.CheckFramebufferStatus(ffi::DRAW_FRAMEBUFFER);
                ensure!(
                    status == ffi::FRAMEBUFFER_COMPLETE,
                    "grain framebuffer incomplete: {status:#x}"
                );
                gl.Viewport(0, 0, size.w, size.h);

                gl.UseProgram(p.program);
                gl.Uniform1i(p.uniform_tex, 0);
                let [amount, kind, scale] = options.uniforms();
                gl.Uniform4f(p.uniform_amount, amount[0], amount[1], amount[2], amount[3]);
                gl.Uniform4f(p.uniform_kind, kind[0], kind[1], kind[2], kind[3]);
                gl.Uniform4f(p.uniform_scale, scale[0], scale[1], scale[2], scale[3]);

                let vertices: [f32; 12] =
                    [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];
                gl.EnableVertexAttribArray(p.attrib_vert as u32);
                gl.BindBuffer(ffi::ARRAY_BUFFER, 0);
                gl.VertexAttribPointer(
                    p.attrib_vert as u32,
                    2,
                    ffi::FLOAT,
                    ffi::FALSE,
                    0,
                    vertices.as_ptr().cast(),
                );

                gl.BindTexture(ffi::TEXTURE_2D, source.tex_id());
                // One texel in, one texel out: no filtering.
                gl.TexParameteri(
                    ffi::TEXTURE_2D,
                    ffi::TEXTURE_MIN_FILTER,
                    ffi::NEAREST as i32,
                );
                gl.TexParameteri(
                    ffi::TEXTURE_2D,
                    ffi::TEXTURE_MAG_FILTER,
                    ffi::NEAREST as i32,
                );
                gl.TexParameteri(
                    ffi::TEXTURE_2D,
                    ffi::TEXTURE_WRAP_S,
                    ffi::CLAMP_TO_EDGE as i32,
                );
                gl.TexParameteri(
                    ffi::TEXTURE_2D,
                    ffi::TEXTURE_WRAP_T,
                    ffi::CLAMP_TO_EDGE as i32,
                );

                gl.DrawArrays(ffi::TRIANGLES, 0, 6);
                let error = gl.GetError();
                ensure!(
                    error == ffi::NO_ERROR,
                    "grain draw failed: GL error {error:#x}"
                );
                Ok(())
            })();
            gl.DisableVertexAttribArray(p.attrib_vert as u32);
            gl.BindFramebuffer(ffi::DRAW_FRAMEBUFFER, 0);
            gl.DeleteFramebuffers(1, &fbo);
            gl.UseProgram(0);
            let cleanup_error = gl.GetError();
            // Consume every error from a failed pass before the normal frame
            // renders its sharp-source fallback in this same GL context.
            while gl.GetError() != ffi::NO_ERROR {}
            if result.is_ok() {
                ensure!(
                    cleanup_error == ffi::NO_ERROR,
                    "grain cleanup failed: GL error {cleanup_error:#x}"
                );
            }
            result
        })??;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use smithay::backend::allocator::Fourcc;
    use smithay::backend::egl::native::EGLSurfacelessDisplay;
    use smithay::backend::egl::{EGLContext, EGLDisplay};
    use smithay::backend::renderer::Offscreen as _;

    use super::*;

    pub(crate) fn with_renderer<T>(test: impl FnOnce(&mut GlesRenderer) -> T) -> T {
        let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
        let context = EGLContext::new(&display).unwrap();
        let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
        test(&mut renderer)
    }

    pub(crate) fn delete_program(renderer: &mut GlesRenderer, program: &GrainProgram) {
        renderer
            .with_context(|gl| unsafe {
                gl.UseProgram(0);
                gl.DeleteProgram(program.0.program);
            })
            .unwrap();
    }

    #[test]
    fn grain_rejects_an_incomplete_framebuffer_and_releases_it() {
        with_renderer(|renderer| {
            let program = GrainProgram::compile(renderer).unwrap();
            let source: GlesTexture = renderer
                .create_buffer(Fourcc::Abgr8888, (8, 8).into())
                .unwrap();
            let target: GlesTexture = renderer
                .create_buffer(Fourcc::Abgr8888, (8, 8).into())
                .unwrap();
            renderer
                .with_context(|gl| unsafe {
                    gl.DeleteTextures(1, &target.tex_id());
                })
                .unwrap();
            let result = program.render(
                renderer,
                &source,
                &target,
                GrainOptions::one(0.3, NoiseType::Fine),
            );
            assert!(
                result.is_err(),
                "an invalid attachment must not be cached as clean"
            );
            renderer
                .with_context(|gl| unsafe {
                    let mut bound = -1;
                    gl.GetIntegerv(ffi::DRAW_FRAMEBUFFER_BINDING, &mut bound);
                    assert_eq!(bound, 0, "failure releases its framebuffer");
                    assert_eq!(
                        gl.GetError(),
                        ffi::NO_ERROR,
                        "failure consumes its GL error before the fallback frame"
                    );
                })
                .unwrap();
        });
    }

    #[test]
    fn grain_propagates_a_draw_error() {
        with_renderer(|renderer| {
            let program = GrainProgram::compile(renderer).unwrap();
            let source: GlesTexture = renderer
                .create_buffer(Fourcc::Abgr8888, (8, 8).into())
                .unwrap();
            let target: GlesTexture = renderer
                .create_buffer(Fourcc::Abgr8888, (8, 8).into())
                .unwrap();
            delete_program(renderer, &program);
            assert!(
                program
                    .render(
                        renderer,
                        &source,
                        &target,
                        GrainOptions::one(0.3, NoiseType::Fine)
                    )
                    .is_err(),
                "a failed GL draw must reach the sharp-source fallback"
            );
        });
    }

    #[test]
    fn grain_program_is_owned_by_its_context_after_buffer_references_drop() {
        with_renderer(|renderer| {
            let original = GrainProgram::compile(renderer).unwrap();
            let id = original.0.program;
            let context_id = original.context_id();
            drop(original);
            let again = GrainProgram::compile(renderer).unwrap();
            assert_eq!(
                again.0.program, id,
                "output reconnection reuses its context's program"
            );
            let context =
                EGLContext::new_shared(renderer.egl_context().display(), renderer.egl_context())
                    .unwrap();
            let mut replacement = unsafe { GlesRenderer::new(context) }.unwrap();
            let shared = GrainProgram::compile(&mut replacement).unwrap();
            assert_eq!(shared.context_id(), context_id);
            assert_eq!(
                shared.0.program, id,
                "shared renderer replacement keeps the context owner"
            );
        });
        with_renderer(|renderer| {
            let program = GrainProgram::compile(renderer).unwrap();
            assert_eq!(
                program.context_id(),
                renderer.context_id(),
                "a fresh context owns its own program"
            );
        });
    }
    #[test]
    fn options_pack_backdrop_layers_in_order_with_neutral_empty_slots() {
        let grain = niri_config::BackdropGrain {
            layers: [
                Some(niri_config::BackdropLayer {
                    amount: 0.3,
                    kind: NoiseType::Fine,
                    scale: 1.,
                }),
                Some(niri_config::BackdropLayer {
                    amount: 0.1,
                    kind: NoiseType::White,
                    scale: 4.,
                }),
                None,
                None,
            ],
        };
        assert_eq!(
            GrainOptions::from(grain).uniforms(),
            [[0.3, 0.1, 0., 0.], [1., 0., 0., 0.], [1., 4., 1., 1.]]
        );
    }

    #[test]
    fn options_differ_when_any_slot_differs() {
        let mut a = GrainOptions::one(0.3, NoiseType::Fine);
        let b = a;
        assert_eq!(a, b);
        a.layers[1] = Some(GrainLayer {
            amount: 0.1,
            kind: NoiseType::White,
            scale: 4.,
        });
        assert_ne!(a, b);
    }
}
