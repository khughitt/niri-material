use niri_config::Config;
use smithay::backend::renderer::element::{Element as _, Id};
use smithay::desktop::PopupManager;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Anchor, KeyboardInteractivity,
};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::{ClientId, LayerConfigureProps};
use super::*;
use crate::render_helpers::surface_remap::{HueBlend, OutputHue};
use crate::render_helpers::{RenderCtx, RenderTarget};

const GLASS: &str = r##"
material "tg" {
    glass {
        ior 1.28
        thickness 31.2
        attenuation-color "#0D1D1E"
        attenuation-distance 12
    }
}
"##;

const RULE: &str = r##"
layer-rule {
    match namespace="^bar$"
    surface-remap {
        surface "#151310"
        container "#221f1c"
        outline "#4d453c"
        material "tg"
        hue-chroma 0.03
    }
}
"##;

fn config(text: &str) -> Config {
    Config::parse_mem(text).unwrap()
}

pub(super) fn look(extra: &str) -> Config {
    config(&format!("animations {{ off; }}\n{GLASS}\n{RULE}\n{extra}"))
}

pub(super) fn open_layer(
    f: &mut Fixture,
    id: ClientId,
    namespace: &str,
    kb: Option<KeyboardInteractivity>,
) -> WlSurface {
    let layer = f.client(id).create_layer(None, Layer::Top, namespace);
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Left | Anchor::Right | Anchor::Top),
        size: Some((0, 50)),
        kb_interactivity: kb,
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(id);
    let layer = f.client(id).layer(&surface);
    layer.attach_new_buffer();
    layer.set_size(100, 50);
    layer.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

pub(super) fn layer<'a>(f: &'a mut Fixture, namespace: &str) -> &'a mut crate::layer::MappedLayer {
    f.niri()
        .mapped_layer_surfaces
        .values_mut()
        .find(|m| m.surface().namespace() == namespace)
        .unwrap()
}

fn red() -> HueBlend {
    HueBlend::from_accent("#e5533c".parse().unwrap())
}

#[test]
fn layer_with_the_rule_gets_a_remap() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_layer(&mut f, id, "other", None);
    assert!(layer(&mut f, "bar").remap_uniforms().is_some());
    assert!(layer(&mut f, "other").remap_uniforms().is_none());
}

#[test]
fn two_rules_replace_the_node_whole() {
    let second = r##"
layer-rule {
    match namespace="^bar$"
    surface-remap {
        surface "#111318"
        container "#1d2025"
        outline "#414751"
        material "tg"
    }
}
"##;
    let mut f = Fixture::with_config(look(second));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let remap = layer(&mut f, "bar").rules().surface_remap.clone().unwrap();
    assert_eq!(remap.surface, "#111318".parse().unwrap());
    assert_eq!(
        remap.hue_chroma, None,
        "the later node's absent field is not merged"
    );
}

#[test]
fn config_reload_recomputes_the_face() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let before = layer(&mut f, "bar").remap_uniforms().unwrap();
    let text = format!(
        "animations {{ off; }}\n{}\n{RULE}",
        GLASS.replace("distance 12", "distance 11")
    );
    f.niri_state().reload_config(Ok(config(&text)));
    let after = layer(&mut f, "bar").remap_uniforms().unwrap();
    assert_eq!(
        before.anchors, after.anchors,
        "Noctalia's colors did not change"
    );
    assert_ne!(
        before.offsets, after.offsets,
        "the face moved with the glass"
    );
}

#[test]
fn hue_change_damages_the_layer_once() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let mapped = layer(&mut f, "bar");
    mapped.set_shell_hue(OutputHue::default());
    let start = mapped.remap_damage_commit();
    mapped.set_shell_hue(OutputHue {
        blend: red(),
        animating: false,
    });
    let changed = mapped.remap_damage_commit();
    assert_ne!(start, changed);
    mapped.set_shell_hue(OutputHue {
        blend: red(),
        animating: false,
    });
    assert_eq!(
        changed,
        mapped.remap_damage_commit(),
        "an unchanged hue adds no damage"
    );
}

