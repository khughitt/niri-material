use niri_config::Config;
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::*;
use crate::layout::LayoutElement as _;
use crate::utils::with_toplevel_role;

/// The resolved material name of the window with this title.
fn material_of(f: &mut Fixture, title: &str) -> Option<String> {
    f.niri()
        .layout
        .windows()
        .find(|(_, mapped)| {
            with_toplevel_role(mapped.toplevel(), |role| {
                role.title.as_deref() == Some(title)
            })
        })
        .and_then(|(_, mapped)| mapped.rules().material.clone())
}

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

#[test]
fn last_matching_rule_wins() {
    let mut f = Fixture::with_config(config(
        r##"
        material "frost" {
            glass {}
        }

        material "clear" {
            glass {}
        }

        window-rule {
            material "frost"
        }

        window-rule {
            match title="target"
            material "clear"
        }
        "##,
    ));
    f.add_output(1, (1920, 1080));

    let id = f.add_client();
    open_window(&mut f, id, "target");

    // The material field is a scalar: the last matching rule replaces
    // earlier ones rather than merging with them.
    assert_eq!(material_of(&mut f, "target").as_deref(), Some("clear"));
}

#[test]
fn non_matching_window_gets_no_material() {
    let mut f = Fixture::with_config(config(
        r##"
        material "frost" {
            glass {}
        }

        window-rule {
            match title="target"
            material "frost"
        }
        "##,
    ));
    f.add_output(1, (1920, 1080));

    let id = f.add_client();
    open_window(&mut f, id, "other");

    assert_eq!(material_of(&mut f, "other"), None);
}

#[test]
fn focus_conditioned_rule_swaps_material() {
    let mut f = Fixture::with_config(config(
        r##"
        material "frost" {
            glass {}
        }

        material "clear" {
            glass {}
        }

        window-rule {
            material "frost"
        }

        window-rule {
            match is-focused=true
            material "clear"
        }
        "##,
    ));
    f.add_output(1, (1920, 1080));

    let id = f.add_client();
    open_window(&mut f, id, "first");
    open_window(&mut f, id, "second");

    // `Mapped::is_focused` is a cached flag that only
    // `State::update_keyboard_focus` (src/niri.rs:1113) writes, and it sets
    // `need_to_recompute_rules`, which `refresh_window_rules` then acts on.
    // Both calls are required, in this order.
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    assert_eq!(material_of(&mut f, "second").as_deref(), Some("clear"));
    assert_eq!(material_of(&mut f, "first").as_deref(), Some("frost"));

    f.niri().layout.focus_left();
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    assert_eq!(material_of(&mut f, "first").as_deref(), Some("clear"));
    assert_eq!(material_of(&mut f, "second").as_deref(), Some("frost"));
}
