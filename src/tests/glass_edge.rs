//! Frozen-clock renders of the glass edge (material-be611b): the instrument
//! for the glass edge optics' intended-change, neutrality and motion evidence
//! (docs/plans/2026-10-02-glass-edge-optics.md).
//!
//! One window over a flat backdrop, rendered through the real material shader
//! by the headless backend's GLES renderer, the way a screenshot renders it
//! (see ring_pair.rs). Set `GLASS_EDGE_DUMP=<dir>` to write each case as
//! `<case>.rgba` (raw RGBA), `<case>.png` and `<case>.json` (output size, the
//! window, face and slab rectangles, the renderer), which
//! docs/materials/scripts/glass-edge-compare.py reads.
//!
//! Each fixture gets the next jelly seed from a process-wide counter, and the
//! aurora reads the seed. Dumps compare across commits because nextest runs
//! every test in its own process, so `every_case_renders_frozen` sees the
//! same seed sequence each time as long as `cases()` only grows at its end.
//! Run it through `just test-one`, never `cargo test`.

use std::time::Duration;

use niri_config::Config;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use smithay::utils::{Logical, Rectangle};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::{ClientId, LayerConfigureProps};
use super::ring_pair::{diff, render_at, set_time};
use super::*;

const OUT_W: u16 = 1280;
const OUT_H: u16 = 720;
const W: u16 = 640;
const H: u16 = 360;
const REST: Duration = Duration::from_secs(2);
/// The smokes' warm-mid backdrop, rgb(140, 115, 90).
const BACKDROP: &str = "#8c735a";
/// Fully transparent content: every pixel shows the glass.
const CLEAR: u32 = 0x0000_0000;
/// Kitty-like content at 0.6 opacity (premultiplied).
const TRANSLUCENT: u32 = 0x990a_0c0e;
/// Opaque content: the shader must pass it through untouched.
const OPAQUE: u32 = 0xff0a_0c0e;

/// A pinned glass look and the geometry the evidence regions need.
#[derive(Clone, Copy)]
struct Look {
    name: &'static str,
    glass: &'static str,
    bevel: f64,
    offset: (f64, f64),
}

/// The stock glass the smokes pin.
const STOCK: Look = Look {
    name: "stock",
    glass: r##"
                ior 1.5
                thickness 20
                attenuation-color "#dfe8ff"
                attenuation-distance 60
                bevel 12
                offset-x 6
                offset-y 6"##,
    bevel: 12.,
    offset: (6., 6.),
};
/// Pinned values modelled on Prism's focused terminal glass (2026-10-02),
/// without grain or blur. Not the live config: that drifts.
const LIVE: Look = Look {
    name: "live",
    glass: r##"
                ior 1.28
                thickness 31.2
                attenuation-color "#2e3034"
                attenuation-distance 11
                chromatic-aberration 0.36
                roughness 0.24
                bevel 15
                offset-x -6
                offset-y -5"##,
    bevel: 15.,
    offset: (-6., -5.),
};
const RING_OFF: &str = r#"focus "none""#;
const RING_ON: &str = r#"focus "ring-light"
                ring-beam-speed 0"#;
const AURORA_ON: &str = "aurora 0.5 { drift-hz 0; }";

struct Case {
    name: String,
    look: Look,
    /// Glass lines after the look's own.
    extra: String,
    response: &'static str,
    content: u32,
}

fn case(look: Look, suffix: &str, extra: &str, response: &'static str) -> Case {
    Case {
        name: format!("{}-{suffix}", look.name),
        look,
        extra: extra.to_owned(),
        response,
        content: CLEAR,
    }
}

