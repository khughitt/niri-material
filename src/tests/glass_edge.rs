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
use smithay::utils::{Logical, Rectangle};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
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
