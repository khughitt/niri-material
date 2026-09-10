use std::cell::RefCell;

use glam::Mat3;
use smithay::backend::renderer::gles::{
    GlesError, GlesFrame, GlesRenderer, GlesTexProgram, Uniform, UniformName, UniformType,
    UniformValue,
};

use super::renderer::NiriRenderer;
use super::shader_element::ShaderProgram;
use crate::render_helpers::blur::BlurProgram;
use crate::render_helpers::material::optics::{self, OPTICS};

/// The material fragment shader: prelude, each optic's GLSL in `OPTICS`
/// order, then main. A comment marker per part keeps compile-error line
/// numbers locatable by hand.
pub(crate) fn material_source() -> String {
    let mut source = String::from(include_str!("material/prelude.frag"));
    for entry in OPTICS {
        source.push_str(&format!("\n// ---- optic: {}\n", entry.name));
        source.push_str(entry.glsl);
    }
    source.push_str("\n// ---- main\n");
    source.push_str(include_str!("material/main.frag"));
    source
}

/// The material program's uniforms: the core list, then every optic's.
pub(crate) fn material_uniform_names() -> Vec<UniformName<'static>> {
    let mut names = vec![
        UniformName::new("mat_win_rect", UniformType::_4f),
        UniformName::new("mat_geo_rect", UniformType::_4f),
        UniformName::new("mat_slab_rect", UniformType::_4f),
        UniformName::new("mat_area_size", UniformType::_2f),
        UniformName::new("mat_chamfer", UniformType::_1f),
        UniformName::new("mat_corner_radius", UniformType::_4f),
        UniformName::new("mat_jelly_move", UniformType::_2f),
        UniformName::new("mat_jelly_resize", UniformType::_2f),
        UniformName::new("mat_jelly_activity", UniformType::_1f),
        UniformName::new("mat_jelly_time", UniformType::_1f),
        UniformName::new("mat_jelly_seed", UniformType::_3f),
        UniformName::new("mat_bg_rect", UniformType::_4f),
        UniformName::new("mat_backdrop_rect", UniformType::_4f),
        UniformName::new("mat_ws_rect", UniformType::_4f),
        UniformName::new("mat_ws_color", UniformType::_4f),
        UniformName::new("mat_backdrop_color", UniformType::_4f),
        UniformName::new("mat_bg_prefilter_mix", UniformType::_1f),
        UniformName::new("mat_backdrop_prefilter_mix", UniformType::_1f),
        UniformName::new("mat_ior", UniformType::_1f),
        UniformName::new("mat_thickness", UniformType::_1f),
        UniformName::new("mat_attenuation_color", UniformType::_4f),
        UniformName::new("mat_attenuation_distance", UniformType::_1f),
        UniformName::new("mat_chromatic_aberration", UniformType::_1f),
        UniformName::new("mat_distortion", UniformType::_1f),
        UniformName::new("mat_distortion_scale", UniformType::_1f),
        UniformName::new("mat_samples", UniformType::_1f),
        UniformName::new("mat_anisotropic_blur", UniformType::_1f),
        UniformName::new("mat_jelly_ripple", UniformType::_1f),
        UniformName::new("mat_sig_accent", UniformType::_4f),
        UniformName::new("mat_sig_level", UniformType::_1f),
        UniformName::new("mat_sig_breath", UniformType::_1f),
        UniformName::new("mat_sig_light", UniformType::_3f),
        UniformName::new("mat_sig_impulse_env", UniformType::_4f),
        UniformName::new("mat_sig_impulse_prog", UniformType::_4f),
        UniformName::new("mat_sig_impulse_rgb0", UniformType::_3f),
        UniformName::new("mat_sig_impulse_rgb1", UniformType::_3f),
        UniformName::new("mat_sig_impulse_rgb2", UniformType::_3f),
        UniformName::new("mat_sig_impulse_rgb3", UniformType::_3f),
        UniformName::new("mat_sig_impulse_resp", UniformType::_4i),
        UniformName::new("mat_sig_response", UniformType::_3i),
        UniformName::new("mat_sig_ring", UniformType::_2f),
        UniformName::new("mat_sig_focus", UniformType::_2f),
        UniformName::new("mat_sig_ring_color", UniformType::_3f),
        UniformName::new("mat_light_ior", UniformType::_1f),
    ];
    names.extend(optics::uniform_names());
    names
}

