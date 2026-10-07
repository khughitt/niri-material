//! Noise layers (design 2026-10-06-noise-layers-design.md §7.2, §8): slot
//! identities, per-slot application and seeding, independence, and grain
//! size with its normalisation and smoothness by position in the lattice
//! cell, rendered in process through the headless GLES renderer under a frozen clock.

use std::time::Duration;

use niri_config::Config;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use smithay::utils::{Logical, Point, Rectangle, Size};

use super::client::LayerConfigureProps;
use super::fixture::Fixture;
use super::ring_pair::{diff, render_at, set_time};

// One large window, so each of the 64 position classes at scale 8 holds
// about 16 000 face pixels and a 10 % tolerance spans many standard errors.
const OUT_W: u16 = 1280;
const OUT_H: u16 = 1024;
const W: u16 = 1200;
const H: u16 = 960;
const BEVEL: u16 = 12;
const OFFSET: u16 = 6;
const NONE: &str = "";

fn config(noise: &str) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 16
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
            background-color "transparent"
        }}
        animations {{ off; }}
        blur {{ passes 3; offset 3; noise 0; saturation 1; }}
        material "a" {{
            glass {{
                ior 1
                thickness 20
                bevel {BEVEL}
                offset-x {OFFSET}
                offset-y {OFFSET}
                attenuation-color "#ffffff"
                attenuation-distance 60
                chromatic-aberration 0
                anisotropic-blur 0
                distortion 0 scale=0.5
                roughness 0
                backdrop-blur false
                jelly-flex 0
                jelly-ripple 0
                saturation 1
                aurora 0 {{ drift-hz 0; }}
                iridescence 0
                {noise}
            }}
            response "default" {{ focus "none"; accent "none"; ring-beam-speed 0; }}
        }}
        window-rule {{
            material "a"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        "##
    ))
    .unwrap()
}

/// One transparent window over a warm mid-tone background layer that fills
/// the output.
fn fixture() -> Fixture {
    let mut f = Fixture::with_config(config(NONE));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));

    let bg = f.add_client();
    let layer = f.client(bg).create_layer(None, Layer::Background, "");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Top | Anchor::Left | Anchor::Right),
        size: Some((0, u32::from(OUT_H))),
        exclusive_zone: Some(-1),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(bg);
    let layer = f.client(bg).layer(&surface);
    layer.attach_new_colored_buffer(140, 115, 90, 255);
    layer.set_size(OUT_W, OUT_H);
    layer.ack_last_and_commit();
    f.double_roundtrip(bg);

    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(0);
    window.set_size(W, H);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    f.niri_state().update_keyboard_focus();
    f.double_roundtrip(id);

    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

fn reload(f: &mut Fixture, noise: &str) {
    f.niri_state().reload_config(Ok(config(noise)));
    f.niri_state().refresh_and_flush_clients();
}

/// RGBA pixels of the output and the window's flat face in output px.
fn render(f: &mut Fixture) -> (Vec<u8>, Rectangle<i32, Logical>) {
    let pixels = render_at(f, Duration::ZERO);
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace.tiles_with_render_positions().next().unwrap();
    let rect = Rectangle::new(pos + tile.window_loc(), tile.animated_window_size());
    let inset = f64::from(BEVEL + OFFSET) + 2.;
    let face = Rectangle::new(
        (rect.loc + Point::from((inset, inset))).to_i32_round(),
        (rect.size - Size::from((2. * inset, 2. * inset))).to_i32_round(),
    );
    let output = Rectangle::from_size(Size::from((i32::from(OUT_W), i32::from(OUT_H))));
    assert_eq!(
        face.intersection(output),
        Some(face),
        "the face {face:?} must lie inside the output"
    );
    assert!(
        face.size.w > 1000 && face.size.h > 800,
        "the window did not take its {W}x{H} size: face {face:?}"
    );
    (pixels, face)
}

