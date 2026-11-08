//! Color math for the shell's surface remap
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §3, §4).
//!
//! `RemapSetup` turns a validated `surface-remap` node and its glass into
//! source anchors and target families; `RemapUniforms` is what
//! `surface_remap.frag` receives, and `apply` is that shader's CPU mirror.

use niri_config::layer_rule::SurfaceRemap;
use niri_config::material::face::face_srgb;
use niri_config::material::ResolvedGlass;
use niri_config::oklab::{
    chroma, color_to_oklab, linear_to_oklab, linear_to_srgb, oklab_to_linear, srgb_to_linear,
};
use niri_config::Color;

/// Distance to the family polyline (Oklab) where the remap starts to fade.
pub const R0: f64 = 0.008;
/// Distance at which the remap is gone.
pub const R1: f64 = 0.02;
/// Below this Oklab chroma an accent has no hue to show.
pub const MIN_ACCENT_CHROMA: f64 = 0.02;

/// A hue state in the coordinates where the target family is affine:
/// `neutral` is the weight of the unhued face, `hue` the weighted sum of
/// accent unit hue vectors (Oklab a, b). Lerping these lerps families.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HueBlend {
    pub neutral: f32,
    pub hue: [f32; 2],
}

impl HueBlend {
    pub const NEUTRAL: Self = Self {
        neutral: 1.,
        hue: [0., 0.],
    };

    pub fn from_accent(accent: Color) -> Self {
        let lab = color_to_oklab(accent);
        let c = chroma(lab);
        if c < MIN_ACCENT_CHROMA {
            return Self::NEUTRAL;
        }
        Self {
            neutral: 0.,
            hue: [(lab[1] / c) as f32, (lab[2] / c) as f32],
        }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            neutral: a.neutral + (b.neutral - a.neutral) * t,
            hue: [0, 1].map(|i| a.hue[i] + (b.hue[i] - a.hue[i]) * t),
        }
    }
}

impl Default for HueBlend {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

/// What every remapped layer on an output shows this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OutputHue {
    pub blend: HueBlend,
    /// A fade or switch blend is running, so frames must keep coming.
    pub animating: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemapSetup {
    /// Oklab of surface, container, outline as Noctalia draws them.
    sources: [[f64; 3]; 3],
    /// Oklab of the glass face over the reference backdrop.
    face: [f64; 3],
    /// Unit hue direction of the face, zero when it has no chroma.
    face_dir: [f64; 2],
    /// Per role: (ΔL, ΔC ≥ 0) from the source surface.
    role_offsets: [(f64, f64); 3],
    hue_chroma: f64,
}

impl RemapSetup {
    pub fn new(remap: &SurfaceRemap, glass: &ResolvedGlass) -> Self {
        Self::with_face(remap, face_srgb(glass))
    }

    /// Builds the setup against an explicit face swatch (sRGB in 0..1).
    pub fn with_face(remap: &SurfaceRemap, face_srgb: [f64; 3]) -> Self {
        let sources = [remap.surface, remap.container, remap.outline].map(color_to_oklab);
        let face = linear_to_oklab(face_srgb.map(srgb_to_linear));
        let face_c = chroma(face);
        let face_dir = if face_c > 1e-9 {
            [face[1] / face_c, face[2] / face_c]
        } else {
            [0., 0.]
        };
        let c0 = chroma(sources[0]);
        let role_offsets = sources.map(|s| (s[0] - sources[0][0], (chroma(s) - c0).max(0.)));
        Self {
            sources,
            face,
            face_dir,
            role_offsets,
            hue_chroma: remap.hue_chroma.map_or(0., |x| x.0),
        }
    }

