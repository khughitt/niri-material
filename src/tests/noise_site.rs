//! Noise placement (design 2026-10-05-noise-placement-design.md §5, §8): the
//! three sites' identities, the backdrop grain's softening under blur, the
//! coverage facts, and the damage contract, rendered in process through the
//! headless GLES renderer under a frozen clock.

use std::time::Duration;

use niri_config::Config;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use smithay::utils::{Logical, Rectangle};

use super::client::LayerConfigureProps;
use super::fixture::Fixture;
use super::ring_pair::{diff, render_at, set_time};
use crate::render_helpers::RenderTarget;

// Two 320 px windows, three 16 px gaps: 688 px, inside the 800 px output,
// so neither window is pushed offscreen when the second takes focus.
const OUT_W: u16 = 800;
const OUT_H: u16 = 480;
const W: u16 = 320;
const H: u16 = 240;
const BEVEL: u16 = 12;
const OFFSET: u16 = 6;

/// `noise` lines for the focused material (`a`) and the second one (`b`).
struct Look<'a> {
    noise_a: &'a str,
    noise_b: &'a str,
    ior: &'a str,
    backdrop_blur: bool,
    blur_passes: u8,
    /// The second window's rule: a material, or a background-effect blur with no material.
    second_rule: &'a str,
}

impl Default for Look<'static> {
    fn default() -> Self {
        Self {
            noise_a: "",
            noise_b: "",
            ior: "1",
            backdrop_blur: false,
            blur_passes: 3,
            second_rule: "material \"b\"",
        }
    }
}

fn glass(noise: &str, ior: &str, backdrop_blur: bool) -> String {
    format!(
        r##"
            glass {{
                ior {ior}
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
                backdrop-blur {backdrop_blur}
                jelly-flex 0
                jelly-ripple 0
                saturation 1
                aurora 0 {{ drift-hz 0; }}
                iridescence 0
                {noise}
            }}
            response "default" {{ focus "none"; accent "none"; ring-beam-speed 0; }}
        "##
    )
}

fn config(look: &Look) -> Config {
    let a = glass(look.noise_a, look.ior, look.backdrop_blur);
    let b = glass(look.noise_b, look.ior, look.backdrop_blur);
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
        blur {{ passes {passes}; offset 3; noise 0; saturation 1; }}
        material "a" {{ {a} }}
        material "b" {{ {b} }}
        window-rule {{
            match is-active=true
            material "a"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        window-rule {{
            match is-active=false
            {second}
        }}
        "##,
        passes = look.blur_passes,
        second = look.second_rule,
    ))
    .unwrap()
}

/// Two transparent windows over a warm mid-tone background layer that fills
/// the output. Window 1 (mapped first, then window 2 takes focus) is the
/// second rule's; window 2 is active and gets material `a`.
fn fixture(look: &Look) -> Fixture {
    fixture_with_layer(look, OUT_H)
}

fn fixture_with_layer(look: &Look, layer_h: u16) -> Fixture {
    let mut f = Fixture::with_config(config(look));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));

    let bg = f.add_client();
    let layer = f.client(bg).create_layer(None, Layer::Background, "");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Top | Anchor::Left | Anchor::Right),
        size: Some((0, u32::from(layer_h))),
        exclusive_zone: Some(-1),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(bg);
    let layer = f.client(bg).layer(&surface);
    layer.attach_new_colored_buffer(140, 115, 90, 255);
    layer.set_size(OUT_W, layer_h);
    layer.ack_last_and_commit();
    f.double_roundtrip(bg);

    for _ in 0..2 {
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
    }
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

fn reload(f: &mut Fixture, look: &Look) {
    f.niri_state().reload_config(Ok(config(look)));
    f.niri_state().refresh_and_flush_clients();
}

/// The rectangle of the tile whose material has this name, or of the tile
/// with no material when `name` is `None`. `window_rect` in ring_pair.rs
/// takes the first tile, which here is the second rule's window, so every
/// test names the window it means.
fn tile_rect(f: &mut Fixture, name: Option<&str>) -> Rectangle<f64, Logical> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.material().map(|m| m.material().name.as_str()) == name)
        .unwrap_or_else(|| panic!("no tile with material {name:?}"));
    Rectangle::new(pos + tile.window_loc(), tile.animated_window_size())
}

