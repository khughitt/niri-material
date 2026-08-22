use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

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

#[derive(Debug)]
pub struct MaterialState {
    pub offscreen: OffscreenBuffer,
    id: Id,
    commit: Cell<CommitCounter>,
}

impl MaterialState {
    pub fn new() -> Self {
        Self {
            offscreen: OffscreenBuffer::default(),
            id: Id::new(),
            commit: Cell::new(CommitCounter::default()),
        }
    }

    pub fn has_program(renderer: &mut GlesRenderer) -> bool {
        Shaders::get(renderer).material.is_some()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn element(
        &self,
        area: Rectangle<f64, Logical>,
        scale: f64,
        alpha: f32,
        time: Duration,
        win_rect: [f32; 4],
        win_src: Rectangle<f64, Buffer>,
        bg_rect: [f32; 4],
        win_texture: GlesTexture,
        bg: Rc<RefCell<EffectBuffer>>,
    ) -> MaterialRenderElement {
        let mut commit = self.commit.get();
        commit.increment();
        self.commit.set(commit);
        MaterialRenderElement {
            id: self.id.clone(),
            commit,
            area,
            scale,
            alpha,
            time,
            win_rect,
            win_src,
            bg_rect,
            win_texture,
            bg,
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
    time: Duration,
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

        let uniforms: Rc<[Uniform<'static>]> = Rc::new([
            Uniform::new("mat_time", self.time.as_secs_f32()),
            Uniform::new("mat_win_rect", self.win_rect),
            Uniform::new("mat_bg_rect", self.bg_rect),
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
    use smithay::utils::Point;

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