/// Every case the evidence steps dump. Each must parse on the commit that
/// dumps it; later tasks append their cases at the end, never in between.
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for look in [STOCK, LIVE] {
        cases.push(case(look, "off", "", RING_OFF));
        cases.push(case(look, "on", AURORA_ON, RING_ON));
        cases.push(case(look, "iridescence", "iridescence 0.8", RING_OFF));
    }
    cases.push(Case {
        content: TRANSLUCENT,
        ..case(LIVE, "translucent", "", RING_OFF)
    });
    cases.push(Case {
        content: OPAQUE,
        ..case(LIVE, "opaque", "", RING_OFF)
    });
    for look in [STOCK, LIVE] {
        cases.push(case(look, "reflection-0", "reflection 0", RING_OFF));
        cases.push(case(look, "reflection", "reflection 0.6", RING_OFF));
    }
    for look in [STOCK, LIVE] {
        cases.push(case(look, "highlight-0", "edge-highlight 0", RING_OFF));
        cases.push(case(look, "k2", "bevel-profile 2", RING_OFF));
        cases.push(case(
            look,
            "highlight",
            "bevel-profile 2\nedge-highlight 0.5",
            RING_OFF,
        ));
    }
    cases
}

/// The output's config. Distortion and jelly ripple are left to the glass
/// lines (defaults 0 and 0.06; ripple acts only in motion), so a case can set
/// them without a duplicate node.
fn config(glass: &str, response: &str, top: &str) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        {top}
        layout {{
            gaps 40
            center-focused-column "always"
            background-color "{BACKDROP}"
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        material "edge" {{
            glass {{
                {glass}
                backdrop-blur false
            }}
            response "default" {{
                accent "none"
                {response}
            }}
        }}
        window-rule {{
            material "edge"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        "##
    ))
    .unwrap()
}

fn open(f: &mut Fixture, id: ClientId, (w, h): (u16, u16), content: u32) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(content);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

fn fixture(config: Config) -> Fixture {
    let mut f = Fixture::with_config(config);
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    f
}

/// Every tile's animated window rectangle, in logical px (scale 1).
fn window_rects(f: &mut Fixture) -> Vec<Rectangle<f64, Logical>> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    workspace
        .tiles_with_render_positions()
        .map(|(tile, pos, _)| Rectangle::new(pos + tile.window_loc(), tile.animated_window_size()))
        .collect()
}

/// The face and slab at rest, from the window, as `slabSurface`'s comment
/// derives them: the face is the window narrowed on both axes by
/// 2 * max(|offset-x|, |offset-y|) and moved by the offset; the slab is the
/// face grown by the bevel. Integer bevels and offsets only: the code
/// applies ceil and round, which this does not repeat.
fn face_and_slab(look: Look, w: Rectangle<f64, Logical>) -> ([f64; 4], [f64; 4]) {
    let (ox, oy) = look.offset;
    let m = ox.abs().max(oy.abs());
    let face = [
        w.loc.x + m + ox,
        w.loc.y + m + oy,
        w.size.w - 2. * m,
        w.size.h - 2. * m,
    ];
    let b = look.bevel;
    let slab = [face[0] - b, face[1] - b, face[2] + 2. * b, face[3] + 2. * b];
    (face, slab)
}

