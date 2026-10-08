use std::time::Duration;

use niri_config::Config;
use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};

use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::*;
use crate::layout::LayoutElement as _;
use crate::niri::SetWindowSignalArgs;
use crate::utils::{get_monotonic_time, with_toplevel_role};
use crate::window::signal::SetSlot;

fn config(text: &str) -> Config {
    Config::parse_mem(text).unwrap()
}

fn open_window(f: &mut Fixture, id: ClientId, title: &str) -> WlSurface {
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

fn trigger_signal_transition(f: &mut Fixture, id: u64) {
    f.niri()
        .set_window_signal(SetWindowSignalArgs {
            id,
            source: String::from("t"),
            accent: None,
            level: SignalLevel::Demand,
            motion: SignalMotion::Static,
            tag: None,
            ttl_ms: None,
            after_level: None,
            after_motion: None,
            until_focus: false,
        })
        .unwrap();
    f.niri()
        .pulse_window_signal(id, "t", ImpulseKind::Done, None)
        .unwrap();
    f.niri_state().refresh_and_flush_clients();
}

fn material_fixture() -> (Fixture, ClientId) {
    let mut f = Fixture::with_config(config(
        r#"
        material "tg" { glass {}; }
        window-rule { material "tg"; }
        "#,
    ));
    f.add_output(1, (1920, 1080));
    let client = f.add_client();
    (f, client)
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
fn hidden_tab_signal_transitions_do_not_animate_layout() {
    let (mut f, client) = material_fixture();
    open_window(&mut f, client, "hidden");
    open_window(&mut f, client, "visible");
    let hidden = window_id(&mut f, "hidden");

    f.niri().layout.consume_or_expel_window_left(None);
    f.niri()
        .layout
        .set_column_display(niri_ipc::ColumnDisplay::Tabbed);
    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    let (_, _, visible) = f
        .niri()
        .layout
        .active_workspace()
        .unwrap()
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().id().get() == hidden)
        .unwrap();
    assert!(!visible, "target window must be a hidden tab");
    assert!(!f.niri().layout.are_animations_ongoing(None));

    trigger_signal_transition(&mut f, hidden);
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None));
}

#[test]
fn offscreen_column_signal_transitions_do_not_animate_layout() {
    let (mut f, client) = material_fixture();
    for title in ["offscreen", "middle", "visible"] {
        let surface = open_window(&mut f, client, title);
        let window = f.client(client).window(&surface);
        window.set_size(1000, 800);
        window.ack_last_and_commit();
        f.double_roundtrip(client);
    }
    let offscreen = window_id(&mut f, "offscreen");

    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    let (tile, pos, _) = f
        .niri()
        .layout
        .active_workspace()
        .unwrap()
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().id().get() == offscreen)
        .unwrap();
    assert!(
        !crate::render_helpers::signal::slab_in_view(
            pos,
            tile.tile_size(),
            tile.material().unwrap().material().glass.bevel,
            smithay::utils::Rectangle::from_size((1920., 1080.).into()),
        ),
        "target column's material slab must be outside the output: pos={pos:?}, size={:?}",
        tile.tile_size()
    );
    assert!(!f.niri().layout.are_animations_ongoing(None));

    trigger_signal_transition(&mut f, offscreen);
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None));
}

#[test]
fn offscreen_workspace_signal_transitions_do_not_animate_layout() {
    let (mut f, client) = material_fixture();
    open_window(&mut f, client, "offscreen");
    open_window(&mut f, client, "visible");
    let offscreen = window_id(&mut f, "offscreen");

    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    f.niri().layout.move_to_workspace_down(true);
    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    assert!(
        !f.niri()
            .layout
            .active_workspace()
            .unwrap()
            .windows()
            .any(|window| window.id().get() == offscreen),
        "target window must be on a culled workspace"
    );
    assert!(!f.niri().layout.are_animations_ongoing(None));

    trigger_signal_transition(&mut f, offscreen);
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None));
}

