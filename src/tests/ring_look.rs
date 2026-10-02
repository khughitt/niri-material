//! The owner's accepted ring look (material-519eeb), pinned as a reference.
//!
//! `ACCEPTED` is the material pair Prism generated on 2026-10-01, when the
//! owner accepted the ring beam with decay on a live a18ca619 session, copied
//! verbatim (the app-id match dropped: the test client sets none). Each pane
//! renders through a real focus gain from a second window: the inactive
//! material, then the comet at frozen instants after the gain, then rest. A large pane's perimeter is
//! longer than the decay distance, so its comet goes dark before the lap; a
//! small pane's is shorter, so its comet completes the lap.
//!
//! The test grades only what must hold for any look: renders are frozen, the
//! comet lights the pane, and rest is quiet. The look itself is judged by eye:
//! set `RING_LOOK_DUMP=<dir>` to write every render as a PNG, before and after
//! a change to ring or edge rendering. Update `ACCEPTED` only when the owner
//! accepts a new look.

use std::fmt::Write as _;
use std::time::Duration;

use niri_config::Config;
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::ring_pair::{diff, render_at, set_time};
use super::*;

const OUT_W: u16 = 2560;
const OUT_H: u16 = 1440;

/// The instant of the focus gain, and the beam's start.
const GAIN: Duration = Duration::from_secs(1);
/// Comet instants after the gain, ms. At 4350 px/s the 4150 px decay is dark
/// at about 954 ms.
const BEAM_MS: [u64; 4] = [0, 150, 400, 800];
/// Rest, long after any run, and one second later.
const REST: Duration = Duration::from_secs(4);
const REST_LATER: Duration = Duration::from_secs(5);

/// The window's content: kitty-like, a dark fill at 0.6 opacity
/// (premultiplied). Opaque pixels would bypass the slab.
const CONTENT: u32 = 0x990a_0c0e;

const ACCEPTED: &str = r##"
material "terminal-glass" {
    glass {
        ior 1.28
        light-ior 4.5
        thickness 31.2
        attenuation-color "#0D1D1E"
        attenuation-distance 11
        chromatic-aberration 0.36
        distortion 0 scale=0.09
        anisotropic-blur 0
        roughness 0.24
        iridescence 0
        aurora 0 {
            drift-hz 4
            color "#3dffb0"
            color "#7a5cff"
        }
        noise 0.03 type="white"
        saturation 0.95
        backdrop-blur true
        jelly-flex 0.0066
        jelly-ripple 0.23
        bevel 10
        offset-x -6
        offset-y -5
    }
    response "default" {
        accent "ring"
        focus "ring-light"
        ring-color "#ccccff"
        ring-beam-speed 4350
        ring-beam-noise 0.55
        ring-beam-noise-hz 12
        ring-beam-decay 4150
        ring-gap 6
        ring-width 1.1
        ring-glow 1.2
    }
}
material "terminal-glass-inactive" {
    glass {
        ior 1.22
        light-ior 4.5
        thickness 25.2
        attenuation-color "#131415"
        attenuation-distance 12
        chromatic-aberration 0.32
        distortion 0.18 scale=0.08
        anisotropic-blur 0
        roughness 0.04
        iridescence 0.01
        aurora 0 {
            drift-hz 4
            color "#3dffb0"
            color "#7a5cff"
        }
        noise 0.13 type="white"
        saturation 0.85
        backdrop-blur false
        jelly-flex 0.0066
        jelly-ripple 0.23
        bevel 10
        offset-x -6
        offset-y -5
    }
    response "default" {
        accent "ring"
        focus "ring-light"
        ring-color "#ccccff"
        ring-beam-speed 4350
        ring-beam-noise 0.55
        ring-beam-noise-hz 12
        ring-beam-decay 4150
        ring-gap 6
        ring-width 1.1
        ring-glow 1.2
    }
}
window-rule {
    match is-active=true
    material "terminal-glass"
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
window-rule {
    match is-active=false
    material "terminal-glass-inactive"
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
"##;

struct Pane {
    name: &'static str,
    w: u16,
    h: u16,
}

/// Perimeter about 5500 px, past the decay: dark before the lap.
const LARGE: Pane = Pane {
    name: "large",
    w: 1600,
    h: 1200,
};
/// Perimeter about 1500 px, inside the decay: the comet laps.
const SMALL: Pane = Pane {
    name: "small",
    w: 480,
    h: 300,
};

fn config(look: &str) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 31
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        {look}
        "##
    ))
    .unwrap()
}

fn dump(name: &str, pixels: &[u8]) {
    let Some(dir) = std::env::var_os("RING_LOOK_DUMP") else {
        return;
    };
    let path = std::path::Path::new(&dir).join(format!("{name}.png"));
    let file = std::fs::File::create(path).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
}

