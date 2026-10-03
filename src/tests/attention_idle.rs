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

use niri_config::{Action, Config};
use smithay::wayland::session_lock::SessionLockHandler;

use super::fixture::Fixture;
use crate::niri::RedrawState;
use crate::render_helpers::{RenderCtx, RenderTarget};
use crate::utils::get_monotonic_time;

fn fixture(idle_after: Duration) -> Fixture {
    let mut config = Config::default();
    config.signal.idle_after = idle_after;
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1280, 720));
    f
}

/// `renderer` adds the surfaceless GLES renderer that `output_pass` needs.
fn aurora_fixture(renderer: bool) -> (Fixture, super::client::ClientId) {
    let config = Config::parse_mem(
        r#"
        signal {
            idle-after-ms 60
        }
        material "frost" {
            glass {
                aurora 0.5 {
                    drift-hz 4
                }
            }
        }
        window-rule {
            material "frost"
        }
        "#,
    )
    .unwrap();
    let mut f = Fixture::with_config(config);
    if renderer {
        f.niri_state().backend.headless().add_renderer().unwrap();
    }
    f.add_output(1, (1280, 720));
    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    (f, id)
}

#[test]
fn optic_settling_aurora_startup_pointer_and_reload_edges() {
    let (mut f, id) = aurora_fixture(false);
    assert_eq!(f.niri().layout.windows().count(), 1);
    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    assert!(!f.niri().clock.optic_time(get_monotonic_time()).running);
    assert!(f.niri().input_activity.is_idle());
    assert!(f.niri().input_idle_timer.is_none());

    f.niri().notified_activity_this_iteration = false;
    f.client(id).nudge_pointer();
    f.roundtrip(id);
    assert!(f.niri().clock.optic_time(get_monotonic_time()).running);
    assert!(f.niri().input_idle_timer.is_some());

    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    let held = f.niri().clock.optic_time(get_monotonic_time());
    assert!(!held.running);
    // Raise far above the elapsed quiet time, so a slow host cannot leave
    // the reload on the idle side.
    f.niri().set_input_idle_threshold(Duration::from_secs(10));
    assert!(f.niri().clock.optic_time(get_monotonic_time()).running);
    assert!(f.niri().input_idle_timer.is_some());
    f.niri().set_input_idle_threshold(Duration::from_millis(10));
    let held_after_reload = f.niri().clock.optic_time(get_monotonic_time());
    assert!(!held_after_reload.running);
    f.niri().set_input_idle_threshold(Duration::ZERO);
    let resumed = f.niri().clock.optic_time(get_monotonic_time());
    assert!(resumed.running);
    assert_eq!(resumed.logical_anchor, held_after_reload.logical_now);
    assert!(f.niri().input_idle_timer.is_none());
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

#[test]
fn optic_settling_power_on_keeps_the_timeline_paused() {
    let mut f = fixture(Duration::from_millis(60));
    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    let held = f.niri().clock.optic_time(get_monotonic_time());
    assert!(!held.running);
    f.niri_state().do_action(Action::PowerOffMonitors, false);
    f.niri_state().do_action(Action::PowerOnMonitors, false);
    let after = f.niri().clock.optic_time(get_monotonic_time());
    assert!(!after.running);
    assert_eq!(after.logical_now, held.logical_now);
    assert!(f.niri().input_activity.is_idle());
}

#[test]
fn optic_settling_unlock_handler_resumes_the_timeline() {
    let mut f = fixture(Duration::from_millis(60));
    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    let held = f.niri().clock.optic_time(get_monotonic_time());
    assert!(!held.running);
    f.niri().notified_activity_this_iteration = false;
    SessionLockHandler::unlock(f.niri_state());
    let after = f.niri().clock.optic_time(get_monotonic_time());
    assert!(after.running);
    assert_eq!(after.logical_anchor, held.logical_now);
    assert!(!f.niri().input_activity.is_idle());
    assert!(f.niri().input_idle_timer.is_some());
}

#[test]
fn optic_settling_idle_inhibitor_does_not_resume_the_timeline() {
    let mut f = fixture(Duration::from_millis(60));
    sleep(Duration::from_millis(80));
    f.state.server.dispatch();
    let held = f.niri().clock.optic_time(get_monotonic_time());
    assert!(!held.running);
    for inhibited in [true, false] {
        f.niri()
            .is_fdo_idle_inhibited
            .store(inhibited, std::sync::atomic::Ordering::SeqCst);
        f.niri().refresh_idle_inhibit();
        let after = f.niri().clock.optic_time(get_monotonic_time());
        assert!(!after.running);
        assert_eq!(after.logical_now, held.logical_now);
        assert!(f.niri().input_activity.is_idle());
    }
}

/// One output pass as `Niri::redraw` runs it on a real backend: elements, a
/// render that gathers tile deadlines into the output's ticks, then the timer
/// arm. The headless backend's own render draws nothing. Returns whether the
/// output's optic/signal timer is armed afterwards.
fn output_pass(f: &mut Fixture) -> bool {
    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target: RenderTarget::Output,
                xray: None,
                signal_ticks: None,
            };
            niri.render_to_vec(ctx, &output, false);
        })
        .unwrap();
    niri.arm_signal_timer(&output);
    niri.output_state[&output].signal_timer.is_some()
}

fn settle_redraws(f: &mut Fixture) {
    for state in f.niri().output_state.values_mut() {
        state.redraw_state = RedrawState::Idle;
    }
}

fn all_redraws(f: &mut Fixture, queued: bool) -> bool {
    f.niri().output_state.values().all(|state| {
        let is_queued = matches!(state.redraw_state, RedrawState::Queued);
        is_queued == queued
    })
}

#[test]
fn optic_settling_edges_queue_a_redraw_and_drop_then_rearm_the_optic_timer() {
    let (mut f, _id) = aurora_fixture(true);
    // Keep the detector's own timer out of the way: the edges below come from
    // threshold reloads and `notify_activity`.
    f.niri().set_input_idle_threshold(Duration::from_secs(10));
    f.state.server.dispatch();
    assert!(output_pass(&mut f), "a running Aurora arms its next bucket");

    // Idle edge: one redraw per output, and the held pass arms nothing.
    settle_redraws(&mut f);
    sleep(Duration::from_millis(20));
    f.niri().set_input_idle_threshold(Duration::from_millis(10));
    assert!(!f.niri().clock.optic_time(get_monotonic_time()).running);
    assert!(all_redraws(&mut f, true), "the idle edge queued no redraw");
    f.state.server.dispatch();
    assert!(!output_pass(&mut f), "the held pass kept the optic timer");

    // A reload on the same side is not an edge.
    settle_redraws(&mut f);
    f.niri().set_input_idle_threshold(Duration::from_millis(5));
    assert!(
        all_redraws(&mut f, false),
        "a same-side reload queued a redraw"
    );

    // Resume edge through input: one redraw, and the pass re-arms the wake.
    // No dispatch below: the detector timer, re-armed at 5 ms, must not fire.
    f.niri().notified_activity_this_iteration = false;
    f.niri().notify_activity();
    assert!(f.niri().clock.optic_time(get_monotonic_time()).running);
    assert!(
        all_redraws(&mut f, true),
        "the resume edge queued no redraw"
    );
    assert!(
        output_pass(&mut f),
        "the resumed pass did not re-arm the optic timer"
    );

    // Repeated input while active queues nothing.
    settle_redraws(&mut f);
    f.niri().notified_activity_this_iteration = false;
    f.niri().notify_activity();
    assert!(all_redraws(&mut f, false), "repeated input queued a redraw");
}
