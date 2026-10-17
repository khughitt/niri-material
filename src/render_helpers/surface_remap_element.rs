//! A layer surface drawn through the surface remap program
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §7). Geometry,
//! damage and opaque regions are the client's: the remap moves colors only.

use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::{Element, Id, Kind, RenderElement, UnderlyingStorage};
use smithay::backend::renderer::gles::{
    GlesError, GlesFrame, GlesRenderer, GlesTexProgram, Uniform,
};
use smithay::backend::renderer::utils::{CommitCounter, DamageSet, OpaqueRegions};
use smithay::utils::user_data::UserDataMap;
use smithay::utils::{Buffer, Physical, Rectangle, Scale, Transform};

use super::renderer::{AsGlesFrame as _, NiriRenderer};
use super::shaders::Shaders;
use super::surface_remap::RemapUniforms;
use crate::backend::tty::{TtyFrame, TtyRenderer, TtyRendererError};

#[derive(Debug)]
pub struct SurfaceRemapRenderElement<R: NiriRenderer> {
    inner: WaylandSurfaceRenderElement<R>,
    program: GlesTexProgram,
    uniforms: RemapUniforms,
}

impl<R: NiriRenderer> SurfaceRemapRenderElement<R> {
    pub fn new(
        inner: WaylandSurfaceRenderElement<R>,
        program: GlesTexProgram,
        uniforms: RemapUniforms,
    ) -> Self {
        Self {
            inner,
            program,
            uniforms,
        }
    }

    pub fn shader(renderer: &mut R) -> Option<&GlesTexProgram> {
        Shaders::get(renderer).surface_remap.as_ref()
    }

    fn compute_uniforms(&self) -> Vec<Uniform<'static>> {
        let u = &self.uniforms;
        vec![
            Uniform::new("remap_anchor0", u.anchors[0]),
            Uniform::new("remap_anchor1", u.anchors[1]),
            Uniform::new("remap_anchor2", u.anchors[2]),
            Uniform::new("remap_offset0", u.offsets[0]),
            Uniform::new("remap_offset1", u.offsets[1]),
            Uniform::new("remap_offset2", u.offsets[2]),
            Uniform::new("remap_radii", u.radii),
        ]
    }
}

impl<R: NiriRenderer> Element for SurfaceRemapRenderElement<R> {
    fn id(&self) -> &Id {
        self.inner.id()
    }

    fn current_commit(&self) -> CommitCounter {
        self.inner.current_commit()
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.inner.geometry(scale)
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        self.inner.src()
    }

    fn transform(&self) -> Transform {
        self.inner.transform()
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        // Hue and target changes are damaged by MappedLayer's ExtraDamage.
        self.inner.damage_since(scale, commit)
    }

    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.inner.opaque_regions(scale)
    }

    fn alpha(&self) -> f32 {
        self.inner.alpha()
    }

    fn kind(&self) -> Kind {
        self.inner.kind()
    }
}

impl RenderElement<GlesRenderer> for SurfaceRemapRenderElement<GlesRenderer> {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), GlesError> {
        frame.override_default_tex_program(self.program.clone(), self.compute_uniforms());
        RenderElement::<GlesRenderer>::draw(
            &self.inner,
            frame,
            src,
            dst,
            damage,
            opaque_regions,
            cache,
        )?;
        frame.clear_tex_program_override();
        Ok(())
    }

    fn underlying_storage(&self, _renderer: &mut GlesRenderer) -> Option<UnderlyingStorage<'_>> {
        // The remap must run, so the surface is never scanned out directly.
        None
    }
}

impl<'render> RenderElement<TtyRenderer<'render>>
    for SurfaceRemapRenderElement<TtyRenderer<'render>>
{
    fn draw(
        &self,
        frame: &mut TtyFrame<'render, '_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        cache: Option<&UserDataMap>,
    ) -> Result<(), TtyRendererError<'render>> {
        frame
            .as_gles_frame()
            .override_default_tex_program(self.program.clone(), self.compute_uniforms());
        RenderElement::draw(&self.inner, frame, src, dst, damage, opaque_regions, cache)?;
        frame.as_gles_frame().clear_tex_program_override();
        Ok(())
    }

    fn underlying_storage(
        &self,
        _renderer: &mut TtyRenderer<'render>,
    ) -> Option<UnderlyingStorage<'_>> {
        None
    }
}
