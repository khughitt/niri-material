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