pub struct Shaders {
    pub border: Option<ShaderProgram>,
    pub shadow: Option<ShaderProgram>,
    pub clipped_surface: Option<GlesTexProgram>,
    pub postprocess_and_clip: Option<GlesTexProgram>,
    pub resize: Option<ShaderProgram>,
    pub gradient_fade: Option<GlesTexProgram>,
    pub blur: Option<BlurProgram>,
    pub material: Option<ShaderProgram>,
    pub custom_resize: RefCell<Option<ShaderProgram>>,
    pub custom_close: RefCell<Option<ShaderProgram>>,
    pub custom_open: RefCell<Option<ShaderProgram>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProgramType {
    Border,
    Shadow,
    Resize,
    Material,
    Close,
    Open,
}

impl Shaders {
    fn compile(renderer: &mut GlesRenderer) -> Self {
        let _span = tracy_client::span!("Shaders::compile");

        let border = ShaderProgram::compile(
            renderer,
            concat!(
                include_str!("border.frag"),
                include_str!("rounding_alpha.frag")
            ),
            &[
                UniformName::new("colorspace", UniformType::_1f),
                UniformName::new("hue_interpolation", UniformType::_1f),
                UniformName::new("color_from", UniformType::_4f),
                UniformName::new("color_to", UniformType::_4f),
                UniformName::new("grad_offset", UniformType::_2f),
                UniformName::new("grad_width", UniformType::_1f),
                UniformName::new("grad_vec", UniformType::_2f),
                UniformName::new("input_to_geo", UniformType::Matrix3x3),
                UniformName::new("geo_size", UniformType::_2f),
                UniformName::new("outer_radius", UniformType::_4f),
                UniformName::new("border_width", UniformType::_1f),
            ],
            &[],
        )
        .map_err(|err| {
            warn!("error compiling border shader: {err:?}");
        })
        .ok();

        let shadow = ShaderProgram::compile(
            renderer,
            concat!(
                include_str!("shadow.frag"),
                include_str!("rounding_alpha.frag")
            ),
            &[
                UniformName::new("shadow_color", UniformType::_4f),
                UniformName::new("sigma", UniformType::_1f),
                UniformName::new("input_to_geo", UniformType::Matrix3x3),
                UniformName::new("geo_size", UniformType::_2f),
                UniformName::new("corner_radius", UniformType::_4f),
                UniformName::new("window_input_to_geo", UniformType::Matrix3x3),
                UniformName::new("window_geo_size", UniformType::_2f),
                UniformName::new("window_corner_radius", UniformType::_4f),
            ],
            &[],
        )
        .map_err(|err| {
            warn!("error compiling shadow shader: {err:?}");
        })
        .ok();

        let clipped_surface = renderer
            .compile_custom_texture_shader(
                concat!(
                    include_str!("clipped_surface.frag"),
                    include_str!("rounding_alpha.frag"),
                    "\nvec4 postprocess(vec4 color) { return color; }",
                ),
                &[
                    UniformName::new("niri_scale", UniformType::_1f),
                    UniformName::new("geo_size", UniformType::_2f),
                    UniformName::new("corner_radius", UniformType::_4f),
                    UniformName::new("input_to_geo", UniformType::Matrix3x3),
                ],
            )
            .map_err(|err| {
                warn!("error compiling clipped surface shader: {err:?}");
            })
            .ok();

        let postprocess_and_clip = renderer
            .compile_custom_texture_shader(
                concat!(
                    include_str!("clipped_surface.frag"),
                    include_str!("rounding_alpha.frag"),
                    include_str!("postprocess.frag"),
                ),
                &[
                    UniformName::new("niri_scale", UniformType::_1f),
                    UniformName::new("geo_size", UniformType::_2f),
                    UniformName::new("corner_radius", UniformType::_4f),
                    UniformName::new("input_to_geo", UniformType::Matrix3x3),
                    UniformName::new("noise", UniformType::_1f),
                    UniformName::new("saturation", UniformType::_1f),
                    UniformName::new("bg_color", UniformType::_4f),
                ],
            )
            .map_err(|err| {
                warn!("error compiling postprocess_and_clip shader: {err:?}");
            })
            .ok();

        let resize = compile_resize_program(renderer, include_str!("resize.frag"))
            .map_err(|err| {
                warn!("error compiling resize shader: {err:?}");
            })
            .ok();

        let gradient_fade = renderer
            .compile_custom_texture_shader(
                include_str!("gradient_fade.frag"),
                &[UniformName::new("cutoff", UniformType::_2f)],
            )
            .map_err(|err| {
                warn!("error compiling gradient fade shader: {err:?}");
            })
            .ok();

        let blur = BlurProgram::compile(renderer)
            .map_err(|err| {
                warn!("error compiling blur shaders: {err:?}");
            })
            .ok();

        let material_source = material_source();
        let material_uniforms = material_uniform_names();
        let material = ShaderProgram::compile(
            renderer,
            &material_source,
            &material_uniforms,
            &[
                "niri_tex_win",
                "niri_tex_bg",
                "niri_tex_bg_high",
                "niri_tex_backdrop",
                "niri_tex_backdrop_high",
            ],
        )
        .map_err(|err| {
            warn!("error compiling material shader: {err:?}");
        })
        .ok();

        Self {
            border,
            shadow,
            clipped_surface,
            postprocess_and_clip,
            resize,
            gradient_fade,
            blur,
            material,
            custom_resize: RefCell::new(None),
            custom_close: RefCell::new(None),
            custom_open: RefCell::new(None),
        }
    }