/// RGBA pixels of the output, and material `a`'s flat face in output px.
fn render(f: &mut Fixture) -> (Vec<u8>, Rectangle<i32, Logical>) {
    let pixels = render_at(f, Duration::ZERO);
    let rect = tile_rect(f, Some("a"));
    let inset = f64::from(BEVEL + OFFSET) + 2.;
    let face = Rectangle::new(
        (rect.loc + smithay::utils::Point::from((inset, inset))).to_i32_round(),
        (rect.size - smithay::utils::Size::from((2. * inset, 2. * inset))).to_i32_round(),
    );
    (pixels, face)
}

fn region_pixels<'a>(
    pixels: &'a [u8],
    rect: Rectangle<i32, Logical>,
) -> impl Iterator<Item = &'a [u8]> {
    // Clip to the output: a region that reaches past it would index out of
    // range (or wrap, without overflow checks), and an empty one would make
    // every mean a division by zero.
    let output = Rectangle::from_size(smithay::utils::Size::from((
        i32::from(OUT_W),
        i32::from(OUT_H),
    )));
    let rect = rect
        .intersection(output)
        .unwrap_or_else(|| panic!("region {rect:?} lies outside the {OUT_W}x{OUT_H} output"));
    let w = usize::from(OUT_W);
    (rect.loc.y..rect.loc.y + rect.size.h).flat_map(move |y| {
        (rect.loc.x..rect.loc.x + rect.size.w).map(move |x| {
            let i = (y as usize * w + x as usize) * 4;
            &pixels[i..i + 4]
        })
    })
}

/// Max channel delta and mean absolute delta over the RGB of a region.
fn region_diff(a: &[u8], b: &[u8], rect: Rectangle<i32, Logical>) -> (u8, f64) {
    let mut max = 0u8;
    let mut sum = 0u64;
    let mut n = 0u64;
    for (pa, pb) in region_pixels(a, rect).zip(region_pixels(b, rect)) {
        for c in 0..3 {
            let d = pa[c].abs_diff(pb[c]);
            max = max.max(d);
            sum += u64::from(d);
            n += 1;
        }
    }
    (max, sum as f64 / n as f64)
}

/// Standard deviation of the signed green-channel difference over a region.
fn region_sd(a: &[u8], b: &[u8], rect: Rectangle<i32, Logical>) -> f64 {
    let diffs: Vec<f64> = region_pixels(a, rect)
        .zip(region_pixels(b, rect))
        .map(|(pa, pb)| f64::from(pa[1]) - f64::from(pb[1]))
        .collect();
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    (diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / diffs.len() as f64).sqrt()
}

#[test]
fn omitted_site_equals_glass_pixel_for_pixel() {
    for kind in ["white", "fine", "lightness"] {
        let omitted = Look {
            noise_a: &format!("noise 0.3 type=\"{kind}\""),
            ..Default::default()
        };
        let mut f = fixture(&omitted);
        let (a, _) = render(&mut f);
        reload(
            &mut f,
            &Look {
                noise_a: &format!("noise 0.3 type=\"{kind}\" site=\"glass\""),
                ..Default::default()
            },
        );
        let (b, _) = render(&mut f);
        assert_eq!(diff(&a, &b), (0, 0), "{kind}");
    }
}
#[test]
fn amount_zero_is_neutral_at_every_site() {
    let mut f = fixture(&Look::default());
    let (none, _) = render(&mut f);
    for site in ["glass", "backdrop", "film"] {
        reload(
            &mut f,
            &Look {
                noise_a: &format!("noise 0 type=\"fine\" site=\"{site}\""),
                ..Default::default()
            },
        );
        let (zero, _) = render(&mut f);
        assert_eq!(diff(&none, &zero), (0, 0), "{site}");
    }
}

