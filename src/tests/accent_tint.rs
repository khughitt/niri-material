//! The accent tint (docs/specs/2026-10-03-accent-tint-design.md §8): what
//! must hold for any weight. The look is judged by eye on the dumps
//! (`accent_tint_dumps`, plan Task 6).

use std::time::Duration;

use niri_config::Config;
use smithay::backend::renderer::element::Element as _;
use smithay::utils::{Logical, Rectangle};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::ring_pair::{diff, render_at, set_time, window_rect};
use super::*;
use crate::niri::SetWindowSignalArgs;
use crate::render_helpers::{RenderCtx, RenderTarget};

pub(super) const OUT_W: u16 = 1280;
pub(super) const OUT_H: u16 = 800;

/// A dark fill at 0.6 opacity (premultiplied), kitty-like: the glass shows through.
pub(super) const TRANSLUCENT: u32 = 0x990a_0c0e;
/// The same fill, opaque: every pixel bypasses the glass.
const OPAQUE: u32 = 0xff0a_0c0e;

/// The terminal-glass `glass` block of the owner's accepted look, verbatim
/// from `ring_look.rs` `ACCEPTED`.
pub(super) const ACCEPTED_GLASS: &str = r##"
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
"##;
/// Its `response "default"` block without `accent`, which callers add with
/// `accent-tint`: the band and filament the owner accepted.
pub(super) const ACCEPTED_RESPONSE: &str = r##"
        focus "ring-light"
        ring-color "#ccccff"
        ring-beam-speed 4350
        ring-beam-noise 0.55
        ring-beam-noise-hz 12
        ring-beam-decay 4150
        ring-gap 6
        ring-width 1.1
        ring-glow 1.2
"##;
/// Every glass parameter at its default.
pub(super) const DEFAULT_GLASS: &str = "";

/// The crossfade, made linear so a time fraction is a crossfade fraction.
const FADE_MS: u64 = 400;
/// Well after any fade.
const REST: Duration = Duration::from_secs(4);
const REST_LATER: Duration = Duration::from_secs(5);

pub(super) fn look(glass: &str, response: &str, background: &str, extra: &str) -> Config {
    look_animated(glass, response, background, "", extra)
}

/// `look` with `animations` children ahead of the linear crossfade: one
/// config holds a single `animations` node, so `off` cannot go in `extra`.
fn look_animated(
    glass: &str,
    response: &str,
    background: &str,
    animations: &str,
    extra: &str,
) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        animations {{ {animations} material-signal {{ duration-ms {FADE_MS}; curve "linear"; }}; }}
        layout {{
            gaps 64
            background-color "{background}"
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        material "tg" {{
            glass {{
                {glass}
            }}
            response "default" {{
                {response}
            }}
        }}
        window-rule {{ material "tg"; }}
        {extra}
        "##
    ))
    .unwrap()
}

pub(super) fn open(f: &mut Fixture, id: ClientId, (w, h): (u16, u16), argb: u32) -> WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(argb);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

/// Sets (`Some`) or clears (`None`) the accent of the `index`th window.
pub(super) fn signal(f: &mut Fixture, index: usize, accent: Option<&str>) {
    use niri_ipc::{SignalLevel, SignalMotion};

    let id = f.niri().layout.windows().nth(index).unwrap().1.id().get();
    match accent {
        Some(accent) => f
            .niri()
            .set_window_signal(SetWindowSignalArgs {
                id,
                source: String::from("t"),
                accent: Some(String::from(accent)),
                level: SignalLevel::Active,
                motion: SignalMotion::Static,
                tag: None,
                ttl_ms: None,
                after_level: None,
                after_motion: None,
                until_focus: false,
            })
            .unwrap(),
        None => f.niri().clear_window_signal(id, "t").unwrap(),
    }
    f.niri_state().refresh_and_flush_clients();
}

/// One 800×500 window filled with `argb` under `config`, settled,
/// optionally with an accent; returns the fixture and its render at rest.
fn one_window(config: Config, argb: u32, accent: Option<&str>) -> (Fixture, Vec<u8>) {
    let mut f = Fixture::with_config(config);
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    let id = f.add_client();
    open(&mut f, id, (800, 500), argb);
    set_time(&mut f, Duration::ZERO);
    if let Some(accent) = accent {
        signal(&mut f, 0, Some(accent));
    }
    f.niri_complete_animations();
    // The tile starts the signal's crossfade on the first frame that sees it.
    let _ = render_at(&mut f, REST);
    let pixels = render_at(&mut f, REST_LATER);
    (f, pixels)
}

