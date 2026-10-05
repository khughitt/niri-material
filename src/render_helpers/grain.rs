//! The backdrop grain pass (`noise site=backdrop`, design
//! `2026-10-05-noise-placement-design.md` §4): one full-quad draw that grains
//! an output's sharp effect-buffer texture into a sibling texture, which the
//! blur, both prefilter pyramids and the direct sample then read.

use std::rc::Rc;

use anyhow::{ensure, Context as _};
use niri_config::{BackdropGrain, NoiseType};
use smithay::backend::renderer::gles::{ffi, link_program, GlesRenderer, GlesTexture};
use smithay::backend::renderer::{ContextId, Renderer as _, Texture as _};
use smithay::gpu_span_location;

use crate::render_helpers::shaders::grain_source;

/// The agreed backdrop grain, as the pass needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrainOptions {
    pub amount: f32,
    pub kind: NoiseType,
}

impl From<BackdropGrain> for GrainOptions {
    fn from(grain: BackdropGrain) -> Self {
        Self {
            amount: grain.amount as f32,
            kind: grain.kind,
        }
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
    attrib_vert: ffi::types::GLint,
    context_id: ContextId<GlesTexture>,
}

impl GrainProgram {
    pub fn compile(renderer: &mut GlesRenderer) -> anyhow::Result<Self> {
        let source = grain_source();
        let context_id = renderer.context_id();
        renderer
            .with_context(move |gl| unsafe {
                let program = link_program(gl, include_str!("shaders/blur.vert"), &source)
                    .context("error compiling grain shader")?;
                Ok(Self(Rc::new(GrainProgramInner {
                    program,
                    uniform_tex: gl.GetUniformLocation(program, c"tex".as_ptr()),
                    uniform_amount: gl.GetUniformLocation(program, c"mat_noise".as_ptr()),
                    uniform_kind: gl.GetUniformLocation(program, c"mat_noise_type".as_ptr()),
                    attrib_vert: gl.GetAttribLocation(program, c"vert".as_ptr()),
                    context_id,
                })))
            })
            .context("error making GL context current")?
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
            gl.BindFramebuffer(ffi::DRAW_FRAMEBUFFER, fbo);
            gl.FramebufferTexture2D(
                ffi::DRAW_FRAMEBUFFER,
                ffi::COLOR_ATTACHMENT0,
                ffi::TEXTURE_2D,
                target.tex_id(),
                0,
            );
            gl.Viewport(0, 0, size.w, size.h);

            gl.UseProgram(p.program);
            gl.Uniform1i(p.uniform_tex, 0);
            gl.Uniform1f(p.uniform_amount, options.amount);
            gl.Uniform1f(p.uniform_kind, options.kind as u8 as f32);

            let vertices: [f32; 12] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];
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

            gl.DisableVertexAttribArray(p.attrib_vert as u32);
            gl.BindFramebuffer(ffi::DRAW_FRAMEBUFFER, 0);
            gl.DeleteFramebuffers(1, &fbo);
        })?;
        Ok(())
    }
}
