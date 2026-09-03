use std::time::Duration;

use niri_config::Config;
use niri_ipc::{SignalLevel, SignalMotion};

use super::client::ClientId;
use super::*;
use crate::layout::LayoutElement as _;
use crate::utils::{get_monotonic_time, with_toplevel_role};
use crate::window::signal::SetSlot;

fn config(text: &str) -> Config {
    Config::parse_mem(text).unwrap()
}

fn open_window(
    f: &mut Fixture,
    id: ClientId,
    title: &str,
) -> wayland_client::protocol::wl_surface::WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.set_title(title);
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

fn material_of(f: &mut Fixture, title: &str) -> Option<String> {
    f.niri()
        .layout
        .windows()
        .find(|(_, m)| with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some(title)))
        .and_then(|(_, m)| m.rules().material.as_ref().map(|r| r.name.clone()))
}

fn window_id(f: &mut Fixture, title: &str) -> u64 {
    f.niri()
        .layout
        .windows()
        .find(|(_, m)| with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some(title)))
        .map(|(_, m)| m.id().get())
        .unwrap()
}

fn set_slot(level: SignalLevel, tag: Option<&str>, until_focus: bool) -> SetSlot {
    SetSlot {
        accent: None,
        level,
        motion: SignalMotion::Static,
        tag: tag.map(String::from),
        expiry: None,
        until_focus,
    }
}

#[test]
fn urgency_creates_native_slot_and_matches_signal_source() {
    let mut f = Fixture::with_config(config(
        r##"
        material "calm" { glass {}; }
        material "alarm" { glass {}; }
        window-rule { material "calm"; }
        window-rule {
            match signal-source="^niri$"
            material "alarm"
        }
        "##,
    ));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    open_window(&mut f, id, "b");
    assert_eq!(material_of(&mut f, "a").as_deref(), Some("calm"));

    let now = get_monotonic_time();
    {
        let niri = f.niri();
        let mapped = niri
            .layout
            .find_window_mut_by(|m| {
                with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some("a"))
            })
            .unwrap();
        mapped.set_urgent(true);
        let folded = mapped.signals().fold(now).unwrap();
        assert_eq!(
            (folded.level, folded.motion),
            (SignalLevel::Demand, SignalMotion::Pulse)
        );
        assert!(mapped.take_signal_deadline_dirty());
    }
    f.niri_state().refresh_and_flush_clients();
    assert_eq!(material_of(&mut f, "a").as_deref(), Some("alarm"));

    let mapped = f
        .niri()
        .layout
        .find_window_mut_by(|m| {
            with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some("a"))
        })
        .unwrap();
    mapped.set_is_focused(true);
    assert!(mapped.signals().fold(get_monotonic_time()).is_none());
    assert!(mapped.take_signal_deadline_dirty());
    f.niri_state().refresh_and_flush_clients();
    assert_eq!(material_of(&mut f, "a").as_deref(), Some("calm"));
}

#[test]
fn source_matches_any_slot_and_tag_matches_only_the_folded_tag() {
    let mut f = Fixture::with_config(config(
        r##"
        material "calm" { glass {}; }
        material "tagged" { glass {}; }
        material "sourced" { glass {}; }
        window-rule { material "calm"; }
        window-rule {
            match signal-tag="^page$"
            material "tagged"
        }
        window-rule {
            match signal-source="^background$"
            material "sourced"
        }
        "##,
    ));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    for title in ["any-source", "folded-tag", "none"] {
        open_window(&mut f, id, title);
    }

    let now = get_monotonic_time();
    for (title, background) in [("any-source", true), ("folded-tag", false)] {
        let mapped = f
            .niri()
            .layout
            .find_window_mut_by(|m| {
                with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some(title))
            })
            .unwrap();
        if background {
            mapped
                .signals_mut()
                .set("background", set_slot(SignalLevel::Quiet, None, false), now)
                .unwrap();
            mapped.signal_changed();
        }
        mapped
            .signals_mut()
            .set(
                "winner",
                set_slot(SignalLevel::Demand, Some("page"), false),
                now + Duration::from_millis(1),
            )
            .unwrap();
        mapped.signal_changed();
        assert!(mapped.take_signal_deadline_dirty());
    }

    f.niri_state().refresh_and_flush_clients();
    assert_eq!(
        material_of(&mut f, "any-source").as_deref(),
        Some("sourced")
    );
    assert_eq!(material_of(&mut f, "folded-tag").as_deref(), Some("tagged"));
    assert_eq!(material_of(&mut f, "none").as_deref(), Some("calm"));
}