fn renderer_name(f: &mut Fixture) -> String {
    f.niri_state()
        .backend
        .with_primary_renderer(|r| {
            r.with_context(|gl| unsafe {
                let name = gl.GetString(smithay::backend::renderer::gles::ffi::RENDERER);
                std::ffi::CStr::from_ptr(name.cast())
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap()
        })
        .unwrap()
}

struct Render {
    pixels: Vec<u8>,
    again: Vec<u8>,
    window: Rectangle<f64, Logical>,
    renderer: String,
}

/// `case` at rest, one focused window, rendered twice in one fixture.
fn render_case(case: &Case) -> Render {
    let glass = format!("{}\n{}", case.look.glass, case.extra);
    let mut f = fixture(config(&glass, case.response, ""));
    let id = f.add_client();
    open(&mut f, id, (W, H), case.content);
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    f.double_roundtrip(id);
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    let pixels = render_at(&mut f, REST);
    let again = render_at(&mut f, REST);
    let window = window_rects(&mut f)[0];
    let renderer = renderer_name(&mut f);
    Render {
        pixels,
        again,
        window,
        renderer,
    }
}

fn rect_json(r: [f64; 4]) -> String {
    format!("[{}, {}, {}, {}]", r[0], r[1], r[2], r[3])
}

fn dump(
    name: &str,
    pixels: &[u8],
    look: Option<Look>,
    window: Rectangle<f64, Logical>,
    renderer: &str,
) {
    let Some(dir) = std::env::var_os("GLASS_EDGE_DUMP") else {
        return;
    };
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(format!("{name}.rgba")), pixels).unwrap();
    let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
    let win = [window.loc.x, window.loc.y, window.size.w, window.size.h];
    let regions = look.map_or(String::new(), |look| {
        let (face, slab) = face_and_slab(look, window);
        format!(
            r#", "face": {}, "slab": {}"#,
            rect_json(face),
            rect_json(slab)
        )
    });
    let json = format!(
        r#"{{"size": [{OUT_W}, {OUT_H}], "window": {}{regions}, "renderer": {renderer:?}}}"#,
        rect_json(win)
    );
    std::fs::write(dir.join(format!("{name}.json")), json).unwrap();
}

#[test]
fn every_case_renders_frozen() {
    for case in cases() {
        let r = render_case(&case);
        let (px, max) = diff(&r.pixels, &r.again);
        assert!(
            px == 0,
            "{}: a repeat render differs in {px} px, max {max}",
            case.name
        );
        dump(
            &case.name,
            &r.pixels,
            Some(case.look),
            r.window,
            &r.renderer,
        );
    }
}

const MOTION: &str = r#"animations {
            window-resize { duration-ms 1000; curve "linear"; }
            horizontal-view-movement { duration-ms 1000; curve "linear"; }
        }"#;

/// The live look at full flex with profile `k`.
fn motion_glass(k: f64, distortion: &str) -> String {
    format!(
        "{}\nbevel-profile {k}\njelly-flex 0.02\n{distortion}",
        LIVE.glass
    )
}

/// A glass-parameter reload: it keeps the material state, its seed and any
/// running animation (see ring_pair.rs), so the next render is the same
/// instant under the new glass.
fn reload(f: &mut Fixture, config: Config) {
    f.niri_state().reload_config(Ok(config));
    f.niri_state().refresh_and_flush_clients();
}

fn px(pixels: &[u8], x: i32, y: i32) -> Option<[u8; 3]> {
    if x < 0 || y < 0 || x >= i32::from(OUT_W) || y >= i32::from(OUT_H) {
        return None;
    }
    let i = ((y * i32::from(OUT_W) + x) * 4) as usize;
    Some([pixels[i], pixels[i + 1], pixels[i + 2]])
}

/// Per side of `rect` whose middle lies on the output, scanning outward from
/// 20 px inside the window along its middle row or column to the first three
/// backdrop pixels: the rim ratio, the rise of the last pixel before the
/// anti-aliased one over the mean rise of the four pixels before it, in summed
/// RGB. A planar facet rises about linearly (at most 1.85 at rest on the live
/// look, 2.67 under ±6 px of jelly); a rounded profile climbs into the rim (the
/// model gives about 4.3 at rest; the render measures 4.15 to 5.25, still above
/// the gate of 3). A flat run before the outline gives 0.
fn rim_ratios(tag: &str, pixels: &[u8], rect: Rectangle<f64, Logical>) -> Vec<(&'static str, f64)> {
    let backdrop = px(pixels, 2, 2).unwrap();
    let cx = (rect.loc.x + rect.size.w / 2.) as i32;
    let cy = (rect.loc.y + rect.size.h / 2.) as i32;
    let sides = [
        ("left", (rect.loc.x as i32 + 20, cy), (-1, 0)),
        (
            "right",
            ((rect.loc.x + rect.size.w) as i32 - 20, cy),
            (1, 0),
        ),
        ("top", (cx, rect.loc.y as i32 + 20), (0, -1)),
        (
            "bottom",
            (cx, (rect.loc.y + rect.size.h) as i32 - 20),
            (0, 1),
        ),
    ];
    let mut ratios = Vec::new();
    for (side, (x0, y0), (dx, dy)) in sides {
        let at = |i: i32| px(pixels, x0 + dx * i, y0 + dy * i);
        if at(0).is_none() {
            continue;
        }
        let outline = (0..)
            .take_while(|&i| at(i + 2).is_some())
            .find(|&i| (0..3).all(|j| at(i + j) == Some(backdrop)))
            .unwrap_or_else(|| panic!("{tag} {side}: no outline before the output's edge"));
        let sum = |i: i32| at(i).unwrap().iter().map(|&c| f64::from(c)).sum::<f64>();
        let p: Vec<f64> = (0..6).map(|i| sum(outline - 2 - i)).collect();
        ratios.push((side, (p[0] - p[1]) / ((p[1] - p[5]).abs() / 4.).max(0.5)));
    }
    ratios
}