#[test]
fn a_running_fade_keeps_the_layer_animating() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    open_layer(&mut f, id, "other", None);
    let fading = OutputHue {
        blend: red(),
        animating: true,
    };
    layer(&mut f, "bar").set_shell_hue(fading);
    layer(&mut f, "other").set_shell_hue(fading);
    assert!(layer(&mut f, "bar").are_animations_ongoing());
    assert!(
        !layer(&mut f, "other").are_animations_ongoing(),
        "no rule, nothing to animate"
    );
}

#[test]
fn reload_without_the_rule_and_its_material_drops_the_remap() {
    // Glass off: Prism removes both the material and the rule (plan review
    // round 1). The layer must rebuild against the new rules, not panic.
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    assert!(layer(&mut f, "bar").remap_uniforms().is_some());
    f.niri_state()
        .reload_config(Ok(config("animations { off; }")));
    assert!(layer(&mut f, "bar").remap_uniforms().is_none());
}

#[test]
fn reload_renaming_the_material_follows_the_rule() {
    let mut f = Fixture::with_config(look(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_layer(&mut f, id, "bar", None);
    let text = format!(
        "animations {{ off; }}\n{}\n{}",
        GLASS.replace("\"tg\"", "\"tg2\""),
        RULE.replace("\"tg\"", "\"tg2\"")
    );
    f.niri_state().reload_config(Ok(config(&text)));
    let remap = layer(&mut f, "bar").rules().surface_remap.clone().unwrap();
    assert_eq!(remap.material.name, "tg2");
    assert!(layer(&mut f, "bar").remap_uniforms().is_some());
}

/// The output's render elements, front to back, as the real loop renders them.
fn output_elements(f: &mut Fixture) -> Vec<Id> {
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
            niri.render_to_vec(ctx, &output, false)
                .iter()
                .map(|e| e.id().clone())
                .collect()
        })
        .unwrap()
}

#[test]
fn remap_damage_precedes_the_pixels_it_redraws() {
    // Elements are front to back, and smithay skips damage hidden behind a
    // declared opaque region (plan review round 1): each remap damage element
    // must come before the surface it redraws, popups included.
    let mut f = Fixture::with_config(look(""));
    // Fixture::with_config has no renderer; render_to_vec needs one.
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    let bar = open_layer(&mut f, id, "bar", None);

    let popup = f
        .client(id)
        .create_layer_popup(&bar, (200, 100))
        .surface
        .clone();
    f.client(id).popup(&popup).commit();
    f.roundtrip(id);
    {
        let client = f.client(id);
        client.set_opaque_region(&popup, 200, 100);
        let popup = client.popup(&popup);
        popup.attach_new_colored_buffer(0x15, 0x13, 0x10, 0xff);
        popup.set_size(200, 100);
        popup.ack_last_and_commit();
    }
    f.double_roundtrip(id);

    let (bar_id, popup_id, bar_damage, popup_damage) = {
        let mapped = layer(&mut f, "bar");
        let (popup, _) = PopupManager::popups_for_surface(mapped.surface().wl_surface())
            .next()
            .expect("the popup is mapped");
        (
            Id::from_wayland_resource(mapped.surface().wl_surface()),
            Id::from_wayland_resource(popup.wl_surface()),
            mapped.remap_damage_id().clone(),
            mapped.popup_remap_damage_id().clone(),
        )
    };
    let ids = output_elements(&mut f);
    let at = |id: &Id| {
        ids.iter()
            .position(|e| e == id)
            .unwrap_or_else(|| panic!("{id:?} is not rendered"))
    };
    assert!(
        at(&popup_damage) < at(&popup_id),
        "popup damage sits in front of the popup"
    );
    assert!(
        at(&bar_damage) < at(&bar_id),
        "layer damage sits in front of the layer"
    );

    // A hue-only change, with the popup's buffer untouched, damages it.
    let mapped = layer(&mut f, "bar");
    let before = mapped.popup_remap_damage_commit();
    mapped.set_shell_hue(OutputHue {
        blend: red(),
        animating: false,
    });
    assert_ne!(before, mapped.popup_remap_damage_commit());
}