#[test]
fn focus_clears_native_urgency_and_demotes_only_until_focus_slots() {
    let mut f = Fixture::new();
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    open_window(&mut f, id, "b");

    let now = get_monotonic_time();
    let mapped = f
        .niri()
        .layout
        .find_window_mut_by(|m| {
            with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some("a"))
        })
        .unwrap();
    mapped
        .signals_mut()
        .set("until", set_slot(SignalLevel::Demand, None, true), now)
        .unwrap();
    mapped.signal_changed();
    mapped
        .signals_mut()
        .set(
            "persistent",
            set_slot(SignalLevel::Notice, None, false),
            now + Duration::from_millis(1),
        )
        .unwrap();
    mapped.signal_changed();
    mapped.set_urgent(true);
    assert!(mapped
        .signals()
        .fold(now)
        .unwrap()
        .sources
        .iter()
        .any(|s| s == "niri"));
    assert!(mapped.take_signal_deadline_dirty());

    mapped.set_is_focused(true);
    let folded = mapped.signals().fold(now).unwrap();
    assert_eq!(folded.level, SignalLevel::Notice);
    assert!(!folded.sources.iter().any(|s| s == "niri"));
    assert!(folded.sources.iter().any(|s| s == "until"));
    assert!(folded.sources.iter().any(|s| s == "persistent"));
    assert!(mapped.take_signal_deadline_dirty());
}

#[test]
fn ipc_entry_points_validate_and_mutate() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};

    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    let wid = window_id(&mut f, "a");

    let args = |source: &str| SetWindowSignalArgs {
        id: wid,
        source: source.to_owned(),
        accent: Some(String::from("#e5a33c")),
        level: SignalLevel::Demand,
        motion: SignalMotion::Pulse,
        tag: Some(String::from("cats/ginger")),
        ttl_ms: Some(30_000),
        after_level: Some(SignalLevel::Quiet),
        after_motion: None,
        until_focus: true,
    };

    let niri = f.niri();
    assert!(niri
        .set_window_signal(SetWindowSignalArgs {
            id: 999,
            ..args("familiar")
        })
        .is_err());
    assert!(niri.set_window_signal(args("niri")).is_err());
    assert!(niri
        .pulse_window_signal(wid, "niri", ImpulseKind::Done, None)
        .is_err());
    assert!(niri.clear_window_signal(wid, "niri").is_err());
    assert!(niri
        .pulse_window_signal(wid, "familiar", ImpulseKind::Done, None)
        .is_err());
    niri.set_window_signal(args("familiar")).unwrap();
    assert!(niri
        .pulse_window_signal(wid, "familiar", ImpulseKind::Done, Some("zzz"))
        .is_err());
    niri.pulse_window_signal(wid, "familiar", ImpulseKind::Done, None)
        .unwrap();

    let now = get_monotonic_time();
    let (_, mapped) = niri
        .layout
        .windows()
        .find(|(_, m)| m.id().get() == wid)
        .unwrap();
    let signal = crate::ipc::server::to_ipc_signal(&mapped.signals().fold(now).unwrap());
    assert_eq!(signal.level, SignalLevel::Demand);
    assert_eq!(signal.accent.as_deref(), Some("#e5a33c"));
    assert_eq!(signal.sources, vec!["familiar"]);
    assert_eq!(signal.impulses.len(), 1);
    assert_eq!(signal.impulses[0].kind, ImpulseKind::Done);
    assert_eq!(
        signal.impulses[0].expires_at.secs * 1_000_000_000
            + u64::from(signal.impulses[0].expires_at.nanos),
        signal.impulses[0].at.secs * 1_000_000_000
            + u64::from(signal.impulses[0].at.nanos)
            + 1_500_000_000
    );

    niri.clear_window_signal(wid, "familiar").unwrap();
    assert!(niri.clear_window_signal(wid, "familiar").is_err());
    let (_, mapped) = niri
        .layout
        .windows()
        .find(|(_, m)| m.id().get() == wid)
        .unwrap();
    assert!(mapped.signals().fold(now).is_none());
}

#[test]
fn refresh_reconciles_deadline_timer_after_focus() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{SignalLevel, SignalMotion};

    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    open_window(&mut f, id, "b");
    let wid = window_id(&mut f, "a");
    f.niri()
        .set_window_signal(SetWindowSignalArgs {
            id: wid,
            source: String::from("t"),
            accent: None,
            level: SignalLevel::Demand,
            motion: SignalMotion::Pulse,
            tag: None,
            ttl_ms: Some(5000),
            after_level: Some(SignalLevel::Quiet),
            after_motion: None,
            until_focus: true,
        })
        .unwrap();
    f.niri_state().refresh_and_flush_clients();
    let mapped_id = f
        .niri()
        .layout
        .windows()
        .find(|(_, m)| m.id().get() == wid)
        .map(|(_, m)| m.id())
        .unwrap();
    assert!(
        f.niri().signal_deadlines.contains_key(&mapped_id),
        "ttl armed a deadline"
    );

    // Focusing "a" demotes the until-focus slot and cancels its expiry.
    // `Mapped::is_focused` is written only by `State::update_keyboard_focus`
    // (see the idiom in src/tests/material.rs), so move focus, update it,
    // then refresh so `refresh_signal_deadlines` runs.
    f.niri().layout.focus_left();
    f.niri_state().update_keyboard_focus();
    f.niri_state().refresh_and_flush_clients();
    assert!(
        !f.niri().signal_deadlines.contains_key(&mapped_id),
        "focus reconciled the timer away"
    );
}