#[test]
fn film_equals_glass_on_the_face_within_one_code() {
    for kind in ["white", "fine", "lightness"] {
        let mut f = fixture(&Look {
            noise_a: &format!("noise 0.3 type=\"{kind}\" site=\"glass\""),
            ..Default::default()
        });
        let (glass, face) = render(&mut f);
        reload(
            &mut f,
            &Look {
                noise_a: &format!("noise 0.3 type=\"{kind}\" site=\"film\""),
                ..Default::default()
            },
        );
        let (film, _) = render(&mut f);
        let (max, _) = region_diff(&glass, &film, face);
        assert!(
            max <= 1,
            "{kind}: film differs from glass by {max} codes on the face"
        );
        let (none, _) = {
            reload(&mut f, &Look::default());
            render(&mut f)
        };
        assert!(
            region_sd(&film, &none, face) > 5.,
            "{kind}: film grain is present"
        );
    }
}

#[test]
fn backdrop_equals_glass_on_the_face_at_blur_off_within_two_codes() {
    let mut f = fixture(&Look {
        noise_a: "noise 0.3 type=\"fine\" site=\"glass\"",
        ..Default::default()
    });
    let (glass, face) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..Default::default()
        },
    );
    let (backdrop, _) = render(&mut f);
    let (max, mean) = region_diff(&glass, &backdrop, face);
    assert!(
        mean <= 2.,
        "backdrop grain differs from glass grain by {mean:.2} codes on average (max {max}); \
         a mean near the grain's own spread means the seeds do not land on the same pixels"
    );
}

#[test]
fn backdrop_grain_softens_under_three_blur_passes() {
    let sharp = Look {
        noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
        ..Default::default()
    };
    let mut f = fixture(&sharp);
    let (grained, face) = render(&mut f);
    reload(&mut f, &Look::default());
    let (plain, _) = render(&mut f);
    let sd_sharp = region_sd(&grained, &plain, face);

    let frosted = Look {
        backdrop_blur: true,
        ..sharp
    };
    reload(&mut f, &frosted);
    let (grained, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            backdrop_blur: true,
            ..Default::default()
        },
    );
    let (plain, _) = render(&mut f);
    let sd_frosted = region_sd(&grained, &plain, face);
    assert!(
        sd_sharp > 5.,
        "sharp backdrop grain is present: sd {sd_sharp:.2}"
    );
    assert!(
        sd_frosted < 0.5 * sd_sharp,
        "three passes should remove at least half the grain: {sd_frosted:.2} vs {sd_sharp:.2}"
    );
}

#[test]
fn backdrop_to_glass_reload_clears_the_grain() {
    let glass_look = Look {
        noise_a: "noise 0.3 type=\"fine\" site=\"glass\"",
        ..Default::default()
    };
    let mut f = fixture(&glass_look);
    let (glass, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..Default::default()
        },
    );
    let _ = render(&mut f);
    reload(&mut f, &glass_look);
    let (again, _) = render(&mut f);
    assert_eq!(
        diff(&glass, &again),
        (0, 0),
        "glass after backdrop equals glass before it"
    );
}

#[test]
fn backdrop_grain_leaves_transparent_texels_alone() {
    let half = OUT_H / 2;
    let mut f = fixture_with_layer(&Look::default(), half);
    let (plain, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..Default::default()
        },
    );
    let (grained, _) = render(&mut f);
    let below = Rectangle::new(
        smithay::utils::Point::from((0, i32::from(half) + 2)),
        smithay::utils::Size::from((i32::from(OUT_W), i32::from(OUT_H - half) - 2)),
    );
    assert_eq!(
        region_diff(&plain, &grained, below),
        (0, 0.),
        "no layer, no grain"
    );
    let above = Rectangle::new(
        smithay::utils::Point::from((0, 0)),
        smithay::utils::Size::from((i32::from(OUT_W), i32::from(half) - 2)),
    );
    assert!(
        region_sd(&plain, &grained, above) > 0.,
        "the layer's half is grained where glass covers it"
    );
}