/// Renders `at` under profile 2, then under a forced profile 1 at the same
/// instant, and returns both renders and the window rectangles.
fn both_profiles(
    f: &mut Fixture,
    distortion: &str,
    at: Duration,
) -> (Vec<u8>, Vec<u8>, Vec<Rectangle<f64, Logical>>) {
    let rounded = render_at(f, at);
    let rects = window_rects(f);
    reload(f, config(&motion_glass(1., distortion), RING_OFF, MOTION));
    let planar = render_at(f, at);
    assert_eq!(
        window_rects(f),
        rects,
        "the reload moved the frozen instant"
    );
    reload(f, config(&motion_glass(2., distortion), RING_OFF, MOTION));
    (rounded, planar, rects)
}

#[test]
fn the_rounded_rim_is_rounded_at_rest_mid_resize_and_mid_scroll() {
    for (tag, distortion) in [("plain", ""), ("distorted", "distortion 1 scale=0.5")] {
        // At rest: an absolute check on all four sides, with the forced
        // planar render as its negative control.
        let mut f = fixture(config(&motion_glass(2., distortion), RING_OFF, MOTION));
        let id = f.add_client();
        let surface = open(&mut f, id, (W, H), CLEAR);
        f.niri_state().update_keyboard_focus();
        f.double_roundtrip(id);
        set_time(&mut f, Duration::ZERO);
        f.niri_complete_animations();
        let (rounded, planar, rects) = both_profiles(&mut f, distortion, Duration::ZERO);
        dump(
            &format!("profile-{tag}-rest-k2"),
            &rounded,
            None,
            rects[0],
            "",
        );
        dump(
            &format!("profile-{tag}-rest-k1"),
            &planar,
            None,
            rects[0],
            "",
        );
        let k2 = rim_ratios("rest k2", &rounded, rects[0]);
        let k1 = rim_ratios("rest k1", &planar, rects[0]);
        eprintln!("{tag} rest k2 {k2:?} k1 {k1:?}");
        assert_eq!(k2.len(), 4, "rest: sides {k2:?}");
        for (side, r) in &k2 {
            assert!(
                *r >= 3.,
                "{tag} rest {side}: rim ratio {r} is not rounded ({k2:?})"
            );
        }
        for (side, r) in &k1 {
            assert!(
                *r < 3.,
                "{tag} rest {side}: the forced planar bevel reads rounded, ratio {r}"
            );
        }

        // Mid-resize: the column 200 px wider, half way through. Paired at
        // one instant: the rounded rim must out-climb the planar one on every
        // side (at least 1.56 times under ±6 px of jelly in the model).
        f.niri()
            .layout
            .set_column_width(niri_ipc::SizeChange::AdjustFixed(200));
        f.double_roundtrip(id);
        let window = f.client(id).window(&surface);
        window.attach_new_shm_buffer(CLEAR);
        window.set_size(W + 200, H);
        window.ack_last_and_commit();
        f.roundtrip(id);
        let mid = Duration::from_millis(500);
        let (rounded, planar, rects) = both_profiles(&mut f, distortion, mid);
        dump(
            &format!("profile-{tag}-resize-k2"),
            &rounded,
            None,
            rects[0],
            "",
        );
        dump(
            &format!("profile-{tag}-resize-k1"),
            &planar,
            None,
            rects[0],
            "",
        );
        let k2 = rim_ratios("resize k2", &rounded, rects[0]);
        let k1 = rim_ratios("resize k1", &planar, rects[0]);
        eprintln!("{tag} resize k2 {k2:?} k1 {k1:?}");
        assert_eq!(k2.len(), 4, "mid-resize: sides {k2:?}");
        for ((side, r2), (_, r1)) in k2.iter().zip(&k1) {
            assert!(
                *r2 >= 1.3 * r1.max(1.),
                "{tag} mid-resize {side}: k2 {r2} vs k1 {r1}"
            );
        }

        // Mid-scroll: two 800 px columns; focusing the second scrolls the
        // view, so at 500 ms the first tile's right edge trails and the
        // second's left edge leads, both on the output.
        let mut f = fixture(config(&motion_glass(2., distortion), RING_OFF, MOTION));
        let id = f.add_client();
        let first = open(&mut f, id, (W, H), CLEAR);
        let second = open(&mut f, id, (W, H), CLEAR);
        // The newest window has focus: widen its column, then the first's,
        // each client committing the new width as mid_resize_fixture does.
        for surface in [&second, &first] {
            f.niri()
                .layout
                .set_column_width(niri_ipc::SizeChange::SetFixed(800));
            f.double_roundtrip(id);
            let window = f.client(id).window(surface);
            window.attach_new_shm_buffer(CLEAR);
            window.set_size(800, H);
            window.ack_last_and_commit();
            f.double_roundtrip(id);
            f.niri().layout.focus_left();
        }
        f.niri_state().update_keyboard_focus();
        f.double_roundtrip(id);
        set_time(&mut f, Duration::ZERO);
        f.niri_complete_animations();
        let _ = render_at(&mut f, Duration::ZERO);
        f.niri().layout.focus_right();
        let (rounded, planar, rects) = both_profiles(&mut f, distortion, mid);
        let mut sides = Vec::new();
        for (i, rect) in rects.iter().enumerate() {
            dump(
                &format!("profile-{tag}-scroll-{i}-k2"),
                &rounded,
                None,
                *rect,
                "",
            );
            let k2 = rim_ratios("scroll k2", &rounded, *rect);
            let k1 = rim_ratios("scroll k1", &planar, *rect);
            eprintln!("{tag} scroll tile {i} k2 {k2:?} k1 {k1:?}");
            for ((side, r2), (_, r1)) in k2.iter().zip(&k1) {
                assert!(
                    *r2 >= 1.3 * r1.max(1.),
                    "{tag} mid-scroll tile {i} {side}: k2 {r2} vs k1 {r1}"
                );
                sides.push(*side);
            }
        }
        assert!(
            sides.contains(&"left") && sides.contains(&"right"),
            "mid-scroll checked only {sides:?}: no leading and trailing edge on the output"
        );
    }
}

