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

/// The material fragment shader: the precision line, the shared helpers,
/// the prelude, each optic's GLSL in `OPTICS` order, then main. A comment
/// marker per part keeps compile-error line numbers locatable by hand.
pub(crate) fn material_source() -> String {
    let mut source = String::from("precision highp float;\n// ---- common\n");
    source.push_str(include_str!("material/common.frag"));
    source.push_str("\n// ---- prelude\n");
    source.push_str(include_str!("material/prelude.frag"));
    for entry in OPTICS {
        source.push_str(&format!("\n// ---- optic: {}\n", entry.name));
        source.push_str(entry.glsl);
    }
    source.push_str("\n// ---- main\n");
    source.push_str(include_str!("material/main.frag"));
    source
}

/// The effect-program grain pass (`noise site=backdrop`): a complete
/// `#version 100` fragment program over `blur.vert`'s `v_coords`, built from
/// the shared helpers, the noise optic's GLSL and `grain.frag`'s main.
pub(crate) fn grain_source() -> String {
    let noise = OPTICS
        .iter()
        .find(|entry| entry.name == "noise")
        .expect("the noise optic is registered");
    let mut source = String::from(
        "#version 100\nprecision highp float;\nvarying vec2 v_coords;\nuniform sampler2D tex;\n",
    );
    source.push_str("\n// ---- common\n");
    source.push_str(include_str!("material/common.frag"));
    source.push_str("\n// ---- optic: noise\n");
    source.push_str(noise.glsl);
    source.push_str("\n// ---- main\n");
    source.push_str(include_str!("grain.frag"));
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
        UniformName::new("mat_scatter", UniformType::_1f),
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
        UniformName::new("mat_sig_ring", UniformType::_4f),
        UniformName::new("mat_sig_focus", UniformType::_4f),
        UniformName::new("mat_sig_ring_color", UniformType::_3f),
        UniformName::new("mat_sig_ring_accent", UniformType::_1f),
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
            "// ---- optic: aurora",
            "// ---- optic: iridescence",
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
        assert!(source.contains("iridescence_specular(specular"));
        assert!(source.contains("uniform float mat_scatter;"));
        assert!(source.contains("aurora_within(p, n, att, innerDist)"));
        assert!(source.contains("filamentBand(q0, gap, width, mat_scatter)"));
        assert!(source.contains("uniform vec4 mat_sig_focus;"));
        assert!(source.contains("uniform vec4 mat_sig_ring;"));
        assert!(source.contains("uniform float mat_sig_ring_accent;"));
        assert!(material_uniform_names()
            .iter()
            .any(|name| { name.name == "mat_scatter" && name.type_ == UniformType::_1f }));
        let main = source.split_once("void main()").unwrap().1;
        let averaged = main.find("sampled = acc / count;").unwrap();
        let saturation = main.find("sampled = saturation_behind(").unwrap();
        let noise = main.find("sampled = noise_behind(").unwrap();
        let attenuation = main.find("vec3 transmitted = sampled * att;").unwrap();
        let within = main.find("vec3 within = vec3(0.0);").unwrap();
        let specular = main.find("vec3 specular =").unwrap();
        assert!(averaged < saturation && saturation < noise && noise < attenuation);
        assert!(attenuation < within && within < specular);
        assert!(main.contains("float depth = mat_thickness * 0.2;"));
        assert!(!main.contains("float mask"));
        assert!(!main.contains("* mask"));
        assert!(!main.contains("aurora_emissive"));
        assert!(!main.contains("saturation_post("));
        let encode = main.find("vec3 glassColor = linearToSrgb(").unwrap();
        let post = main
            .find("glassColor = noise_post(glassColor, gl_FragCoord.xy);")
            .unwrap();
        let coverage = main
            .find("glassed = vec4(glassColor, 1.0) * coverage;")
            .unwrap();
        assert!(
            encode < post && post < coverage,
            "the post hook sits between encode and coverage"
        );
        // GLSL ES needs the float precision declared before the first float
        // function; the shared helpers come right after it, before the prelude.
        assert!(source.starts_with("precision highp float;\n// ---- common\n"));
        assert_eq!(source.matches("precision highp float;").count(), 1);
        assert!(source.find("// ---- common").unwrap() < source.find("// ---- prelude").unwrap());
        assert!(source.contains("uniform vec4 mat_noise_site;"));
    }

    use niri_config::material::pipeline::{Program, STAGES};

    /// GLSL without its comments, so a commented-out call does not count.
    fn strip_comments(source: &str) -> String {
        let mut out = String::with_capacity(source.len());
        let mut rest = source;
        while !rest.is_empty() {
            if let Some(stripped) = rest.strip_prefix("/*") {
                let end = stripped.find("*/").map(|i| i + 2).unwrap_or(stripped.len());
                rest = &stripped[end..];
            } else if let Some(stripped) = rest.strip_prefix("//") {
                let end = stripped.find('\n').unwrap_or(stripped.len());
                rest = &stripped[end..];
            } else {
                let mut chars = rest.chars();
                out.push(chars.next().unwrap());
                rest = chars.as_str();
            }
        }
        out
    }

    /// Actual calls of `call` (`noise_behind(`): occurrences that are not a
    /// definition (preceded by a GLSL type) and not the tail of a longer
    /// identifier. Comments must already be stripped.
    fn hook_calls(source: &str, call: &str) -> Vec<usize> {
        const TYPES: &[&str] = &["void", "float", "vec2", "vec3", "vec4"];
        let bytes = source.as_bytes();
        let mut found = Vec::new();
        let mut from = 0;
        while let Some(at) = source[from..].find(call) {
            let at = from + at;
            from = at + 1;
            if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
                continue;
            }
            let before = source[..at].trim_end();
            let token_start = before
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map(|i| i + 1)
                .unwrap_or(0);
            if TYPES.contains(&&before[token_start..]) {
                continue;
            }
            found.push(at);
        }
        found
    }

    #[test]
    fn hook_call_counter_ignores_comments_and_definitions() {
        let glsl = strip_comments(
            "vec3 noise_behind(vec3 c, vec2 p) { return c; }\n// sampled = noise_behind(sampled, p);\n/* noise_behind( */\nsampled = noise_behind(sampled, p);\nx = xnoise_behind(1);\n",
        );
        assert_eq!(hook_calls(&glsl, "noise_behind(").len(), 1);
        let twice = strip_comments("a = noise_behind(a, p);\nb = noise_behind(b, p);\n");
        assert_eq!(hook_calls(&twice, "noise_behind(").len(), 2);
    }

    /// The material program after the `// ---- main` marker, comments
    /// stripped: hook calls are counted there, not in an optic's own GLSL
    /// where the function is defined.
    fn main_body() -> String {
        let source = material_source();
        let marker = "\n// ---- main\n";
        let at = source
            .find(marker)
            .expect("material_source carries the main marker");
        strip_comments(&source[at + marker.len()..])
    }

    #[test]
    fn pipeline_material_hooks_are_called_exactly_once_in_stage_order() {
        let body = main_body();
        let mut last = 0;
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            if optic.program != Program::Material {
                continue;
            }
            let call = format!("{}_{}(", optic.name, optic.hook);
            let calls = hook_calls(&body, &call);
            assert_eq!(
                calls.len(),
                1,
                "{}: main.frag calls {call} {} times, expected exactly one",
                stage.id,
                calls.len()
            );
            assert!(
                calls[0] >= last,
                "{}: {call} is called before an earlier stage's hook",
                stage.id
            );
            last = calls[0];
        }
    }

    /// The non-material programs, each as its files: an Effect hook must be
    /// called exactly once in exactly one effect file.
    fn program_files(program: Program) -> Vec<(&'static str, String)> {
        match program {
            Program::Material => Vec::new(),
            Program::Effect => vec![
                (
                    "blur_down.frag",
                    strip_comments(include_str!("blur_down.frag")),
                ),
                ("blur_up.frag", strip_comments(include_str!("blur_up.frag"))),
                ("grain.frag", strip_comments(&grain_source())),
            ],
            Program::Postprocess => vec![(
                "postprocess.frag",
                strip_comments(include_str!("postprocess.frag")),
            )],
        }
    }

    #[test]
    fn pipeline_other_programs_call_their_hooks_exactly_once() {
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            if optic.program == Program::Material {
                continue;
            }
            let call = format!("{}_{}(", optic.name, optic.hook);
            let calling: Vec<&str> = program_files(optic.program)
                .iter()
                .filter_map(|(file, source)| match hook_calls(source, &call).len() {
                    0 => None,
                    1 => Some(*file),
                    n => panic!("{}: {file} calls {call} {n} times", stage.id),
                })
                .collect();
            assert_eq!(
                calling.len(),
                1,
                "{}: the {:?} program must call {call} in exactly one file, found {calling:?}",
                stage.id,
                optic.program
            );
        }
    }

    #[test]
    fn grain_source_is_a_complete_effect_program_over_the_noise_optic() {
        let source = grain_source();
        assert!(source.starts_with("#version 100\n"));
        assert_eq!(source.matches("void main()").count(), 1);
        assert!(source.contains("uniform sampler2D tex;"));
        assert!(source.contains("vec4 noise_source(vec4 texel, vec2 fragCoord)"));
        assert!(source.contains("float hash12(vec2 p)"));
        assert!(
            !source.contains("niri_v_coords"),
            "the grain program has no material prelude"
        );
        let body = strip_comments(source.split_once("// ---- main").unwrap().1);
        assert_eq!(hook_calls(&body, "noise_source(").len(), 1);
    }

    /// Uniform names that are not parameters: slab geometry, jelly and signal
    /// state, textures, and the window's seed.
    const NON_PARAMETER_UNIFORMS: &[&str] = &[
        "mat_area_size",
        "mat_geo_rect",
        "mat_win_rect",
        "mat_slab_rect",
        "mat_bg_rect",
        "mat_backdrop_rect",
        "mat_ws_rect",
        "mat_ws_color",
        "mat_backdrop_color",
        "mat_corner_radius",
        "mat_chamfer",
        "mat_samples",
        "mat_bg_prefilter_mix",
        "mat_backdrop_prefilter_mix",
        "mat_jelly_seed",
        "mat_jelly_time",
        "mat_jelly_activity",
        "mat_jelly_move",
        "mat_jelly_resize",
        "mat_aurora_phase",
    ];

    /// Uniforms whose name is not the parameter's node: each maps to the
    /// nodes it carries.
    fn uniform_nodes(uniform: &str) -> Vec<&'static str> {
        match uniform {
            "mat_scatter" => vec!["roughness", "ior"],
            "mat_noise_scale" => vec!["noise scale="],
            "mat_noise_site" => vec!["noise site="],
            "mat_noise_type" => vec!["noise type="],
            "mat_aurora_color_a" | "mat_aurora_color_b" => vec!["aurora color"],
            "mat_distortion_scale" => vec!["distortion scale="],
            _ => Vec::new(),
        }
    }

    #[test]
    fn pipeline_optic_glsl_reads_are_declared() {
        for stage in STAGES {
            let Some(optic) = stage.optic else { continue };
            let entry = OPTICS.iter().find(|e| e.name == optic.name).unwrap();
            let mut seen = HashSet::new();
            for token in entry
                .glsl
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            {
                if !token.starts_with("mat_") || !seen.insert(token) {
                    continue;
                }
                if token.starts_with("mat_sig_") || NON_PARAMETER_UNIFORMS.contains(&token) {
                    continue;
                }
                let nodes = match uniform_nodes(token) {
                    v if v.is_empty() => {
                        vec![token.trim_start_matches("mat_").replace('_', "-").leak() as &str]
                    }
                    v => v,
                };
                for node in nodes {
                    assert!(
                        stage.reads.contains(&node),
                        "{}: its GLSL reads {token} ({node}), which `reads` omits",
                        stage.id
                    );
                }
            }
        }
    }
}
