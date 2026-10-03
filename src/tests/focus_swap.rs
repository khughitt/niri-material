//! What a focus-conditioned material swap keeps and what it replaces
//! (material-8e3b73). The live Prism config selects `terminal-glass` for the
//! active window and `terminal-glass-inactive` for the others; these tests
//! drive that rule pair through the real focus and window-rule path and
//! record the `MaterialState` lifetime next to a same-definition control.

use std::time::Duration;

use niri_config::animations::{Curve, EasingParams, Kind};
use niri_config::Config;
use smithay::backend::renderer::element::Id;
use smithay::utils::{Logical, Point};

use super::client::ClientId;
use super::*;
use crate::niri::Niri;
use crate::utils::with_toplevel_role;

/// The live pair's glass, pinned from the generated `prism.kdl` of
/// 2026-09-30. Both responses are identical, so only glass differs.
const LIVE_PAIR: &str = r##"
material "terminal-glass" {
    glass {
        ior 1.28
        light-ior 2.5
        thickness 31.2
        attenuation-color "#152F30"
        attenuation-distance 16
        chromatic-aberration 0.36
        distortion 0 scale=0.09
        roughness 0.24
        noise 0.03 type="white"
        saturation 0.95
        backdrop-blur true
        jelly-flex 0.0066
        jelly-ripple 0.23
        bevel 12
        offset-x 0
        offset-y 0
    }
}
material "terminal-glass-inactive" {
    glass {
        ior 1.28
        light-ior 2.5
        thickness 49.5
        attenuation-color "#0C1314"
        attenuation-distance 28
        chromatic-aberration 0.34
        distortion 0.42 scale=0.08
        roughness 0.03
        noise 0.03 type="white"
        saturation 0.8
        backdrop-blur false
        jelly-flex 0.0066
        jelly-ripple 0.23
        bevel 12
        offset-x 0
        offset-y 0
    }
}
window-rule {
    match is-active=true
    material "terminal-glass"
}
window-rule {
    match is-active=false
    material "terminal-glass-inactive"
}
"##;

/// The control: one definition in both focus states.
const SAME: &str = r##"
material "terminal-glass" {
    glass { thickness 31.2; }
}
window-rule {
    material "terminal-glass"
}
"##;

/// One definition, a named response per focus state.
const RESPONSE_PAIR: &str = r##"
material "terminal-glass" {
    glass { thickness 31.2; }
    response "default" { ring-glow 0.1; }
    response "lit" { ring-glow 1; }
}
window-rule {
    match is-active=true
    material "terminal-glass" response="lit"
}
window-rule {
    match is-active=false
    material "terminal-glass"
}
"##;

#[derive(Debug, Clone, PartialEq)]
struct Slot {
    name: String,
    id: Id,
    seed: [f32; 3],
    ring_glow: f64,
}

fn set_up(text: &str) -> (Fixture, ClientId) {
    const LINEAR: Kind = Kind::Easing(EasingParams {
        duration_ms: 1000,
        curve: Curve::Linear,
    });
    let mut config = Config::parse_mem(text).unwrap();
    config.animations.window_movement.0.kind = LINEAR;
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    for title in ["left", "right"] {
        let window = f.client(id).create_window();
        let surface = window.surface.clone();
        window.set_title(title);
        window.commit();
        f.roundtrip(id);
        let window = f.client(id).window(&surface);
        window.attach_new_buffer();
        window.ack_last_and_commit();
        f.double_roundtrip(id);
    }
    refresh(&mut f);
    f.niri_complete_animations();
    (f, id)
}

/// One compositor refresh: keyboard focus, then the rule recompute that
/// re-resolves tile materials (`State::refresh` order).
fn refresh(f: &mut Fixture) {
    f.niri_state().refresh_and_flush_clients();
}

fn slot(f: &mut Fixture, title: &str) -> Slot {
    f.niri()
        .layout
        .workspaces()
        .flat_map(|(_, _, workspace)| workspace.tiles())
        .find(|tile| {
            with_toplevel_role(tile.window().toplevel(), |role| {
                role.title.as_deref() == Some(title)
            })
        })
        .and_then(|tile| tile.material())
        .map(|state| Slot {
            name: state.material().name.clone(),
            id: state.id().clone(),
            seed: state.jelly_seed(),
            ring_glow: state.material().response(None).ring_glow,
        })
        .unwrap()
}

/// Where the tile with this title renders: its column's move offset and the
/// view position included, which is what the jelly residual is taken from.
fn render_pos(f: &mut Fixture, title: &str) -> Point<f64, Logical> {
    f.niri()
        .layout
        .active_workspace()
        .unwrap()
        .tiles_with_render_positions()
        .find(|(tile, _, _)| {
            with_toplevel_role(tile.window().toplevel(), |role| {
                role.title.as_deref() == Some(title)
            })
        })
        .map(|(_, pos, _)| pos)
        .unwrap()
}