    pub fn get_from_frame<'a>(frame: &'a mut GlesFrame<'_, '_>) -> &'a Self {
        let data = frame.egl_context().user_data();
        data.get()
            .expect("shaders::init() must be called when creating the renderer")
    }

    pub fn get(renderer: &mut impl NiriRenderer) -> &Self {
        let renderer = renderer.as_gles_renderer();
        let data = renderer.egl_context().user_data();
        data.get()
            .expect("shaders::init() must be called when creating the renderer")
    }

    pub fn replace_custom_resize_program(
        &self,
        program: Option<ShaderProgram>,
    ) -> Option<ShaderProgram> {
        self.custom_resize.replace(program)
    }

    pub fn replace_custom_close_program(
        &self,
        program: Option<ShaderProgram>,
    ) -> Option<ShaderProgram> {
        self.custom_close.replace(program)
    }

    pub fn replace_custom_open_program(
        &self,
        program: Option<ShaderProgram>,
    ) -> Option<ShaderProgram> {
        self.custom_open.replace(program)
    }

    pub fn program(&self, program: ProgramType) -> Option<ShaderProgram> {
        match program {
            ProgramType::Border => self.border.clone(),
            ProgramType::Shadow => self.shadow.clone(),
            ProgramType::Resize => self
                .custom_resize
                .borrow()
                .clone()
                .or_else(|| self.resize.clone()),
            ProgramType::Material => self.material.clone(),
            ProgramType::Close => self.custom_close.borrow().clone(),
            ProgramType::Open => self.custom_open.borrow().clone(),
        }
    }
}

pub fn init(renderer: &mut GlesRenderer) {
    let shaders = Shaders::compile(renderer);
    let data = renderer.egl_context().user_data();
    if !data.insert_if_missing(|| shaders) {
        error!("shaders were already compiled");
    }
}

fn compile_resize_program(
    renderer: &mut GlesRenderer,
    src: &str,
) -> Result<ShaderProgram, GlesError> {
    let mut program = include_str!("resize_prelude.frag").to_string();
    program.push_str(src);
    program.push_str(include_str!("resize_epilogue.frag"));
    program.push_str(include_str!("rounding_alpha.frag"));

    ShaderProgram::compile(
        renderer,
        &program,
        &[
            UniformName::new("niri_input_to_curr_geo", UniformType::Matrix3x3),
            UniformName::new("niri_curr_geo_to_prev_geo", UniformType::Matrix3x3),
            UniformName::new("niri_curr_geo_to_next_geo", UniformType::Matrix3x3),
            UniformName::new("niri_curr_geo_size", UniformType::_2f),
            UniformName::new("niri_geo_to_tex_prev", UniformType::Matrix3x3),
            UniformName::new("niri_geo_to_tex_next", UniformType::Matrix3x3),
            UniformName::new("niri_progress", UniformType::_1f),
            UniformName::new("niri_clamped_progress", UniformType::_1f),
            UniformName::new("niri_corner_radius", UniformType::_4f),
            UniformName::new("niri_clip_to_geometry", UniformType::_1f),
        ],
        &["niri_tex_prev", "niri_tex_next"],
    )
}

pub fn set_custom_resize_program(renderer: &mut GlesRenderer, src: Option<&str>) {
    let program = if let Some(src) = src {
        match compile_resize_program(renderer, src) {
            Ok(program) => Some(program),
            Err(err) => {
                warn!("error compiling custom resize shader: {err:?}");
                return;
            }
        }
    } else {
        None
    };

    if let Some(prev) = Shaders::get(renderer).replace_custom_resize_program(program) {
        if let Err(err) = prev.destroy(renderer) {
            warn!("error destroying previous custom resize shader: {err:?}");
        }
    }
}