fn rest(glass: &str, response: &str, argb: u32, accent: Option<&str>) -> Vec<u8> {
    one_window(look(glass, response, "#202020", ""), argb, accent).1
}

/// The pixels of `rect` inset by `inset`, row by row.
fn region(pixels: &[u8], rect: Rectangle<f64, Logical>, inset: f64) -> Vec<u8> {
    let x0 = (rect.loc.x + inset).ceil() as usize;
    let y0 = (rect.loc.y + inset).ceil() as usize;
    let x1 = (rect.loc.x + rect.size.w - inset).floor() as usize;
    let y1 = (rect.loc.y + rect.size.h - inset).floor() as usize;
    let stride = usize::from(OUT_W) * 4;
    (y0..y1)
        .flat_map(|y| {
            pixels[y * stride + x0 * 4..y * stride + x1 * 4]
                .iter()
                .copied()
        })
        .collect()
}

/// The `index`th window's rect on the output, in logical px (scale 1). By
/// window, not by tile order: render order puts the active tile first.
fn rect_of(f: &mut Fixture, index: usize) -> Rectangle<f64, Logical> {
    let niri = f.niri();
    let id = niri.layout.windows().nth(index).unwrap().1.id().get();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().id().get() == id)
        .unwrap();
    Rectangle::new(pos + tile.window_loc(), tile.animated_window_size())
}

/// An output pass as the real loop renders it (`RenderTarget::Output`): the
/// first tile's material element as `id@commit`, and whether a signal timer
/// was armed. The id is read each pass, since a reload may rebuild the state.
fn output_pass(f: &mut Fixture, time: Duration) -> (String, bool) {
    set_time(f, time);
    f.niri().advance_animations();
    let output = f.niri_output(1);
    let material = {
        let niri = f.niri();
        let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
        let (tile, _, _) = workspace.tiles_with_render_positions().next().unwrap();
        tile.material().expect("a material tile").id().clone()
    };
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    let element = backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target: RenderTarget::Output,
                xray: None,
                signal_ticks: None,
            };
            niri.render_to_vec(ctx, &output, false)
                .iter()
                .find(|e| *e.id() == material)
                .map(|e| format!("{:?}@{:?}", e.id(), e.current_commit()))
                .expect("the material element is rendered")
        })
        .unwrap();
    niri.arm_signal_timer(&output);
    (element, niri.output_state[&output].signal_timer.is_some())
}

#[test]
fn completing_animations_keeps_the_configured_clock_setting() {
    for (animations, off) in [("", false), ("off;", true)] {
        let mut f =
            Fixture::with_config(look_animated(DEFAULT_GLASS, "", "#202020", animations, ""));
        assert_eq!(
            f.niri().clock.should_complete_instantly(),
            off,
            "{animations:?}: config sets it"
        );
        f.niri_complete_animations();
        assert_eq!(
            f.niri().clock.should_complete_instantly(),
            off,
            "{animations:?}: the helper restores it"
        );
    }
}