fn set_time(niri: &mut Niri, time: Duration) {
    // As in `tests::animations`: zero the adjustable clock, set the time at
    // rate 1, then freeze it.
    let now = niri.clock.now();
    niri.clock.set_unadjusted(now);
    let _ = niri.clock.now();
    niri.clock.set_unadjusted(Duration::ZERO);
    niri.clock.set_rate(1.0);
    let _ = niri.clock.now();
    niri.clock.set_unadjusted(time);
    let _ = niri.clock.now();
    niri.clock.set_rate(0.0);
}

#[test]
fn name_swap_replaces_state_identity_and_seed() {
    let (mut f, _) = set_up(LIVE_PAIR);
    let left = slot(&mut f, "left");
    let right = slot(&mut f, "right");
    assert_eq!(left.name, "terminal-glass-inactive");
    assert_eq!(right.name, "terminal-glass");

    f.niri().layout.focus_left();
    refresh(&mut f);

    let left_after = slot(&mut f, "left");
    let right_after = slot(&mut f, "right");
    assert_eq!(left_after.name, "terminal-glass");
    assert_eq!(right_after.name, "terminal-glass-inactive");
    // Both tiles get a new `MaterialState`: a new element `Id` (so a fresh
    // offscreen and full damage) and a new seed, on the frame of the swap.
    assert_ne!(left_after.id, left.id);
    assert_ne!(right_after.id, right.id);
    assert_ne!(left_after.seed, left.seed);
    assert_ne!(right_after.seed, right.seed);
}

#[test]
fn same_definition_focus_toggle_keeps_state() {
    let (mut f, _) = set_up(SAME);
    let left = slot(&mut f, "left");
    let right = slot(&mut f, "right");

    for _ in 0..3 {
        f.niri().layout.focus_left();
        refresh(&mut f);
        f.niri().layout.focus_right();
        refresh(&mut f);
    }

    assert_eq!(slot(&mut f, "left"), left);
    assert_eq!(slot(&mut f, "right"), right);
}

#[test]
fn response_only_focus_switch_updates_in_place() {
    let (mut f, _) = set_up(RESPONSE_PAIR);
    let left = slot(&mut f, "left");
    assert_eq!(left.ring_glow, 0.1);

    f.niri().layout.focus_left();
    refresh(&mut f);

    // A focus-selected named response on one definition keeps the state:
    // same `Id`, same seed, new response values.
    let after = slot(&mut f, "left");
    assert_eq!(after.ring_glow, 1.);
    assert_eq!((after.id, after.seed), (left.id, left.seed));
}

#[test]
fn reversal_within_one_refresh_is_not_a_swap() {
    let (mut f, _) = set_up(LIVE_PAIR);
    let left = slot(&mut f, "left");
    let right = slot(&mut f, "right");

    f.niri().layout.focus_left();
    f.niri().layout.focus_right();
    refresh(&mut f);

    assert_eq!(slot(&mut f, "left"), left);
    assert_eq!(slot(&mut f, "right"), right);
}

#[test]
fn reversal_across_refreshes_never_restores_the_original_state() {
    let (mut f, _) = set_up(LIVE_PAIR);
    let right = slot(&mut f, "right");

    f.niri().layout.focus_left();
    refresh(&mut f);
    let away = slot(&mut f, "right");
    f.niri().layout.focus_right();
    refresh(&mut f);
    let back = slot(&mut f, "right");

    // A -> B -> A mints a third state: the name returns, the seed does not.
    assert_eq!(back.name, right.name);
    assert_ne!(back.id, right.id);
    assert_ne!(back.seed, right.seed);
    assert_ne!(back.seed, away.seed);
}

#[test]
fn swap_during_a_move_keeps_the_layout_motion() {
    let (mut f, _) = set_up(LIVE_PAIR);
    set_time(f.niri(), Duration::ZERO);
    let start = render_pos(&mut f, "right");

    // "right" is active; move its column left and settle to learn the target.
    f.niri().layout.move_left();
    refresh(&mut f);
    set_time(f.niri(), Duration::from_millis(500));
    f.niri().advance_animations();
    let moving = render_pos(&mut f, "right");
    let other = render_pos(&mut f, "left");
    assert!(
        moving.x < start.x && moving.x > 0.,
        "move animation in progress: {start:?} -> {moving:?}"
    );
    let before = slot(&mut f, "right");

    // Toggle focus mid-move: the material swaps, the motion does not.
    f.niri().layout.focus_right();
    refresh(&mut f);
    let after = slot(&mut f, "right");
    assert_ne!(after.name, before.name);
    assert_ne!(after.id, before.id);
    assert_eq!(render_pos(&mut f, "right"), moving);
    assert_eq!(render_pos(&mut f, "left"), other);

    // The move settles on its own clock; settling swaps nothing further.
    set_time(f.niri(), Duration::from_millis(5000));
    f.niri().advance_animations();
    let settled = render_pos(&mut f, "right");
    set_time(f.niri(), Duration::from_millis(6000));
    f.niri().advance_animations();
    assert_eq!(render_pos(&mut f, "right"), settled);
    assert!(settled.x < moving.x);
    assert_eq!(slot(&mut f, "right"), after);
}