fn compile_close_program(
    renderer: &mut GlesRenderer,
    src: &str,
) -> Result<ShaderProgram, GlesError> {
    let mut program = include_str!("close_prelude.frag").to_string();
    program.push_str(src);
    program.push_str(include_str!("close_epilogue.frag"));

    ShaderProgram::compile(
        renderer,
        &program,
        &[
            UniformName::new("niri_input_to_geo", UniformType::Matrix3x3),
            UniformName::new("niri_geo_size", UniformType::_2f),
            UniformName::new("niri_geo_to_tex", UniformType::Matrix3x3),
            UniformName::new("niri_progress", UniformType::_1f),
            UniformName::new("niri_clamped_progress", UniformType::_1f),
            UniformName::new("niri_random_seed", UniformType::_1f),
        ],
        &["niri_tex"],
    )
}

pub fn set_custom_close_program(renderer: &mut GlesRenderer, src: Option<&str>) {
    let program = if let Some(src) = src {
        match compile_close_program(renderer, src) {
            Ok(program) => Some(program),
            Err(err) => {
                warn!("error compiling custom close shader: {err:?}");
                return;
            }
        }
    } else {
        None
    };

    if let Some(prev) = Shaders::get(renderer).replace_custom_close_program(program) {
        if let Err(err) = prev.destroy(renderer) {
            warn!("error destroying previous custom close shader: {err:?}");
        }
    }
}

fn compile_open_program(
    renderer: &mut GlesRenderer,
    src: &str,
) -> Result<ShaderProgram, GlesError> {
    let mut program = include_str!("open_prelude.frag").to_string();
    program.push_str(src);
    program.push_str(include_str!("open_epilogue.frag"));

    ShaderProgram::compile(
        renderer,
        &program,
        &[
            UniformName::new("niri_input_to_geo", UniformType::Matrix3x3),
            UniformName::new("niri_geo_size", UniformType::_2f),
            UniformName::new("niri_geo_to_tex", UniformType::Matrix3x3),
            UniformName::new("niri_progress", UniformType::_1f),
            UniformName::new("niri_clamped_progress", UniformType::_1f),
            UniformName::new("niri_random_seed", UniformType::_1f),
        ],
        &["niri_tex"],
    )
}

pub fn set_custom_open_program(renderer: &mut GlesRenderer, src: Option<&str>) {
    let program = if let Some(src) = src {
        match compile_open_program(renderer, src) {
            Ok(program) => Some(program),
            Err(err) => {
                warn!("error compiling custom open shader: {err:?}");
                return;
            }
        }
    } else {
        None
    };

    if let Some(prev) = Shaders::get(renderer).replace_custom_open_program(program) {
        if let Err(err) = prev.destroy(renderer) {
            warn!("error destroying previous custom open shader: {err:?}");
        }
    }
}

pub fn mat3_uniform(name: &str, mat: Mat3) -> Uniform<'_> {
    Uniform::new(
        name,
        UniformValue::Matrix3x3 {
            matrices: vec![mat.to_cols_array()],
            transpose: false,
        },
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::render_helpers::material::optics::OPTICS;

    #[test]
    fn optic_order_matches_the_config_crate() {
        let renderer: Vec<&str> = OPTICS.iter().map(|entry| entry.name).collect();
        assert_eq!(renderer, niri_config::material::optics::ORDER);
    }

    #[test]
    fn every_optic_declares_its_uniforms_in_its_glsl() {
        for entry in OPTICS {
            for (name, _) in entry.uniforms {
                assert!(
                    entry.glsl.contains(&format!(" {name};")),
                    "{}: {name} is not declared in its GLSL",
                    entry.name
                );
            }
            assert!(
                entry.glsl.contains(&format!("{}_", entry.name)),
                "{}: no hook function",
                entry.name
            );
        }
    }

    #[test]
    fn material_uniform_names_are_unique() {
        let names = material_uniform_names();
        let unique: HashSet<_> = names.iter().map(|n| n.name.clone()).collect();
        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn material_source_is_prelude_then_optics_in_order_then_main() {
        let source = material_source();
        let mut last = 0;
        for marker in [
            "// ---- optic: saturation",
            "// ---- optic: noise",
            "// ---- main",
        ] {
            let at = source
                .find(marker)
                .unwrap_or_else(|| panic!("{marker} missing"));
            assert!(at > last, "{marker} out of order");
            last = at;
        }
        assert_eq!(source.matches("void main()").count(), 1);
        assert!(!source.starts_with("#version"));
        assert!(source.contains("saturation_post(glassColor"));
        assert!(source.contains("noise_post(glassColor"));
    }
}