#[test]
fn zero_weight_and_default_render_identically() {
    for glass in [ACCEPTED_GLASS, DEFAULT_GLASS] {
        let absent = rest(glass, r#"accent "ring""#, TRANSLUCENT, Some("#ff6600"));
        let zero = rest(
            glass,
            "accent \"ring\"\n accent-tint 0",
            TRANSLUCENT,
            Some("#ff6600"),
        );
        assert_eq!(diff(&absent, &zero), (0, 0), "default vs explicit 0");

        let full_quiet = rest(glass, "accent-tint 1", TRANSLUCENT, None);
        let zero_quiet = rest(glass, "accent-tint 0", TRANSLUCENT, None);
        assert_eq!(diff(&full_quiet, &zero_quiet), (0, 0), "no accent, no tint");
    }
}

#[test]
fn opaque_pixels_are_untouched() {
    let (mut f, tinted) = one_window(
        look(ACCEPTED_GLASS, "accent-tint 1", "#202020", ""),
        OPAQUE,
        Some("#ff00ff"),
    );
    let rect = window_rect(&mut f);
    let plain = rest(ACCEPTED_GLASS, "accent-tint 0", OPAQUE, Some("#ff00ff"));
    assert_eq!(region(&tinted, rect, 16.), region(&plain, rect, 16.));
}

#[test]
fn full_weight_tints_the_slab_and_translucent_content() {
    let tinted = rest(
        ACCEPTED_GLASS,
        "accent-tint 1",
        TRANSLUCENT,
        Some("#ff6600"),
    );
    let plain = rest(
        ACCEPTED_GLASS,
        "accent-tint 0",
        TRANSLUCENT,
        Some("#ff6600"),
    );
    assert!(diff(&tinted, &plain).0 > 0, "no visible tint");
}

#[test]
fn accent_none_still_tints_the_body() {
    let tinted = rest(
        ACCEPTED_GLASS,
        "accent \"none\"\n accent-tint 1",
        TRANSLUCENT,
        Some("#ff6600"),
    );
    let plain = rest(
        ACCEPTED_GLASS,
        "accent \"none\"\n accent-tint 0",
        TRANSLUCENT,
        Some("#ff6600"),
    );
    assert!(
        diff(&tinted, &plain).0 > 0,
        "accent \"none\" removed the body tint"
    );
}

#[test]
fn removal_returns_to_the_never_tinted_render() {
    let config = || look(ACCEPTED_GLASS, "accent-tint 1", "#202020", "");
    let (mut f, _) = one_window(config(), TRANSLUCENT, Some("#ff6600"));
    signal(&mut f, 0, None);
    let start = REST_LATER + Duration::from_secs(1);
    let _ = render_at(&mut f, start); // starts the fade out
    let _ = render_at(&mut f, start + Duration::from_millis(FADE_MS / 2));
    let _ = render_at(&mut f, start + Duration::from_millis(FADE_MS + 100));
    let after = render_at(&mut f, start + Duration::from_secs(2));

    let (_, never) = one_window(config(), TRANSLUCENT, None);
    assert_eq!(diff(&after, &never), (0, 0));
}

#[test]
fn settled_tint_neither_commits_nor_animates_nor_arms_a_timer() {
    let (mut f, _) = one_window(
        look(ACCEPTED_GLASS, "accent-tint 1", "#202020", ""),
        TRANSLUCENT,
        Some("#ff00ff"),
    );
    // The first Output pass primes that target's fingerprint.
    let (first, _) = output_pass(&mut f, REST_LATER + Duration::from_secs(1));
    let (second, armed) = output_pass(&mut f, REST_LATER + Duration::from_secs(2));
    assert_eq!(first, second, "a settled tint commits damage");
    assert!(!armed, "a settled tint arms a signal timer");
    assert!(!f.niri().layout.are_animations_ongoing(None));
    let output = f.niri_output(1);
    let niri = f.niri();
    let monitor = niri.layout.monitor_for_output(&output).unwrap();
    assert!(
        !monitor.are_transitions_ongoing(),
        "a settled tint keeps a transition going"
    );
}

#[test]
fn animations_off_tints_the_first_frame() {
    let config = |w: &str, animations: &str| {
        look_animated(
            ACCEPTED_GLASS,
            &format!("accent-tint {w}"),
            "#202020",
            animations,
            "",
        )
    };
    const OFF: &str = "off;";
    // A settled window with no accent; the accent arrives, and the next frame
    // renders at the same instant: no time advanced, no animation completed.
    let first_frame = |config: Config| {
        let (mut f, _) = one_window(config, TRANSLUCENT, None);
        signal(&mut f, 0, Some("#ff6600"));
        let pixels = render_at(&mut f, REST_LATER);
        (pixels, f.niri().layout.are_animations_ongoing(None))
    };

    let (off, off_ongoing) = first_frame(config("1", OFF));
    let (_, settled) = one_window(config("1", OFF), TRANSLUCENT, Some("#ff6600"));
    let (_, untinted) = one_window(config("0", OFF), TRANSLUCENT, Some("#ff6600"));
    assert_eq!(
        diff(&off, &settled),
        (0, 0),
        "the first frame is the settled tint"
    );
    assert!(diff(&off, &untinted).0 > 0, "and it is tinted");
    assert!(!off_ongoing, "nothing left animating");

    // Control: with the 400 ms crossfade the same first frame is still at
    // fraction 0, untinted, so this test can tell a running fade apart.
    let (on, on_ongoing) = first_frame(config("1", ""));
    let (_, quiet) = one_window(config("0", ""), TRANSLUCENT, None);
    assert_eq!(diff(&on, &quiet), (0, 0), "a running fade starts untinted");
    assert!(on_ongoing, "the control fade is running");
}

#[test]
fn reload_of_only_the_weight_commits_damage_and_rerenders() {
    let (mut f, before) = one_window(
        look(ACCEPTED_GLASS, "accent-tint 0.4", "#202020", ""),
        TRANSLUCENT,
        Some("#ff6600"),
    );
    let t = REST_LATER + Duration::from_secs(1);
    let s = Duration::from_secs;
    let (primed, _) = output_pass(&mut f, t);
    assert_eq!(
        output_pass(&mut f, t + s(1)).0,
        primed,
        "settled before the reload"
    );

    f.niri_state()
        .reload_config(Ok(look(ACCEPTED_GLASS, "accent-tint 0.6", "#202020", "")));
    f.niri_state().refresh_and_flush_clients();
    let (reloaded, _) = output_pass(&mut f, t + s(2));
    assert_ne!(
        reloaded, primed,
        "the weight-only reload committed no damage on the output"
    );
    assert_eq!(
        output_pass(&mut f, t + s(3)).0,
        reloaded,
        "and then it settles"
    );

    let after = render_at(&mut f, t + s(4));
    assert!(diff(&before, &after).0 > 0, "the new weight never rendered");
}

#[test]
fn neighbor_accent_does_not_reach_a_window() {
    // Every other accent path is off (band, focus light, attention glint):
    // only the body tint can differ. A mid-gray backdrop: over near-black the
    // luminance-keeping tint rounds away at 8 bits.
    let scene = |neighbor: &str| {
        let mut f = Fixture::with_config(look(
            ACCEPTED_GLASS,
            "accent \"none\"\n focus \"none\"\n attention \"none\"\n accent-tint 1",
            "#808080",
            "",
        ));
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (500, 400), TRANSLUCENT);
        open(&mut f, id, (500, 400), TRANSLUCENT);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, Some("#ff6600"));
        signal(&mut f, 1, Some(neighbor));
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        let pixels = render_at(&mut f, REST_LATER);
        [0, 1].map(|index| region(&pixels, rect_of(&mut f, index), 24.))
    };
    let [own_blue, neighbor_blue] = scene("#0066ff");
    let [own_magenta, neighbor_magenta] = scene("#ff00ff");
    assert_eq!(
        own_blue, own_magenta,
        "the neighbor's accent reached window 0"
    );
    assert_ne!(
        neighbor_blue, neighbor_magenta,
        "the neighbor tints by its own accent"
    );
}