#[test]
fn culled_workspace_stops_live_signal_transition() {
    let (mut f, client) = material_fixture();
    open_window(&mut f, client, "offscreen");
    open_window(&mut f, client, "visible");
    let offscreen = window_id(&mut f, "offscreen");

    f.niri_complete_animations();
    trigger_signal_transition(&mut f, offscreen);
    f.niri().layout.update_render_elements(None);
    assert!(f.niri().layout.are_animations_ongoing(None));

    f.niri().layout.move_to_workspace_down(true);
    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None));
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

/// Three 1000 px columns on a 1920 px output, the first ("offscreen") outside
/// the normal view. Only it carries a sustained optic (Aurora drift), so it
/// alone can report a deadline. In the overview (zoom 0.5) the workspace
/// spans 3840 px around the output's centre and the column is on screen.
/// Each test checks the closed-overview baseline first, so a layout default
/// that brings the column into the normal view fails loudly. Returns the
/// windows' surfaces in that order.
fn overview_fixture() -> (Fixture, ClientId, Vec<WlSurface>) {
    let mut f = Fixture::with_config(config(
        r#"
        overview { zoom 0.5; }
        material "tg" { glass {}; }
        material "aurora" { glass { aurora 0.5 { drift-hz 4; }; }; }
        window-rule { material "tg"; }
        window-rule {
            match title="offscreen"
            material "aurora"
        }
        "#,
    ));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (1920, 1080));
    let client = f.add_client();
    let mut surfaces = Vec::new();
    for title in ["offscreen", "middle", "visible"] {
        let surface = open_window(&mut f, client, title);
        let window = f.client(client).window(&surface);
        window.set_size(1000, 800);
        window.ack_last_and_commit();
        f.double_roundtrip(client);
        surfaces.push(surface);
    }
    f.niri_complete_animations();
    (f, client, surfaces)
}

/// One output frame through `Niri::render`, then the deadline its tiles
/// reported.
fn rendered_deadline(f: &mut Fixture) -> Option<Duration> {
    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    backend
        .with_primary_renderer(|renderer| {
            let ctx = crate::render_helpers::RenderCtx {
                renderer,
                target: crate::render_helpers::RenderTarget::Output,
                xray: None,
                signal_ticks: None,
            };
            niri.render_to_vec(ctx, &output, false);
        })
        .unwrap();
    niri.output_state[&output].signal_ticks.next.get()
}

#[test]
fn overview_column_outside_the_normal_view_reports_its_optic_deadline() {
    let (mut f, _client, _) = overview_fixture();
    assert_eq!(
        rendered_deadline(&mut f),
        None,
        "outside the normal view the column reports no deadline"
    );

    f.niri().layout.open_overview();
    f.niri_complete_animations();
    assert!(
        rendered_deadline(&mut f).is_some(),
        "the overview shows the column, so its Aurora drift must report a deadline"
    );

    f.niri().layout.close_overview();
    f.niri_complete_animations();
    assert_eq!(rendered_deadline(&mut f), None);
}

#[test]
fn overview_column_outside_the_normal_view_animates_its_signal_transition() {
    let (mut f, _client, _) = overview_fixture();
    let offscreen = window_id(&mut f, "offscreen");
    trigger_signal_transition(&mut f, offscreen);
    f.niri().layout.update_render_elements(None);
    assert!(
        !f.niri().layout.are_animations_ongoing(None),
        "outside the normal view the column's transition does not animate"
    );

    f.niri().layout.open_overview();
    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    assert!(
        f.niri().layout.are_animations_ongoing(None),
        "the overview shows the column, so its live transition must animate"
    );
}

/// The ids of the elements one frame of `target` draws.
fn rendered_ids(
    f: &mut Fixture,
    target: crate::render_helpers::RenderTarget,
) -> std::collections::HashSet<smithay::backend::renderer::element::Id> {
    use smithay::backend::renderer::element::Element as _;

    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    backend
        .with_primary_renderer(|renderer| {
            let ctx = crate::render_helpers::RenderCtx {
                renderer,
                target,
                xray: None,
                signal_ticks: None,
            };
            niri.render_to_vec(ctx, &output, false)
                .iter()
                .map(|elem| elem.id().clone())
                .collect()
        })
        .unwrap()
}