/// A 16 px checker of a warm and a cool colour (opaque ARGB).
fn checker(x: u16, y: u16) -> u32 {
    if (x / 16 + y / 16) % 2 == 0 {
        0xffc8_783c
    } else {
        0xff28_5ac8
    }
}

/// A Background-layer surface covering the output with `checker`: the glass's
/// background buffer, so what the reflection samples depends on where it looks.
fn patterned_backdrop(f: &mut Fixture, id: ClientId) {
    let layer = f
        .client(id)
        .create_layer(None, Layer::Background, "glass-edge-checker");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Left | Anchor::Right | Anchor::Top | Anchor::Bottom),
        size: Some((0, 0)),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(id);
    let layer = f.client(id).layer(&surface);
    layer.attach_new_shm_pattern(OUT_W, OUT_H, checker);
    layer.ack_last_and_commit();
    f.double_roundtrip(id);
}

/// Linear light from an 8-bit sRGB value.
fn lin(c: u8) -> f64 {
    let c = f64::from(c) / 255.;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Channels whose reflection term changes by more than 0.02 (linear) when the
/// perturbation is switched on: the reflection follows the perturbed
/// direction. Calibrated in Task 7 Step 6: 364 real, 0 mutated, so the geometric
/// mean of 364 and max(0, 4) is 38.
const REFLECTION_MOTION_MIN: usize = 38;

#[test]
fn the_reflection_follows_the_perturbed_direction_in_motion() {
    // Reflection on, rounded bevel, at full flex (edge-highlight parses only
    // from Task 8, and cancels out of the isolate anyway). Perturbed:
    // distortion and jelly ripple; flat: neither.
    let glass = |reflection: f64, perturbed: bool| {
        let perturbation = if perturbed {
            "distortion 0.4 scale=1\njelly-ripple 0.5"
        } else {
            "jelly-ripple 0"
        };
        format!(
            "{}\nbevel-profile 2\njelly-flex 0.02\nreflection {reflection}\n{perturbation}",
            LIVE.glass
        )
    };
    let mut f = fixture(config(&glass(0.6, true), RING_OFF, MOTION));
    let id = f.add_client();
    patterned_backdrop(&mut f, id);
    let surface = open(&mut f, id, (W, H), CLEAR);
    f.niri_state().update_keyboard_focus();
    f.double_roundtrip(id);
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    let _ = render_at(&mut f, Duration::ZERO);
    f.niri()
        .layout
        .set_column_width(niri_ipc::SizeChange::AdjustFixed(200));
    f.double_roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(CLEAR);
    window.set_size(W + 200, H);
    window.ack_last_and_commit();
    f.roundtrip(id);

    // Four renders of one frozen mid-resize instant. In linear light the
    // reflection is additive, so (on - off) isolates it under each
    // perturbation state; if its direction ignored the perturbation, the two
    // isolates would agree to quantization.
    // The window rectangle is read after each render: before the first one
    // the clock still stands at the resize's start.
    let mid = Duration::from_millis(500);
    let mut rect = None;
    let mut renders = Vec::new();
    for (reflection, perturbed) in [(0.6, true), (0., true), (0.6, false), (0., false)] {
        reload(
            &mut f,
            config(&glass(reflection, perturbed), RING_OFF, MOTION),
        );
        let pixels = render_at(&mut f, mid);
        let now = window_rects(&mut f)[0];
        assert_eq!(
            *rect.get_or_insert(now),
            now,
            "the reload moved the frozen instant"
        );
        dump(
            &format!("reflection-motion-{reflection}-{perturbed}"),
            &pixels,
            None,
            now,
            "",
        );
        renders.push(pixels);
    }
    let mut clipped = 0;
    let mut changed = 0;
    for i in (0..renders[0].len()).filter(|i| i % 4 != 3) {
        // A channel at 255 in any render is clipped: its linear sum is not
        // recoverable, so it says nothing either way.
        if renders.iter().any(|r| r[i] == 255) {
            clipped += 1;
            continue;
        }
        let perturbed = lin(renders[0][i]) - lin(renders[1][i]);
        let flat = lin(renders[2][i]) - lin(renders[3][i]);
        if (perturbed - flat).abs() > 0.02 {
            changed += 1;
        }
    }
    eprintln!("reflection motion: {changed} channels changed, {clipped} clipped");
    assert!(
        changed >= REFLECTION_MOTION_MIN,
        "the reflection ignores the perturbation: {changed} channels changed ({clipped} clipped)"
    );
}