#[test]
fn window_site_element_sees_backdrop_grain() {
    let plain = Look {
        second_rule: "background-effect { blur true; noise 0; saturation 1; }",
        blur_passes: 1,
        ..Default::default()
    };
    let mut f = fixture(&plain);
    let (before, _) = render(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.3 type=\"fine\" site=\"backdrop\"",
            ..plain
        },
    );
    let (after, _) = render(&mut f);
    // The window with no material has the blurred background effect; its
    // area must change with the grain.
    let rect = tile_rect(&mut f, None).to_i32_round();
    let sd = region_sd(&before, &after, rect);
    assert!(
        sd > 0.5,
        "the background-effect element must see grain: sd {sd:.6}"
    );
}

#[test]
fn a_backdrop_only_reload_rerenders_the_unchanged_glass_window() {
    let look = Look {
        noise_a: "noise 0.2 type=\"fine\" site=\"backdrop\"",
        noise_b: "noise 0.1 type=\"fine\" site=\"glass\"",
        ..Default::default()
    };
    let mut f = fixture(&look);
    let _ = render(&mut f);
    let read = |f: &mut Fixture| {
        let output = f.niri_output(1);
        let niri = f.niri();
        let buffer = niri.output_state[&output].xray.background
            [RenderTarget::ScreenCapture as usize]
            .borrow()
            .commit();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let glass_window = workspace
            .tiles_with_render_positions()
            .find_map(|(tile, _, _)| tile.material().filter(|m| m.material().name == "b"))
            .expect("material b")
            .commit();
        (buffer, glass_window)
    };
    let (buffer_0, window_0) = read(&mut f);

    // Same config again: nothing moves.
    reload(&mut f, &look);
    let _ = render(&mut f);
    let (buffer_1, window_1) = read(&mut f);
    assert_eq!(
        buffer_1, buffer_0,
        "an unchanged backdrop grain publishes nothing"
    );
    assert_eq!(
        window_1, window_0,
        "an unchanged glass window keeps its commit"
    );

    // Only the backdrop amount changes; material `b` is byte-identical.
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.4 type=\"fine\" site=\"backdrop\"",
            ..look
        },
    );
    let _ = render(&mut f);
    let (buffer_2, window_2) = read(&mut f);
    assert_ne!(
        buffer_2, buffer_1,
        "the effect buffer publishes the grain change"
    );
    assert_ne!(
        window_2, window_1,
        "the glass window's fingerprint carries the buffer's commit"
    );
}

#[test]
fn a_backdrop_layer_scale_change_publishes_damage() {
    let look = Look {
        noise_a: "noise 0.2 type=\"fine\" site=\"backdrop\"\n\
                  noise 0.1 type=\"white\" site=\"backdrop\" scale=4",
        noise_b: "noise 0.1 type=\"fine\" site=\"glass\"",
        ..Default::default()
    };
    let mut f = fixture(&look);
    let _ = render(&mut f);
    let read = |f: &mut Fixture| {
        let output = f.niri_output(1);
        let niri = f.niri();
        let buffer = niri.output_state[&output].xray.background
            [RenderTarget::ScreenCapture as usize]
            .borrow()
            .commit();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let glass_window = workspace
            .tiles_with_render_positions()
            .find_map(|(tile, _, _)| tile.material().filter(|m| m.material().name == "b"))
            .expect("material b")
            .commit();
        (buffer, glass_window)
    };
    let (buffer_0, window_0) = read(&mut f);
    reload(
        &mut f,
        &Look {
            noise_a: "noise 0.2 type=\"fine\" site=\"backdrop\"\n\
                      noise 0.1 type=\"white\" site=\"backdrop\" scale=8",
            ..look
        },
    );
    let _ = render(&mut f);
    let (buffer_1, window_1) = read(&mut f);
    assert_ne!(
        buffer_1, buffer_0,
        "the effect buffer publishes the scale change"
    );
    assert_ne!(
        window_1, window_0,
        "the glass window's fingerprint carries the buffer's commit"
    );
}