/// The id of the material element the window titled `title` draws.
fn material_id(f: &mut Fixture, title: &str) -> smithay::backend::renderer::element::Id {
    let id = window_id(f, title);
    f.niri()
        .layout
        .workspaces()
        .flat_map(|(_, _, ws)| ws.tiles())
        .find(|tile| tile.window().id().get() == id)
        .and_then(|tile| tile.material())
        .unwrap()
        .id()
        .clone()
}

#[test]
fn output_render_culls_the_material_of_a_column_outside_the_view() {
    use crate::render_helpers::RenderTarget;

    let (mut f, _client, _) = overview_fixture();
    let offscreen = material_id(&mut f, "offscreen");
    let visible = material_id(&mut f, "visible");

    let ids = rendered_ids(&mut f, RenderTarget::Output);
    assert!(ids.contains(&visible), "the column in view draws its glass");
    assert!(
        !ids.contains(&offscreen),
        "the column outside the view skips its material work"
    );
    assert!(
        rendered_ids(&mut f, RenderTarget::ScreenCapture).contains(&offscreen),
        "targets other than the output keep rendering every tile"
    );

    f.niri().layout.open_overview();
    f.niri_complete_animations();
    assert!(
        rendered_ids(&mut f, RenderTarget::Output).contains(&offscreen),
        "the overview shows the column, so it draws its glass"
    );

    f.niri().layout.close_overview();
    f.niri_complete_animations();
    assert!(!rendered_ids(&mut f, RenderTarget::Output).contains(&offscreen));
}

#[test]
fn culled_column_shows_its_latest_commit_on_the_first_revealed_frame() {
    use smithay::backend::allocator::Fourcc;
    use smithay::backend::renderer::element::Element as _;
    use smithay::utils::{Scale, Transform};

    use crate::render_helpers::{render_to_vec, RenderCtx, RenderTarget};

    const BLUE: u32 = 0xff00_00ff;
    const RED: u32 = 0xffff_0000;

    let (mut f, client, surfaces) = overview_fixture();
    let offscreen = window_id(&mut f, "offscreen");
    let offscreen_material = material_id(&mut f, "offscreen");
    let surface = surfaces[0].clone();
    let commit = |f: &mut Fixture, argb: u32| {
        let window = f.client(client).window(&surface);
        window.attach_new_shm_buffer(argb);
        window.commit();
        f.double_roundtrip(client);
    };

    // The overview shows the column, so its offscreen holds blue.
    commit(&mut f, BLUE);
    f.niri().layout.open_overview();
    f.niri_complete_animations();
    assert!(rendered_ids(&mut f, RenderTarget::Output).contains(&offscreen_material));
    f.niri().layout.close_overview();
    f.niri_complete_animations();

    // More commits than the surface damage history keeps, each followed by
    // a culled frame; the last is red.
    for argb in [BLUE, BLUE, BLUE, BLUE, BLUE, RED] {
        commit(&mut f, argb);
        assert!(!rendered_ids(&mut f, RenderTarget::Output).contains(&offscreen_material));
    }

    f.niri().layout.focus_column_first();
    f.niri_complete_animations();

    let output = f.niri_output(1);
    let size = output.current_mode().unwrap().size;
    let (tile, pos, _) = f
        .niri()
        .layout
        .active_workspace()
        .unwrap()
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().id().get() == offscreen)
        .unwrap();
    let centre = pos + tile.window_loc() + tile.window_size().downscale(2.).to_point();
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    let pixels = backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target: RenderTarget::Output,
                xray: None,
                signal_ticks: None,
            };
            let elements = niri.render_to_vec(ctx, &output, false);
            assert!(
                elements.iter().any(|elem| elem.id() == &offscreen_material),
                "the revealed column draws its glass"
            );
            render_to_vec(
                renderer,
                size,
                Scale::from(1.),
                Transform::Normal,
                Fourcc::Abgr8888,
                elements.iter().rev(),
            )
            .unwrap()
        })
        .unwrap();
    let i = (centre.y as usize * size.w as usize + centre.x as usize) * 4;
    assert_eq!(
        &pixels[i..i + 4],
        &[255, 0, 0, 255],
        "the first frame in view shows the commit made while culled"
    );
}