/// The signed green-channel grain `on - off` over the face, row-major, with
/// each pixel's output coordinates.
fn grain(on: &[u8], off: &[u8], face: Rectangle<i32, Logical>) -> Vec<(i32, i32, f64)> {
    let w = usize::from(OUT_W);
    let mut out = Vec::with_capacity((face.size.w * face.size.h) as usize);
    for y in face.loc.y..face.loc.y + face.size.h {
        for x in face.loc.x..face.loc.x + face.size.w {
            let i = (y as usize * w + x as usize) * 4 + 1;
            out.push((x, y, f64::from(on[i]) - f64::from(off[i])));
        }
    }
    out
}

/// The grain of `noise` against no noise, over the face.
fn grain_of(f: &mut Fixture, noise: &str) -> (Vec<(i32, i32, f64)>, Rectangle<i32, Logical>) {
    reload(f, NONE);
    let (off, face) = render(f);
    reload(f, noise);
    let (on, _) = render(f);
    (grain(&on, &off, face), face)
}

fn sd(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

fn grain_sd(g: &[(i32, i32, f64)]) -> f64 {
    sd(&g.iter().map(|p| p.2).collect::<Vec<_>>())
}

/// The grain's deviation per position class `(x mod s, y mod s)`: each class
/// holds the pixels at one position inside the lattice cell, whatever the
/// cell's phase or the framebuffer's orientation.
fn position_sds(g: &[(i32, i32, f64)], s: i32) -> Vec<f64> {
    let mut classes = vec![Vec::new(); (s * s) as usize];
    for &(x, y, v) in g {
        classes[(y.rem_euclid(s) * s + x.rem_euclid(s)) as usize].push(v);
    }
    classes.iter().map(|c| sd(c)).collect()
}

/// Mean squared difference to the right and lower neighbours, per position
/// class `(x mod s, y mod s)`, over the classes' mean. A lattice whose grain
/// flattens at its points and steepens along its cell edges, the grid the
/// owner saw at scale 8 (design §4), spreads these far from 1.
fn position_gradient_energies(
    g: &[(i32, i32, f64)],
    face: Rectangle<i32, Logical>,
    s: i32,
) -> Vec<f64> {
    let (w, h) = (face.size.w as usize, face.size.h as usize);
    let mut sums = vec![(0., 0usize); (s * s) as usize];
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let (px, py, v) = g[y * w + x];
            let dx = g[y * w + x + 1].2 - v;
            let dy = g[(y + 1) * w + x].2 - v;
            let class = &mut sums[(py.rem_euclid(s) * s + px.rem_euclid(s)) as usize];
            class.0 += dx * dx + dy * dy;
            class.1 += 1;
        }
    }
    let means: Vec<f64> = sums.iter().map(|(e, n)| e / *n as f64).collect();
    let mean = means.iter().sum::<f64>() / means.len() as f64;
    means.iter().map(|m| m / mean).collect()
}

/// Deviation of 4x4 block means over the pixels' deviation: about 1/4 for
/// independent pixels, rising toward 1 as the grain coarsens.
fn low_frequency_ratio(g: &[(i32, i32, f64)], face: Rectangle<i32, Logical>) -> f64 {
    let (w, h) = (face.size.w as usize, face.size.h as usize);
    let mut means = Vec::with_capacity((w / 4) * (h / 4));
    for by in 0..h / 4 {
        for bx in 0..w / 4 {
            let mut sum = 0.;
            for dy in 0..4 {
                for dx in 0..4 {
                    sum += g[(by * 4 + dy) * w + bx * 4 + dx].2;
                }
            }
            means.push(sum / 16.);
        }
    }
    sd(&means) / grain_sd(g)
}

#[test]
fn one_layer_equals_the_same_layer_with_empty_slots_after_it() {
    let mut f = fixture();
    for site in ["glass", "backdrop", "film"] {
        for kind in ["white", "fine", "lightness"] {
            let one = format!("noise 0.3 type=\"{kind}\" site=\"{site}\"");
            reload(&mut f, &one);
            let (a, _) = render(&mut f);
            reload(&mut f, &format!("{one}\nnoise 0\nnoise 0\nnoise 0"));
            let (b, _) = render(&mut f);
            assert_eq!(diff(&a, &b), (0, 0), "{kind} at {site}");
        }
    }
}