/// A second window, opened after the pane, that holds focus until the gain.
const OTHER: (u16, u16) = (400, 300);

fn open(f: &mut Fixture, id: ClientId, (w, h): (u16, u16)) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(CONTENT);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

/// Renders `pane` under `look` while another window holds focus, through
/// its focus gain, and at rest. Returns the report and (tag, render) in
/// order; dumps are named `<prefix><pane>-<tag>`.
fn sequence(pane: &Pane, look: &str, prefix: &str) -> (String, Vec<(String, Vec<u8>)>) {
    let mut f = Fixture::with_config(config(look));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    let id = f.add_client();
    open(&mut f, id, (pane.w, pane.h));
    open(&mut f, id, OTHER);
    // `is_focused` is cached and only `update_keyboard_focus` writes it; it
    // flags the rules for `refresh_window_rules` (see tests/material.rs).
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();

    let mut renders = Vec::new();
    renders.push(("inactive".to_owned(), render_at(&mut f, Duration::ZERO)));

    // The gain: focus moves to the pane. Both columns fit on the output, so
    // the view does not move. The tile stamps the beam's start from the clock
    // when its render elements next update, and a client dispatch would run
    // that on the unfrozen clock, so the first render after the gain, frozen
    // at `GAIN`, is where the beam starts.
    f.niri().layout.focus_left();
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_states();
    f.niri().refresh_layout();
    f.niri().refresh_window_rules();
    for ms in BEAM_MS {
        let pixels = render_at(&mut f, GAIN + Duration::from_millis(ms));
        renders.push((format!("beam-{ms:03}ms"), pixels));
    }
    renders.push(("rest".to_owned(), render_at(&mut f, REST)));
    renders.push(("rest-later".to_owned(), render_at(&mut f, REST_LATER)));

    let mut report = String::new();
    let rest = &renders[renders.len() - 2].1;
    for (tag, pixels) in &renders {
        dump(&format!("{prefix}{}-{tag}", pane.name), pixels);
        let (px, max) = diff(rest, pixels);
        let _ = writeln!(report, "{tag}: vs rest {px} px, max {max}");
    }
    (report, renders)
}

#[test]
fn accepted_ring_look() {
    let mut failures = Vec::new();
    for pane in [&LARGE, &SMALL] {
        let (report, renders) = sequence(pane, ACCEPTED, "");
        eprintln!("{} {}x{}:\n{report}", pane.name, pane.w, pane.h);
        let get = |tag: &str| &renders.iter().find(|(t, _)| t == tag).unwrap().1;
        let mut check = |ok: bool, what: &str| {
            if !ok {
                failures.push(format!("{}: {what}\n{report}", pane.name));
            }
        };
        check(
            diff(get("rest"), get("rest-later")).0 == 0,
            "rest is not quiet",
        );
        check(
            diff(get("rest"), get("beam-150ms")).0 > 0,
            "no comet 150 ms after the gain",
        );
        check(
            diff(get("rest"), get("inactive")).0 > 0,
            "the inactive material renders like the active one",
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `ACCEPTED` with every response's `ring-glow` line followed by `extra`.
fn with_response(extra: &str) -> String {
    ACCEPTED.replace(
        "ring-glow 1.2\n",
        &format!("ring-glow 1.2\n        {extra}\n"),
    )
}

/// `ring-rest 0` (material-1d70db) removes the resting ring and nothing
/// else: at rest the pane renders exactly as with no focus light, and the
/// comet still runs after the gain.
#[test]
fn ring_rest_zero_keeps_the_comet() {
    let (report, renders) = sequence(&LARGE, &with_response("ring-rest 0"), "rest0-");
    eprintln!("ring-rest 0:\n{report}");
    let get = |tag: &str| &renders.iter().find(|(t, _)| t == tag).unwrap().1;
    assert!(
        diff(get("rest"), get("beam-150ms")).0 > 0,
        "no comet\n{report}"
    );

    let unlit = ACCEPTED.replace(r#"focus "ring-light""#, r#"focus "none""#);
    let (_, dark) = sequence(&LARGE, &unlit, "unlit-");
    let dark_rest = &dark.iter().find(|(t, _)| t == "rest").unwrap().1;
    let (px, max) = diff(get("rest"), dark_rest);
    assert_eq!(
        (px, max),
        (0, 0),
        "ring-rest 0 at rest differs from no focus light"
    );

    let (_, lit) = sequence(&LARGE, ACCEPTED, "");
    let lit_rest = &lit.iter().find(|(t, _)| t == "rest").unwrap().1;
    assert!(
        diff(get("rest"), lit_rest).0 > 0,
        "ring-rest 0 left the resting ring"
    );
}
