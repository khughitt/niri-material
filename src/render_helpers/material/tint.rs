//! Accent tint of the attenuation color (docs/specs/2026-10-03-accent-tint-design.md §5).

use niri_config::Color;

/// The shader's floor on the attenuation coefficient (`main.frag`, stage 4).
const COEFF_FLOOR: f64 = 0.001;

fn luminance(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// The attenuation color to upload: `base` tinted toward `chroma` by
/// `weight`, matched on the flat face's transmittance so the face passes the
/// untinted luminance of a neutral backdrop at every weight (design §5).
///
/// `chroma` is `accent_chroma` of the accent (or its crossfaded value);
/// `weight` is `accent-tint × presence`. The neutral path returns the
/// configured color bit for bit.
pub fn accent_tint(
    base: Color,
    thickness: f64,
    attenuation_distance: f64,
    chroma: Option<[f32; 3]>,
    weight: f64,
) -> [f32; 4] {
    let configured = base.to_array_unpremul();
    let Some(chroma) = chroma else {
        return configured;
    };
    let p_f = thickness / attenuation_distance;
    if weight <= 0. || p_f <= 0. {
        return configured;
    }

    let c = [0, 1, 2].map(|i| f64::from(configured[i]).clamp(COEFF_FLOOR, 1.));
    let t_c = c.map(|v| v.powf(p_f));
    let gray = luminance(t_c);
    if gray <= 0. {
        // A face that transmits nothing has no hue to show.
        return configured;
    }
    let lo = COEFF_FLOOR.powf(p_f);

    // The accent's hue at the face's density, pulled toward the gray until
    // every channel is inside [lo, 1]. The gray itself is inside, so a pull
    // always exists; it keeps luminance and gives up saturation.
    let target = chroma.map(|k| f64::from(k) * gray);
    let mut s: f64 = 1.;
    for v in target {
        if v > 1. {
            s = s.min((1. - gray) / (v - gray));
        }
        if v < lo {
            s = s.min((gray - lo) / (gray - v));
        }
    }
    let target = target.map(|v| gray + s * (v - gray));

    let w = weight.min(1.);
    let t = [0, 1, 2].map(|i| t_c[i] + w * (target[i] - t_c[i]));
    let x = t.map(|v| (v.powf(1. / p_f).clamp(COEFF_FLOOR, 1.)) as f32);
    [x[0], x[1], x[2], configured[3]]
}

#[cfg(test)]
mod tests {
    use niri_config::Color;

    use super::*;
    use crate::render_helpers::signal::{accent_chroma, color_linear};

    fn hex(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8_unpremul(r, g, b, 0xff)
    }

    /// (name, attenuation-color, thickness, attenuation-distance): the
    /// default glass, the owner's accepted look, and a mid-density glass
    /// whose face exponent is below 1.
    fn glasses() -> [(&'static str, Color, f64, f64); 3] {
        [
            ("default", hex(0xdf, 0xe8, 0xff), 20., 60.),
            ("accepted", hex(0x0d, 0x1d, 0x1e), 31.2, 11.),
            ("mid", hex(0x7a, 0x88, 0x99), 20., 40.),
        ]
    }

    fn accents() -> [(&'static str, [f32; 3]); 7] {
        [
            ("orange", color_linear(hex(0xff, 0x66, 0x00))),
            ("blue", color_linear(hex(0x00, 0x66, 0xff))),
            ("magenta", color_linear(hex(0xff, 0x00, 0xff))),
            ("near-black red", color_linear(hex(0x03, 0x00, 0x00))),
            ("gray", color_linear(hex(0x80, 0x80, 0x80))),
            ("black", [0.; 3]),
            ("white", [1.; 3]),
        ]
    }

    fn y(c: [f64; 3]) -> f64 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    /// Face transmittance of an uploaded coefficient, clamped as the shader clamps.
    fn face(x: [f32; 4], p_f: f64) -> [f64; 3] {
        [0, 1, 2].map(|i| f64::from(x[i]).clamp(0.001, 1.).powf(p_f))
    }

    #[test]
    fn neutral_path_returns_the_configured_color_bitwise() {
        let orange = Some(accent_chroma(color_linear(hex(0xff, 0x66, 0x00))));
        for (name, base, thickness, distance) in glasses() {
            let configured = base.to_array_unpremul();
            assert_eq!(
                accent_tint(base, thickness, distance, orange, 0.),
                configured,
                "{name} w=0"
            );
            assert_eq!(
                accent_tint(base, thickness, distance, None, 1.),
                configured,
                "{name} no accent"
            );
            assert_eq!(
                accent_tint(base, 0., distance, orange, 1.),
                configured,
                "{name} zero thickness"
            );
        }
        // A face that transmits nothing: T_c underflows to 0.
        let dark = hex(0x0d, 0x1d, 0x1e);
        assert_eq!(
            accent_tint(dark, 200., 1e-3, orange, 1.),
            dark.to_array_unpremul(),
            "underflowing face"
        );
    }

    #[test]
    fn face_luminance_is_kept_at_every_weight() {
        for (gname, base, thickness, distance) in glasses() {
            let p_f = thickness / distance;
            let untinted = y(face(base.to_array_unpremul(), p_f));
            for (aname, accent) in accents() {
                for w in [0.25, 0.5, 1.] {
                    let x = accent_tint(base, thickness, distance, Some(accent_chroma(accent)), w);
                    let rel = (y(face(x, p_f)) / untinted - 1.).abs();
                    assert!(rel < 1e-5, "{gname} {aname} w={w}: {rel}");
                    assert!(
                        x[..3].iter().all(|&v| (0.001..=1.).contains(&v)),
                        "{gname} {aname} w={w}: {x:?} outside the shader clamp"
                    );
                    assert_eq!(x[3], base.to_array_unpremul()[3], "alpha unchanged");
                }
            }
        }
    }

    #[test]
    fn full_weight_shows_the_accents_linear_hue_on_the_face() {
        let (_, base, thickness, distance) = glasses()[1];
        let p_f = thickness / distance;
        for (name, accent) in [accents()[1], accents()[2]] {
            let x = accent_tint(base, thickness, distance, Some(accent_chroma(accent)), 1.);
            let t = face(x, p_f);
            let ts: f64 = t.iter().sum();
            let a: [f64; 3] = accent.map(f64::from);
            let as_: f64 = a.iter().sum();
            for i in 0..3 {
                assert!(
                    (t[i] / ts - a[i] / as_).abs() < 1e-5,
                    "{name}: {t:?} vs {a:?}"
                );
            }
        }
    }

    #[test]
    fn light_glass_gives_up_saturation_not_luminance() {
        let (_, base, thickness, distance) = glasses()[0];
        let p_f = thickness / distance;
        let magenta = accent_chroma(color_linear(hex(0xff, 0x00, 0xff)));
        let t = face(
            accent_tint(base, thickness, distance, Some(magenta), 1.),
            p_f,
        );
        let g = y(t);
        let spread = (t[0].max(t[1]).max(t[2]) - t[0].min(t[1]).min(t[2])) / g;
        assert!(spread > 0., "some hue remains: {t:?}");
        assert!(
            spread < 0.1,
            "near-white glass has little room for hue: {t:?}"
        );
    }
}