#[test]
fn written_scale_one_equals_omitted() {
    let mut f = fixture();
    for site in ["glass", "backdrop", "film"] {
        for kind in ["white", "fine", "lightness"] {
            let omitted = format!("noise 0.3 type=\"{kind}\" site=\"{site}\"");
            reload(&mut f, &omitted);
            let (a, _) = render(&mut f);
            reload(&mut f, &format!("{omitted} scale=1"));
            let (b, _) = render(&mut f);
            assert_eq!(diff(&a, &b), (0, 0), "{kind} at {site}");
        }
    }
}

#[test]
fn amount_zero_layers_are_neutral_with_any_properties() {
    let mut f = fixture();
    reload(&mut f, NONE);
    let (none, _) = render(&mut f);
    reload(
        &mut f,
        "noise 0 type=\"lightness\" scale=8 site=\"film\"\n\
         noise 0 type=\"fine\" scale=3 site=\"backdrop\"\n\
         noise 0 type=\"white\" scale=16",
    );
    let (zero, _) = render(&mut f);
    assert_eq!(diff(&none, &zero), (0, 0));
}

#[test]
fn every_slot_is_applied_and_seeded_apart() {
    let mut f = fixture();
    let (slot0, _) = grain_of(&mut f, "noise 0.3 type=\"fine\"");
    let s0 = grain_sd(&slot0);
    assert!(s0 > 5., "slot 0 grain is present: sd {s0:.2}");
    for k in 1..4 {
        let noise = std::iter::repeat("noise 0")
            .take(k)
            .chain(["noise 0.3 type=\"fine\""])
            .collect::<Vec<_>>()
            .join("\n");
        let (g, _) = grain_of(&mut f, &noise);
        let sk = grain_sd(&g);
        assert!(
            (sk / s0 - 1.).abs() < 0.05,
            "slot {k}: sd {sk:.2} vs slot 0 {s0:.2}"
        );
        let difference: Vec<f64> = g.iter().zip(&slot0).map(|(a, b)| a.2 - b.2).collect();
        assert!(
            sd(&difference) >= s0,
            "slot {k} repeats slot 0's pattern: difference sd {:.2}",
            sd(&difference)
        );
    }
}

#[test]
fn two_independent_layers_add_in_quadrature() {
    let mut f = fixture();
    let (one, _) = grain_of(&mut f, "noise 0.2 type=\"fine\"");
    let (two, _) = grain_of(&mut f, "noise 0.2 type=\"fine\"\nnoise 0.2 type=\"fine\"");
    let ratio = grain_sd(&two) / grain_sd(&one);
    assert!(
        (ratio / std::f64::consts::SQRT_2 - 1.).abs() < 0.05,
        "two layers over one: {ratio:.3}, expected about 1.414"
    );
}

#[test]
fn grain_deviation_holds_across_scales_and_cell_positions() {
    let mut f = fixture();
    for kind in ["white", "fine"] {
        let (g1, face) = grain_of(&mut f, &format!("noise 0.3 type=\"{kind}\""));
        let base = grain_sd(&g1);
        let mut previous = low_frequency_ratio(&g1, face);
        for scale in [2, 4, 8] {
            let (g, _) = grain_of(&mut f, &format!("noise 0.3 type=\"{kind}\" scale={scale}"));
            let aggregate = grain_sd(&g);
            assert!(
                (aggregate / base - 1.).abs() < 0.10,
                "{kind} scale {scale}: sd {aggregate:.2} vs scale 1 {base:.2}"
            );
            for (i, class) in position_sds(&g, scale).into_iter().enumerate() {
                let (x, y) = (i as i32 % scale, i as i32 / scale);
                assert!(
                    (class / aggregate - 1.).abs() < 0.10,
                    "{kind} scale {scale}: position ({x}, {y}) has sd {class:.2} vs {aggregate:.2}"
                );
            }
            let ratio = low_frequency_ratio(&g, face);
            assert!(
                ratio > previous,
                "{kind} scale {scale}: low-frequency ratio {ratio:.3} did not rise from {previous:.3}"
            );
            previous = ratio;
        }
    }
}

