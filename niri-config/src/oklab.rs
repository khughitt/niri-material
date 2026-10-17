//! sRGB transfer functions and Oklab (Björn Ottosson, public domain) in f64.
//!
//! The same formulas as `src/render_helpers/shaders/material/common.frag`;
//! the shell remap (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md)
//! validates and prepares its uniforms with these.

use crate::appearance::Color;

pub fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1. / 2.4) - 0.055
    }
}

pub fn linear_to_oklab(c: [f64; 3]) -> [f64; 3] {
    let l = 0.4122214708 * c[0] + 0.5363325363 * c[1] + 0.0514459929 * c[2];
    let m = 0.2119034982 * c[0] + 0.6806995451 * c[1] + 0.1073969566 * c[2];
    let s = 0.0883024619 * c[0] + 0.2817188376 * c[1] + 0.6299787005 * c[2];
    let [l, m, s] = [l, m, s].map(|v| v.max(0.).cbrt());
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

pub fn oklab_to_linear(lab: [f64; 3]) -> [f64; 3] {
    let l = lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2];
    let m = lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2];
    let s = lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2];
    let [l, m, s] = [l, m, s].map(|v| v * v * v);
    [
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    ]
}

/// Oklab of a config color (sRGB-encoded, straight alpha ignored).
pub fn color_to_oklab(c: Color) -> [f64; 3] {
    let [r, g, b, _] = c.to_array_unpremul();
    linear_to_oklab([r, g, b].map(|v| srgb_to_linear(f64::from(v))))
}

pub fn chroma(lab: [f64; 3]) -> f64 {
    lab[1].hypot(lab[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Color {
        s.parse().unwrap()
    }

    #[test]
    fn noctalia_surface_has_the_measured_lightness_and_chroma() {
        // Values from the planning computation on the live wallpaper's palette.
        let lab = color_to_oklab(hex("#151310"));
        assert!((lab[0] - 0.188).abs() < 0.001, "{lab:?}");
        assert!((chroma(lab) - 0.007).abs() < 0.001, "{lab:?}");
    }

    #[test]
    fn black_and_white_are_the_lightness_ends() {
        let black = color_to_oklab(hex("#000000"));
        let white = color_to_oklab(hex("#ffffff"));
        assert!(black.iter().all(|v| v.abs() < 1e-9), "{black:?}");
        assert!(
            (white[0] - 1.).abs() < 1e-4 && chroma(white) < 1e-4,
            "{white:?}"
        );
    }

    #[test]
    fn oklab_round_trips() {
        // The published matrices are inverse only to their 10 digits: the
        // round trip errs by about 3e-8 to 2e-7 (plan review round 1), far
        // below one 8-bit level (about 4e-3 at the darkest step).
        for rgb in [[0.2, 0.5, 0.9], [0.01, 0.003, 0.4], [0.9, 0.9, 0.1]] {
            let back = oklab_to_linear(linear_to_oklab(rgb));
            for i in 0..3 {
                assert!((back[i] - rgb[i]).abs() < 1e-6, "{rgb:?} -> {back:?}");
            }
        }
    }
}