#[test]
fn focus_swap_retints_on_the_first_frame() {
    // Focused windows take "on", unfocused "off"; "on" is tinted or not.
    let split = |on_weight: &str| {
        Config::parse_mem(&format!(
            r##"
            hotkey-overlay {{ skip-at-startup; }}
            layout {{ gaps 64; background-color "#202020"; focus-ring {{ off; }}; border {{ off; }}; shadow {{ off; }}; }}
            material "on" {{ glass {{ {ACCEPTED_GLASS} }}; response "default" {{ accent-tint {on_weight}; }}; }}
            material "off" {{ glass {{ {ACCEPTED_GLASS} }}; response "default" {{ accent-tint 0; }}; }}
            window-rule {{ match is-active=true; material "on"; }}
            window-rule {{ match is-active=false; material "off"; }}
            "##
        ))
        .unwrap()
    };
    let refocus = |f: &mut Fixture, left: bool| {
        if left {
            f.niri().layout.focus_left();
        } else {
            f.niri().layout.focus_right();
        }
        // Keyboard focus, then the rule recompute that re-resolves tile
        // materials (`State::refresh` order, as `focus_swap.rs` drives it).
        f.niri_state().refresh_and_flush_clients();
    };
    // One fixture: window 0 focused with its accent settled, then focus away
    // and back. Each frame renders at the same instant, so it is the first
    // frame after the swap.
    let sequence = |config: Config| {
        let mut f = Fixture::with_config(config);
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (500, 400), TRANSLUCENT);
        open(&mut f, id, (500, 400), TRANSLUCENT);
        refocus(&mut f, true);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, Some("#ff6600"));
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        let mut frames = Vec::new();
        for step in [None, Some(false), Some(true)] {
            if let Some(left) = step {
                refocus(&mut f, left);
            }
            let pixels = render_at(&mut f, REST_LATER);
            frames.push(region(&pixels, rect_of(&mut f, 0), 24.));
        }
        frames
    };
    let tinted = sequence(split("1"));
    let plain = sequence(split("0"));
    assert_ne!(
        tinted[0], plain[0],
        "focused, settled: the on material tints"
    );
    assert_eq!(
        tinted[1], plain[1],
        "first frame after losing focus: no stale tint"
    );
    assert_ne!(
        tinted[2], plain[2],
        "first frame after regaining focus: tinted again"
    );
}