#[test]
fn rewrite_replaces_the_deadline_and_close_cancels_it() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{SignalLevel, SignalMotion};

    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    let surface = open_window(&mut f, id, "a");
    let wid = window_id(&mut f, "a");
    let args = |ttl: u32| SetWindowSignalArgs {
        id: wid,
        source: String::from("t"),
        accent: None,
        level: SignalLevel::Notice,
        motion: SignalMotion::Static,
        tag: None,
        ttl_ms: Some(ttl),
        after_level: Some(SignalLevel::Quiet),
        after_motion: None,
        until_focus: false,
    };
    f.niri().set_window_signal(args(5000)).unwrap();
    f.niri_state().refresh_and_flush_clients();
    f.niri().set_window_signal(args(9000)).unwrap();
    f.niri_state().refresh_and_flush_clients();
    assert_eq!(
        f.niri().signal_deadlines.len(),
        1,
        "rewrite replaced, not duplicated"
    );

    // Unmap by committing a null buffer, the idiom used in src/tests/floating.rs.
    let window = f.client(id).window(&surface);
    window.attach_null();
    window.commit();
    f.double_roundtrip(id);
    f.niri_state().refresh_and_flush_clients();
    assert!(
        f.niri().signal_deadlines.is_empty(),
        "unmap cancelled the timer"
    );
}

#[test]
fn crossfade_and_live_impulses_are_transitions_but_sustained_motion_is_not() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};
    use std::time::Duration;

    let mut f = Fixture::with_config(config(
        r#"
        material "tg" { glass {}; }
        window-rule { material "tg"; }
        "#,
    ));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    let wid = window_id(&mut f, "a");

    // Baseline: settle the open animation first, or it masks every assertion below.
    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    assert!(
        !f.niri().layout.are_animations_ongoing(None),
        "settled window has no transition"
    );

    f.niri()
        .set_window_signal(SetWindowSignalArgs {
            id: wid,
            source: String::from("t"),
            accent: Some(String::from("#e5a33c")),
            level: SignalLevel::Demand,
            motion: SignalMotion::Pulse,
            tag: None,
            ttl_ms: None,
            after_level: None,
            after_motion: None,
            until_focus: false,
        })
        .unwrap();
    // The crossfade is created by update_render_elements, which the headless
    // fixture does not run on its own.
    f.niri_state().refresh_and_flush_clients();
    f.niri().layout.update_render_elements(None);
    assert!(
        f.niri().layout.are_animations_ongoing(None),
        "crossfade is a transition"
    );

    f.niri_complete_animations(); // drops the finished crossfade in Tile::advance_animations
    f.niri().layout.update_render_elements(None);
    assert!(
        !f.niri().layout.are_animations_ongoing(None),
        "sustained Pulse is not a transition"
    );

    f.niri()
        .pulse_window_signal(wid, "t", ImpulseKind::Done, None)
        .unwrap();
    f.niri_state().refresh_and_flush_clients();
    f.niri().layout.update_render_elements(None);
    assert!(
        f.niri().layout.are_animations_ongoing(None),
        "live impulse is a transition"
    );

    // Expire the impulse by advancing the frozen unadjusted clock past 1.5 s.
    let later = f.niri().clock.now_unadjusted() + Duration::from_secs(2);
    f.niri().clock.set_unadjusted(later);
    f.niri().layout.update_render_elements(None);
    assert!(
        !f.niri().layout.are_animations_ongoing(None),
        "expired impulse is not a transition"
    );
    f.niri().clock.clear();
}

#[test]
fn unsignaled_material_window_has_no_transition() {
    let mut f = Fixture::with_config(config(
        r#"
        material "tg" { glass {}; }
        window-rule { material "tg"; }
        "#,
    ));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    f.niri_complete_animations(); // finish the open animation
    f.niri().layout.update_render_elements(None);
    assert!(
        !f.niri().layout.are_animations_ongoing(None),
        "no crossfade starts for a quiet window"
    );
}

#[test]
fn arm_signal_timer_follows_the_accumulator() {
    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let output = f.niri_output(1);

    f.niri().arm_signal_timer(&output);
    assert!(f.niri().output_state[&output].signal_timer.is_none());

    let deadline = get_monotonic_time() + std::time::Duration::from_millis(50);
    f.niri().output_state[&output].signal_ticks.report(deadline);
    f.niri().arm_signal_timer(&output);
    assert!(f.niri().output_state[&output].signal_timer.is_some());

    f.niri().output_state[&output].signal_ticks.reset();
    f.niri().arm_signal_timer(&output);
    assert!(
        f.niri().output_state[&output].signal_timer.is_none(),
        "reset removes the timer"
    );
}