    /// Target Oklab for surface, container and outline at `blend`.
    pub fn family(&self, blend: HueBlend) -> [[f64; 3]; 3] {
        // `hue-chroma 0` is the static look: no project hue, whatever the
        // workspace holds or how far a fade has got (plan review round 1).
        let blend = if self.hue_chroma > 0. {
            blend
        } else {
            HueBlend::NEUTRAL
        };
        let n = f64::from(blend.neutral);
        let h = blend.hue.map(f64::from);
        let ab = [0, 1].map(|i| n * self.face[i + 1] + self.hue_chroma * h[i]);
        let dir = [0, 1].map(|i| n * self.face_dir[i] + h[i]);
        self.role_offsets
            .map(|(dl, dc)| [self.face[0] + dl, ab[0] + dc * dir[0], ab[1] + dc * dir[1]])
    }

    pub fn uniforms(&self, blend: HueBlend) -> RemapUniforms {
        let family = self.family(blend);
        RemapUniforms {
            anchors: self.sources.map(|s| s.map(|v| v as f32)),
            offsets: [0, 1, 2]
                .map(|i| [0, 1, 2].map(|k| (family[i][k] - self.sources[i][k]) as f32)),
            radii: [R0 as f32, R1 as f32],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RemapUniforms {
    pub anchors: [[f32; 3]; 3],
    pub offsets: [[f32; 3]; 3],
    pub radii: [f32; 2],
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// Distance from `p` to the segment `a → b`, and the fraction along it.
fn segment(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> (f32, f32) {
    let ab = [0, 1, 2].map(|i| b[i] - a[i]);
    let len2: f32 = ab.iter().map(|v| v * v).sum();
    let t = if len2 > 0. {
        ([0, 1, 2].map(|i| (p[i] - a[i]) * ab[i]).iter().sum::<f32>() / len2).clamp(0., 1.)
    } else {
        0.
    };
    let d: f32 = [0, 1, 2]
        .map(|i| p[i] - (a[i] + t * ab[i]))
        .iter()
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt();
    (d, t)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

impl RemapUniforms {
    /// Distance to the family polyline and the interpolated offset there.
    /// Mirrors the remap shader's nearest-anchor search.
    fn nearest(&self, lab: [f32; 3]) -> (f32, [f32; 3]) {
        let [a0, a1, a2] = self.anchors;
        let [o0, o1, o2] = self.offsets;
        let (d0, t0) = segment(lab, a0, a1);
        let (d1, t1) = segment(lab, a1, a2);
        if d0 <= d1 {
            (d0, mix3(o0, o1, t0))
        } else {
            (d1, mix3(o1, o2, t1))
        }
    }

    pub fn weight(&self, lab: [f32; 3]) -> f32 {
        1. - smoothstep(self.radii[0], self.radii[1], self.nearest(lab).0)
    }

    /// The shader on one premultiplied sRGB pixel.
    pub fn apply(&self, rgba: [f32; 4]) -> [f32; 4] {
        let a = rgba[3];
        if a <= 0. {
            return rgba;
        }
        let straight = [0, 1, 2].map(|i| f64::from((rgba[i] / a).clamp(0., 1.)));
        let lab = linear_to_oklab(straight.map(srgb_to_linear)).map(|v| v as f32);
        let (d, offset) = self.nearest(lab);
        let w = 1. - smoothstep(self.radii[0], self.radii[1], d);
        if w <= 0. {
            return rgba;
        }
        let moved = [0, 1, 2].map(|i| f64::from(lab[i] + w * offset[i]));
        let srgb = oklab_to_linear(moved).map(|v| linear_to_srgb(v.clamp(0., 1.)) as f32);
        [srgb[0] * a, srgb[1] * a, srgb[2] * a, a]
    }
}

#[cfg(test)]
mod tests {
    use niri_config::material::face::{COEFF_FLOOR, SPECULAR_FLOOR};

    use super::*;

    fn hex(s: &str) -> Color {
        s.parse().unwrap()
    }

    /// Builds the node directly: the default `glass {}` is light glass and
    /// would fail the range check in a full config parse.
    fn remap(surface: &str, container: &str, outline: &str, hue_chroma: f64) -> SurfaceRemap {
        SurfaceRemap {
            surface: hex(surface),
            container: hex(container),
            outline: hex(outline),
            material: niri_config::material::MaterialRef {
                name: String::from("tg"),
                response: None,
            },
            hue_chroma: (hue_chroma > 0.).then_some(niri_config::material::Milli(hue_chroma)),
        }
    }

    fn live() -> SurfaceRemap {
        remap("#151310", "#221f1c", "#4d453c", 0.03)
    }

    fn srgb_levels(lab: [f64; 3]) -> [i32; 3] {
        oklab_to_linear(lab).map(|v| (linear_to_srgb(v.clamp(0., 1.)) * 255.).round() as i32)
    }

    fn lab32(s: &str) -> [f32; 3] {
        color_to_oklab(hex(s)).map(|v| v as f32)
    }

    fn lab32_mix(base: &str, over: &str, alpha: f64) -> [f32; 3] {
        let [br, bg, bb, _] = hex(base).to_array_unpremul();
        let [or, og, ob, _] = hex(over).to_array_unpremul();
        let mixed = [(br, or), (bg, og), (bb, ob)]
            .map(|(b, o)| srgb_to_linear(f64::from(b) * (1. - alpha) + f64::from(o) * alpha));
        linear_to_oklab(mixed).map(|v| v as f32)
    }

    #[test]
    fn neutral_family_reproduces_the_palette_helper() {
        // noctalia-glass-palette.py with target #090a0a on the live palette.
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let family = setup.family(HueBlend::NEUTRAL);
        for (got, want) in family.iter().zip([[9, 10, 10], [20, 22, 22], [51, 61, 61]]) {
            let got = srgb_levels(*got);
            for i in 0..3 {
                assert!((got[i] - want[i]).abs() <= 1, "{got:?} vs {want:?}");
            }
        }
    }

    #[test]
    fn hued_surface_keeps_the_face_lightness_and_takes_the_accent_hue() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let accent = hex("#e5533c");
        let family = setup.family(HueBlend::from_accent(accent));
        let face = linear_to_oklab([9., 10., 10.].map(|v: f64| srgb_to_linear(v / 255.)));
        assert!((family[0][0] - face[0]).abs() < 1e-9);
        assert!((chroma(family[0]) - 0.03).abs() < 1e-6);
        let want = color_to_oklab(accent);
        let hue = |lab: [f64; 3]| lab[2].atan2(lab[1]);
        assert!((hue(family[0]) - hue(want)).abs() < 1e-6);
        // Container and outline share the hue at their chroma steps.
        assert!((hue(family[2]) - hue(want)).abs() < 1e-6);
        assert!(chroma(family[2]) > chroma(family[0]));
    }

    #[test]
    fn zero_or_omitted_strength_keeps_the_static_look() {
        let red = HueBlend::from_accent(hex("#e5533c"));
        let omitted = remap("#151310", "#221f1c", "#4d453c", 0.);
        assert_eq!(omitted.hue_chroma, None);
        let explicit = SurfaceRemap {
            hue_chroma: Some(niri_config::material::Milli(0.)),
            ..omitted.clone()
        };
        for node in [omitted, explicit] {
            let setup = RemapSetup::with_face(&node, [9. / 255., 10. / 255., 10. / 255.]);
            let neutral = setup.family(HueBlend::NEUTRAL);
            // Settled on a red agent, and every point of a fade toward it.
            for t in [0., 0.25, 0.5, 1.] {
                let blend = HueBlend::lerp(HueBlend::NEUTRAL, red, t);
                assert_eq!(setup.family(blend), neutral, "{:?} at {t}", node.hue_chroma);
            }
            // Review example: the outline stays #333d3d, not about #413937.
            assert_eq!(
                srgb_levels(setup.family(red)[2]),
                [51, 61, 61],
                "{:?}",
                node.hue_chroma
            );
        }
    }

    #[test]
    fn muted_accent_is_neutral() {
        for gray in ["#ffffff", "#808080", "#000000", "#7f8081"] {
            let blend = HueBlend::from_accent(hex(gray));
            assert_eq!(blend, HueBlend::NEUTRAL, "{gray}");
            let u = RemapSetup::with_face(&live(), [0.03; 3]).uniforms(blend);
            assert!(
                u.offsets.iter().flatten().all(|v| v.is_finite()),
                "{gray}: {u:?}"
            );
        }
    }

    #[test]
    fn red_to_cyan_never_snaps() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let red = HueBlend::from_accent(hex("#ff0000"));
        let cyan = HueBlend::from_accent(hex("#00ffff"));
        let mut previous = setup.family(red);
        for step in 1..=2000 {
            let t = step as f32 / 2000.;
            let family = setup.family(HueBlend::lerp(red, cyan, t));
            for role in 0..3 {
                for k in 0..3 {
                    let jump = (family[role][k] - previous[role][k]).abs();
                    assert!(jump < 0.01, "t {t}: role {role} component {k} jumps {jump}");
                }
            }
            previous = family;
        }
        // The midpoint passes near neutral: hue directions cancel.
        let mid = HueBlend::lerp(red, cyan, 0.5);
        assert!(mid.hue[0].hypot(mid.hue[1]) < 0.2, "{mid:?}");
    }

    #[test]
    fn lerping_blends_lerps_families() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let a = HueBlend::NEUTRAL;
        let b = HueBlend::from_accent(hex("#3cc6e5"));
        for t in [0.1_f32, 0.37, 0.5, 0.9] {
            let direct = setup.family(HueBlend::lerp(a, b, t));
            let (fa, fb) = (setup.family(a), setup.family(b));
            for role in 0..3 {
                for k in 0..3 {
                    let lerped = fa[role][k] + (fb[role][k] - fa[role][k]) * f64::from(t);
                    assert!((direct[role][k] - lerped).abs() < 1e-6);
                }
            }
        }
    }

    // Role colors of three m3-content palettes (planning, 2026-10-17): the live
    // wallpaper, a blue one and a vivid green one.
    const PALETTES: [[&str; 14]; 3] = [
        [
            "#151310", "#221f1c", "#4d453c", "#ffe7cb", "#402d10", "#d5c4b0", "#392f21", "#e9eec4",
            "#2e3316", "#ffb4ab", "#690005", "#e8e1dc", "#d1c5b8", "#e9eec4",
        ],
        [
            "#111318", "#1d2025", "#414751", "#a4c9ff", "#00315d", "#b1c8ec", "#1a314e", "#f4aeff",
            "#510f62", "#ffb4ab", "#690005", "#e1e2e9", "#c1c7d3", "#f4aeff",
        ],
        [
            "#12150a", "#1e2115", "#444935", "#b5e435", "#273500", "#b8d07b", "#273500", "#5cef7e",
            "#003913", "#ffb4ab", "#690005", "#e2e4d2", "#c4c9af", "#5cef7e",
        ],
    ];
    // Indices into a PALETTES row.
    const SURFACE: usize = 0;
    const CONTAINER: usize = 1;
    const OUTLINE: usize = 2;
    const PRIMARY: usize = 3;
    const SECONDARY: usize = 5;
    const TERTIARY: usize = 7;
    const ERROR: usize = 9;
    const ON_ERROR: usize = 10;
    const ON_SURFACE: usize = 11;
    const ON_SURFACE_VARIANT: usize = 12;

    fn uniforms_for(p: &[&str; 14]) -> RemapUniforms {
        RemapSetup::with_face(
            &remap(p[SURFACE], p[CONTAINER], p[OUTLINE], 0.03),
            [0.035; 3],
        )
        .uniforms(HueBlend::NEUTRAL)
    }

    #[test]
    fn family_anchors_take_full_weight() {
        for p in &PALETTES {
            let u = uniforms_for(p);
            for role in [SURFACE, CONTAINER, OUTLINE] {
                assert!((u.weight(lab32(p[role])) - 1.).abs() < 1e-6, "{}", p[role]);
            }
        }
    }

    #[test]
    fn text_accents_errors_and_black_are_untouched() {
        for p in &PALETTES {
            let u = uniforms_for(p);
            for role in [
                ON_SURFACE,
                ON_SURFACE_VARIANT,
                PRIMARY,
                SECONDARY,
                TERTIARY,
                ERROR,
                ON_ERROR,
            ] {
                assert_eq!(u.weight(lab32(p[role])), 0., "{}", p[role]);
            }
            assert_eq!(u.weight(lab32("#000000")), 0.);
        }
    }

    #[test]
    fn neutral_overlays_move_with_the_family() {
        for p in &PALETTES {
            let u = uniforms_for(p);
            for alpha in [0.04, 0.08, 0.12, 0.16] {
                let w = u.weight(lab32_mix(p[SURFACE], p[ON_SURFACE], alpha));
                assert!(w >= 0.5, "{} at {alpha}: {w}", p[SURFACE]);
            }
        }
    }

    #[test]
    fn muted_primary_overlays_move_with_the_family() {
        // Spec §3: true for the live and blue palettes, not the vivid one.
        for p in &PALETTES[..2] {
            let u = uniforms_for(p);
            for alpha in [0.04, 0.08, 0.12, 0.16] {
                let w = u.weight(lab32_mix(p[SURFACE], p[PRIMARY], alpha));
                assert!(w >= 0.5, "{} at {alpha}: {w}", p[SURFACE]);
            }
        }
    }

    #[test]
    fn black_white_and_transparent_pixels_pass_through_unchanged() {
        let u = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.])
            .uniforms(HueBlend::from_accent(hex("#e5533c")));
        for px in [
            [0., 0., 0., 1.],
            [1., 1., 1., 1.],
            [0., 0., 0., 0.],
            [0., 0., 0., 0.5],
        ] {
            assert_eq!(u.apply(px), px);
        }
    }

    #[test]
    fn apply_moves_a_surface_pixel_to_the_target() {
        let setup = RemapSetup::with_face(&live(), [9. / 255., 10. / 255., 10. / 255.]);
        let u = setup.uniforms(HueBlend::NEUTRAL);
        let [r, g, b, _] = hex("#151310").to_array_unpremul();
        // A 0.95-alpha surface pixel, premultiplied.
        let out = u.apply([r * 0.95, g * 0.95, b * 0.95, 0.95]);
        let straight =
            [out[0] / 0.95, out[1] / 0.95, out[2] / 0.95].map(|v| (v * 255.).round() as i32);
        for (got, want) in straight.iter().zip([9, 10, 10]) {
            assert!((got - want).abs() <= 1, "{straight:?}");
        }
        assert_eq!(out[3], 0.95);
    }

    #[test]
    fn the_shader_names_the_face_constants() {
        let frag = include_str!("shaders/material/main.frag");
        // The constants are tied to the shader text, not to a literal.
        assert!(
            frag.contains(&format!("fresnel * ({SPECULAR_FLOOR} + 0.85 * facing)")),
            "SPECULAR_FLOOR {SPECULAR_FLOOR} differs from main.frag"
        );
        assert!(
            frag.contains(&format!(
                "clamp(mat_attenuation_color.rgb, vec3({COEFF_FLOOR}), vec3(1.0))"
            )),
            "COEFF_FLOOR {COEFF_FLOOR} differs from main.frag"
        );
        for line in [
            "vec3 specular = vec3(fresnel * (0.15 + 0.85 * facing));",
            "vec3 transmitted = sampled * att * (1.0 - fresnel);",
            "float f0 = (mat_ior - 1.0) / (mat_ior + 1.0);",
            "float fresnel = f0 + (1.0 - f0) * pow(1.0 - surfaceCosine, 5.0);",
        ] {
            assert!(frag.contains(line), "main.frag lacks `{line}`");
        }
    }
}