fn dump(dir: &std::path::Path, name: &str, pixels: &[u8]) {
    let file = std::fs::File::create(dir.join(format!("{name}.png"))).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
}

/// The owner-review series (spec §8). Ignored by default; run with
/// `ACCENT_TINT_DUMP=<dir> just test-one -p niri accent_tint_dumps --run-ignored only`.
#[test]
#[ignore = "writes owner-review PNGs; set ACCENT_TINT_DUMP"]
fn accent_tint_dumps() {
    let dir = std::path::PathBuf::from(
        std::env::var_os("ACCENT_TINT_DUMP").expect("set ACCENT_TINT_DUMP=<dir>"),
    );
    std::fs::create_dir_all(&dir).unwrap();

    // (name, glass block, response settings): the accepted look keeps its own
    // band and filament; the default glass keeps the default response.
    let glasses = [
        ("accepted", ACCEPTED_GLASS, ACCEPTED_RESPONSE),
        ("default", DEFAULT_GLASS, ""),
    ];
    let accents = [
        ("orange", "#ff6600"),
        ("blue", "#0066ff"),
        ("magenta", "#ff00ff"),
    ];
    let weights = ["0", "0.25", "0.5", "1"];
    // Focused, so the ring band and its focus light are in view.
    let still = |glass: &str, response: &str, background: &str, accent: &str| {
        let mut f = Fixture::with_config(look(glass, response, background, ""));
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (800, 500), TRANSLUCENT);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, Some(accent));
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        render_at(&mut f, REST_LATER)
    };

    for (gname, glass, base) in glasses {
        for (aname, accent) in accents {
            for w in weights {
                for (mode, selector) in [("ring", "ring"), ("none", "none")] {
                    let response = format!("{base}\n accent \"{selector}\"\n accent-tint {w}");
                    let name = format!("still-{gname}-{aname}-w{w}-accent-{mode}");
                    dump(&dir, &name, &still(glass, &response, "#808080", accent));
                }
                for (bname, background) in [("red", "#ff0000"), ("blue", "#0000ff")] {
                    let response = format!("{base}\n accent-tint {w}");
                    let name = format!("backdrop-{gname}-{aname}-w{w}-{bname}");
                    dump(&dir, &name, &still(glass, &response, background, accent));
                }
            }
        }
    }

    // Fades at time fractions 0, ¼, ½, ¾, 1 of the linear crossfade.
    for (fname, from, to) in [
        ("orange-to-blue", Some("#ff6600"), Some("#0066ff")),
        ("black-to-orange", Some("#000000"), Some("#ff6600")),
        ("orange-to-black", Some("#ff6600"), Some("#000000")),
        ("orange-to-none", Some("#ff6600"), None),
    ] {
        let response = format!("{ACCEPTED_RESPONSE}\n accent-tint 1");
        let mut f = Fixture::with_config(look(ACCEPTED_GLASS, &response, "#808080", ""));
        f.niri_state().backend.headless().add_renderer().unwrap();
        f.add_output(1, (OUT_W, OUT_H));
        let id = f.add_client();
        open(&mut f, id, (800, 500), TRANSLUCENT);
        set_time(&mut f, Duration::ZERO);
        signal(&mut f, 0, from);
        f.niri_complete_animations();
        let _ = render_at(&mut f, REST);
        signal(&mut f, 0, to);
        let start = REST_LATER;
        for quarter in 0..=4u64 {
            let at = start + Duration::from_millis(FADE_MS * quarter / 4);
            dump(
                &dir,
                &format!("fade-{fname}-q{quarter}"),
                &render_at(&mut f, at),
            );
        }
    }
}
