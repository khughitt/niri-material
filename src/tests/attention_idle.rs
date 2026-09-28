//! The attention idle gate through the fixture: the timer token invariant
//! across an early fire and reloads, and the input → `notify_activity` →
//! layout → timer wiring through a virtual pointer
//! (docs/specs/2026-09-18-ring-focus-motion-design.md §3).
//!
//! Timer-only checkpoints call `f.state.server.dispatch()` directly:
//! `Fixture::dispatch()` polls the outer loop and dispatches the server only
//! when its file descriptor is readable, and calloop discovers expired
//! timers inside the server loop's own poll.

use std::thread::sleep;
use std::time::Duration;

use niri_config::Config;

use super::fixture::Fixture;
use crate::niri::RedrawState;
use crate::utils::get_monotonic_time;

fn fixture(idle_after: Duration) -> Fixture {
    let mut config = Config::default();
    config.signal.idle_after = idle_after;
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1280, 720));
    f
}

#[test]
fn idle_timer_keeps_one_live_token_across_an_early_fire_and_reloads() {
    let mut f = fixture(Duration::from_millis(150));
    assert!(f.niri().input_idle_timer.is_some(), "armed at construction");
    let fires = f.niri().input_idle_timer_fires;

    // Input at 80 ms, then the original 150 ms deadline fires early:
    // poll says Rearm, and the token must point at the live source.
    sleep(Duration::from_millis(80));
    f.niri().notified_activity_this_iteration = false;
    f.niri().notify_activity();
    sleep(Duration::from_millis(90));
    f.state.server.dispatch();
    assert_eq!(
        f.niri().input_idle_timer_fires,
        fires + 1,
        "the early timer actually fired"
    );
    assert!(!f.niri().input_activity.is_idle());
    assert!(
        f.niri().input_idle_timer.is_some(),
        "re-armed after the early fire"
    );

    // A reload that raises the threshold must be able to cancel that timer
    // and arm one; the invariant is token.is_some() == next_check().is_some().
    f.niri()
        .set_input_idle_threshold(Duration::from_millis(250));
    assert!(f.niri().input_idle_timer.is_some());
    // This is later than the cancelled 150 ms timer's deadline. Wait past
    // both below, so either source leaking is observable.
    let cancelled_deadline = f.niri().input_activity.next_check().unwrap();
    f.niri().set_input_idle_threshold(Duration::ZERO);
    assert!(f.niri().input_idle_timer.is_none(), "gate off, no timer");

    // Lowering under the elapsed quiet time flips on the spot and leaves no timer.
    f.niri().set_input_idle_threshold(Duration::from_millis(50));
    assert!(f.niri().input_activity.is_idle());
    assert!(f.niri().input_idle_timer.is_none());
    assert!(!f.niri().layout.input_active());

    // Neither cancelled source may fire, even if a callback would leave
    // the already-idle state unchanged.
    let fires = f.niri().input_idle_timer_fires;
    sleep(cancelled_deadline.saturating_sub(get_monotonic_time()) + Duration::from_millis(20));
    f.state.server.dispatch();
    assert_eq!(
        f.niri().input_idle_timer_fires,
        fires,
        "a cancelled source fired"
    );
    assert!(f.niri().input_activity.is_idle());
    assert!(f.niri().input_idle_timer.is_none());
}

#[test]
fn virtual_pointer_input_resumes_attention_and_the_gate_re_engages() {
    let mut f = fixture(Duration::from_millis(60));
    let id = f.add_client();
    f.roundtrip(id);

    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    assert!(f.niri().input_activity.is_idle());
    assert!(!f.niri().layout.input_active());
    assert!(f.niri().input_idle_timer.is_none());

    // Real input: the virtual pointer's motion enters process_input_event,
    // which calls notify_activity before any per-event handling.
    f.niri().notified_activity_this_iteration = false;
    f.client(id).nudge_pointer();
    f.roundtrip(id);
    assert!(!f.niri().input_activity.is_idle());
    assert!(f.niri().layout.input_active(), "the layout saw the resume");
    assert!(
        f.niri().input_idle_timer.is_some(),
        "the gate is armed again"
    );

    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    assert!(
        f.niri().input_activity.is_idle(),
        "idle again after the threshold"
    );
    assert!(!f.niri().layout.input_active());
}

#[test]
fn an_idle_edge_that_changes_no_tile_queues_no_redraw() {
    // With no sustained signal on screen the gate changes nothing, so going
    // idle must not wake any output (the idle-budget trace saw exactly one
    // such redraw per quiet case before this).
    // The headless backend bumps `frame_callback_sequence` on every render,
    // so a redraw queued and drawn within one dispatch still shows.
    let mut f = fixture(Duration::from_millis(60));
    // Draw the frame the output was created with, well before the timer.
    f.state.server.dispatch();
    for state in f.niri().output_state.values_mut() {
        state.redraw_state = RedrawState::Idle;
    }
    let sequences = |f: &mut Fixture| -> Vec<u32> {
        f.niri()
            .output_state
            .values()
            .map(|state| state.frame_callback_sequence)
            .collect()
    };
    let before = sequences(&mut f);

    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    assert!(f.niri().input_activity.is_idle());
    assert!(!f.niri().layout.input_active());
    assert_eq!(sequences(&mut f), before, "the idle edge rendered a frame");
    assert!(
        f.niri()
            .output_state
            .values()
            .all(|state| matches!(state.redraw_state, RedrawState::Idle)),
        "the idle edge queued a redraw"
    );
}