/// A material fixture with a renderer, so `rendered_ids` draws real glass.
fn rendered_material_fixture() -> (Fixture, ClientId) {
    let (mut f, client) = material_fixture();
    f.niri_state().backend.headless().add_renderer().unwrap();
    (f, client)
}

fn open_sized_window(f: &mut Fixture, client: ClientId, title: &str, w: u16, h: u16) -> WlSurface {
    let surface = open_window(f, client, title);
    let window = f.client(client).window(&surface);
    window.set_size(w, h);
    window.ack_last_and_commit();
    f.double_roundtrip(client);
    surface
}

/// The render position and size of the tile titled `title` on the active
/// workspace.
fn tile_rect(
    f: &mut Fixture,
    title: &str,
) -> smithay::utils::Rectangle<f64, smithay::utils::Logical> {
    let id = window_id(f, title);
    f.niri()
        .layout
        .active_workspace()
        .unwrap()
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().id().get() == id)
        .map(|(tile, pos, _)| smithay::utils::Rectangle::new(pos, tile.tile_size()))
        .unwrap()
}

#[test]
fn floating_window_at_the_screen_edge_keeps_its_glass() {
    use niri_ipc::PositionChange;

    use crate::render_helpers::RenderTarget;

    let (mut f, client) = rendered_material_fixture();
    open_sized_window(&mut f, client, "tiled", 800, 600);
    open_sized_window(&mut f, client, "floating", 800, 600);
    f.niri().layout.toggle_window_floating(None);
    f.niri().layout.move_floating_window(
        None,
        PositionChange::SetFixed(-5000.),
        PositionChange::SetFixed(-5000.),
        false,
    );
    f.niri_complete_animations();
    let floating = material_id(&mut f, "floating");

    // Floating placement keeps a corner on screen (at most 75 px each way);
    // the cull must see it in the same coordinates as the render.
    let rect = tile_rect(&mut f, "floating");
    assert!(
        rect.loc.x + rect.size.w <= 75. && rect.loc.y + rect.size.h <= 75.,
        "the window must sit at the top-left clamp: {rect:?}"
    );
    assert!(rendered_ids(&mut f, RenderTarget::Output).contains(&floating));
}

/// Drags the window titled `title` with the pointer at each of `pointers`
/// in turn, and reports after each whether the output and a screen capture
/// draw its glass.
fn drag_drawn(f: &mut Fixture, title: &str, pointers: &[(f64, f64)]) -> Vec<(bool, bool)> {
    use smithay::utils::Point;

    use crate::render_helpers::RenderTarget;

    let material = material_id(f, title);
    let id = window_id(f, title);
    let window = f
        .niri()
        .layout
        .windows()
        .find(|(_, m)| m.id().get() == id)
        .map(|(_, m)| m.window.clone())
        .unwrap();
    let output = f.niri_output(1);
    assert!(f.niri().layout.interactive_move_begin(
        window.clone(),
        &output,
        Point::from((960., 540.))
    ));
    pointers
        .iter()
        .map(|&pointer| {
            assert!(f.niri().layout.interactive_move_update(
                &window,
                Point::from((1000., 0.)),
                output.clone(),
                Point::from(pointer),
            ));
            f.niri_complete_animations();
            (
                rendered_ids(f, RenderTarget::Output).contains(&material),
                rendered_ids(f, RenderTarget::ScreenCapture).contains(&material),
            )
        })
        .collect()
}

/// A pointer far past the left edge drags the whole tile off the output.
const DRAG_CENTRE: (f64, f64) = (960., 540.);
const DRAG_OFF: (f64, f64) = (-4000., 540.);