/// Mean and maximum absolute RGB difference over the face, in codes.
fn face_diff(a: &[u8], b: &[u8], face: Rectangle<i32, Logical>) -> (f64, u8) {
    let w = usize::from(OUT_W);
    let (mut sum, mut max, mut n) = (0u64, 0u8, 0u64);
    for y in face.loc.y..face.loc.y + face.size.h {
        for x in face.loc.x..face.loc.x + face.size.w {
            let i = (y as usize * w + x as usize) * 4;
            for c in 0..3 {
                let d = a[i + c].abs_diff(b[i + c]);
                sum += u64::from(d);
                max = max.max(d);
                n += 1;
            }
        }
    }
    (sum as f64 / n as f64, max)
}

/// Lightness in slot 0, then white in slot 1, with each slot's seed fixed.
/// In the reference the hooks fix the order: the glass hook runs the
/// lightness layer before the film hook adds white. Applied as a stack at
/// one site, the same two layers must give the same pixels, so a shader that
/// applies a site's slots in another order fails here. Swapping the config
/// lines cannot test this, because it swaps the seeds too. At amount 0.5 each
/// on this backdrop, reversing the two moves a pixel by about 1.4 codes on
/// average (a float simulation of the formulas while planning); the glass and
/// film stacks differ from the reference only by float rounding, and the
/// backdrop stack by its 8-bit storage (under half a code on average).
#[test]
fn a_sites_layers_apply_in_slot_order() {
    let mut f = fixture();
    reload(
        &mut f,
        r#"noise 0.5 type="lightness" site="glass"
           noise 0.5 type="white" site="film""#,
    );
    let (reference, face) = render(&mut f);
    for (site, mean_limit, max_limit) in [("glass", 0.1, 1), ("film", 0.1, 1), ("backdrop", 0.7, 2)]
    {
        reload(
            &mut f,
            &format!(
                r#"noise 0.5 type="lightness" site="{site}"
                   noise 0.5 type="white" site="{site}""#
            ),
        );
        let (stack, _) = render(&mut f);
        let (mean, max) = face_diff(&reference, &stack, face);
        assert!(
            mean <= mean_limit && max <= max_limit,
            "{site} stack against the hook-ordered reference: mean {mean:.3}, max {max} codes"
        );
    }
}

#[test]
fn the_lattice_does_not_show_by_position_in_the_cell() {
    let mut f = fixture();
    for kind in ["white", "fine"] {
        for scale in [2, 4, 8] {
            let (g, face) = grain_of(&mut f, &format!("noise 0.3 type=\"{kind}\" scale={scale}"));
            for (i, e) in position_gradient_energies(&g, face, scale)
                .into_iter()
                .enumerate()
            {
                let (x, y) = (i as i32 % scale, i as i32 / scale);
                assert!(
                    (0.4..=2.0).contains(&e),
                    "{kind} scale {scale}: position ({x}, {y}) has gradient energy {e:.2} of the mean"
                );
            }
        }
    }
}

#[test]
fn a_fractional_scale_keeps_the_deviation() {
    let mut f = fixture();
    let (g1, _) = grain_of(&mut f, "noise 0.3 type=\"fine\"");
    let (g, _) = grain_of(&mut f, "noise 0.3 type=\"fine\" scale=2.5");
    let (base, sd25) = (grain_sd(&g1), grain_sd(&g));
    assert!(
        (sd25 / base - 1.).abs() < 0.10,
        "scale 2.5: sd {sd25:.2} vs scale 1 {base:.2}"
    );
}