/// Floating, because a tiled tile is dragged at reduced opacity through the
/// alpha animation's offscreen, which never culls and hides the material
/// element's id inside its own.
#[test]
fn interactive_move_culls_a_floating_tile_only_while_it_is_off_the_output() {
    let (mut f, client) = rendered_material_fixture();
    open_sized_window(&mut f, client, "other", 800, 600);
    open_sized_window(&mut f, client, "moving", 800, 600);
    f.niri().layout.toggle_window_floating(None);
    f.niri_complete_animations();

    // Output then screen capture at each pointer: the capture never culls.
    assert_eq!(
        drag_drawn(&mut f, "moving", &[DRAG_CENTRE, DRAG_OFF, DRAG_CENTRE]),
        [(true, true), (false, true), (true, true)]
    );
}

#[test]
fn workspace_switch_draws_both_workspaces_and_culls_columns_beyond_the_view() {
    use crate::render_helpers::RenderTarget;

    let (mut f, client) = rendered_material_fixture();
    open_sized_window(&mut f, client, "upper", 800, 600);
    // The lower workspace holds a column outside its view and one in it.
    open_sized_window(&mut f, client, "lower-offscreen", 1000, 800);
    f.niri().layout.move_to_workspace_down(true);
    for title in ["lower-middle", "lower-visible"] {
        open_sized_window(&mut f, client, title, 1000, 800);
    }
    f.niri_complete_animations();
    let lower_visible = material_id(&mut f, "lower-visible");
    let lower_offscreen = material_id(&mut f, "lower-offscreen");
    let upper = material_id(&mut f, "upper");
    let rect = tile_rect(&mut f, "lower-offscreen");
    assert!(
        rect.loc.x + rect.size.w < -50.,
        "the column must be outside the lower workspace's view: {rect:?}"
    );

    f.niri().layout.switch_workspace_up();
    f.niri_complete_animations();
    let ids = rendered_ids(&mut f, RenderTarget::Output);
    assert!(ids.contains(&upper) && !ids.contains(&lower_visible));

    // Halfway down: both workspaces are on screen.
    let output = f.niri_output(1);
    f.niri()
        .layout
        .workspace_switch_gesture_begin(&output, false);
    f.niri()
        .layout
        .workspace_switch_gesture_update(548., Duration::from_millis(10), false);
    let ids = rendered_ids(&mut f, RenderTarget::Output);
    assert!(ids.contains(&upper), "the outgoing workspace draws");
    assert!(ids.contains(&lower_visible), "the incoming workspace draws");
    assert!(
        !ids.contains(&lower_offscreen),
        "a column outside the incoming workspace's view stays culled"
    );
}

#[test]
fn client_shadow_on_screen_keeps_the_glass_of_a_column_outside_the_view() {
    use crate::render_helpers::RenderTarget;

    // A shadow wider than any column offset: the visual window and its slab
    // band are off screen, but its buffer reaches into the view.
    const SHADOW: i32 = 1500;

    let (mut f, client) = rendered_material_fixture();
    let surface = open_window(&mut f, client, "shadowed");
    let window = f.client(client).window(&surface);
    window.set_size(1000 + 2 * SHADOW as u16, 800);
    window.set_geometry(SHADOW, 0, 1000, 800);
    window.ack_last_and_commit();
    f.double_roundtrip(client);
    for title in ["middle", "visible"] {
        open_sized_window(&mut f, client, title, 1000, 800);
    }
    f.niri_complete_animations();
    let shadowed = material_id(&mut f, "shadowed");

    let rect = tile_rect(&mut f, "shadowed");
    let right = rect.loc.x + rect.size.w;
    assert!(
        right + 50. < 0. && right + f64::from(SHADOW) > 0.,
        "band off screen, shadow on screen: {rect:?}"
    );
    assert!(
        rendered_ids(&mut f, RenderTarget::Output).contains(&shadowed),
        "the buffer extent keeps a visible shadow drawn"
    );
}
