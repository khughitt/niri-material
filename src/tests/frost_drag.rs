//! A glass window keeps its frost while it is dragged by interactive move
//! (material-e88df7).
//!
//! A tiled drag fades its tile to the drag alpha through the alpha offscreen,
//! which lets a quarter of the sharp scene through the blurred glass (research
//! in material-5d92cb). A tile with a material therefore stays opaque for the
//! whole move.
//!
//! Two frosted windows over a 16 px checker placed within the backdrop. One is
//! dragged onto bare wallpaper and rendered at a frozen instant; frost is
//! measured as high-frequency energy in the glass face. Set
//! `FROST_DRAG_DUMP=<dir>` for PNGs.

use std::time::Duration;

use niri_config::Config;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use smithay::utils::{Logical, Point, Rectangle};

use super::client::{ClientId, LayerConfigureProps};
use super::ring_pair::{render_at, set_time};
use super::*;

const OUT_W: u16 = 1280;
const OUT_H: u16 = 720;
const W: u16 = 400;
const H: u16 = 300;
/// Where the pointer ends every drag: the dragged tile lands on bare
/// wallpaper, clear of the stationary window.
const TARGET: (f64, f64) = (1060., 530.);
/// Inset from the window rectangle to the measured face interior, clear of
/// the bevel and offset.
const INSET: f64 = 40.;
/// Past every animation a drag starts.
const AFTER: Duration = Duration::from_secs(5);

/// Windows get the frost material when `material` is set, and none otherwise.
fn config(material: bool) -> Config {
    let material = if material { r#"material "frost""# } else { "" };
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        blur {{
            passes 3
            offset 6
            noise 0
            saturation 1
        }}
        layout {{
            gaps 40
            background-color "transparent"
            default-column-width {{ fixed 400; }}
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        layer-rule {{
            match namespace="^frost-checker$"
            place-within-backdrop true
        }}
        material "frost" {{
            glass {{
                ior 1.5
                thickness 20
                attenuation-color "#dfe8ff"
                attenuation-distance 60
                bevel 12
                offset-x 6
                offset-y 6
                jelly-flex 0
                jelly-ripple 0
                backdrop-blur true
            }}
            response "default" {{
                accent "none"
                focus "none"
            }}
        }}
        window-rule {{
            {material}
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        "##
    ))
    .unwrap()
}

/// A 16 px checker of a warm and a cool colour (opaque ARGB).
fn checker(x: u16, y: u16) -> u32 {
    if (x / 16 + y / 16) % 2 == 0 {
        0xffc8_8a3c
    } else {
        0xff28_5ac8
    }
}

fn wallpaper(f: &mut Fixture, id: ClientId) {
    let layer = f
        .client(id)
        .create_layer(None, Layer::Background, "frost-checker");
    let surface = layer.surface.clone();
    layer.set_configure_props(LayerConfigureProps {
        anchor: Some(Anchor::Left | Anchor::Right | Anchor::Top | Anchor::Bottom),
        size: Some((0, 0)),
        ..Default::default()
    });
    layer.commit();
    f.roundtrip(id);
    let layer = f.client(id).layer(&surface);
    layer.attach_new_shm_pattern(OUT_W, OUT_H, checker);
    layer.ack_last_and_commit();
    f.double_roundtrip(id);
}

fn open(f: &mut Fixture, id: ClientId) {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(0);
    window.set_size(W, H);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
}

/// The focused (to-be-dragged) window's rectangle on the output.
fn focused_rect(f: &mut Fixture) -> Rectangle<f64, Logical> {
    let niri = f.niri();
    let focused = niri.layout.focus().unwrap().window.clone();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    workspace
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().window == focused)
        .map(|(tile, pos, _)| Rectangle::new(pos + tile.window_loc(), tile.animated_window_size()))
        .unwrap()
}

fn luma(p: &[u8], x: usize, y: usize) -> f64 {
    let i = (y * usize::from(OUT_W) + x) * 4;
    0.2126 * f64::from(p[i]) + 0.7152 * f64::from(p[i + 1]) + 0.0722 * f64::from(p[i + 2])
}

/// The face interior: the window inset by INSET, clipped to the output.
fn interior(r: Rectangle<f64, Logical>) -> (usize, usize, usize, usize) {
    let x0 = (r.loc.x + INSET).max(1.) as usize;
    let y0 = (r.loc.y + INSET).max(1.) as usize;
    let x1 = ((r.loc.x + r.size.w - INSET) as usize).min(usize::from(OUT_W) - 1);
    let y1 = ((r.loc.y + r.size.h - INSET) as usize).min(usize::from(OUT_H) - 1);
    assert!(x0 < x1 && y0 < y1, "empty interior for {r:?}");
    (x0, y0, x1, y1)
}

/// High-frequency energy: mean |luma - its 3x3 mean| over the interior.
fn hf(p: &[u8], r: Rectangle<f64, Logical>) -> f64 {
    let (x0, y0, x1, y1) = interior(r);
    let mut sum = 0.;
    let mut n = 0.;
    for y in y0..y1 {
        for x in x0..x1 {
            let mut m = 0.;
            for dy in 0..3 {
                for dx in 0..3 {
                    m += luma(p, x + dx - 1, y + dy - 1);
                }
            }
            sum += (luma(p, x, y) - m / 9.).abs();
            n += 1.;
        }
    }
    sum / n
}

/// Pixels of the interior that differ between two renders.
fn differing_pixels(a: &[u8], b: &[u8], r: Rectangle<f64, Logical>) -> usize {
    let (x0, y0, x1, y1) = interior(r);
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * usize::from(OUT_W) + x) * 4;
            if a[i..i + 4] != b[i..i + 4] {
                n += 1;
            }
        }
    }
    n
}

fn dump(name: &str, pixels: &[u8]) {
    let Some(dir) = std::env::var_os("FROST_DRAG_DUMP") else {
        return;
    };
    let path = std::path::Path::new(&dir).join(format!("{name}.png"));
    let file = std::fs::File::create(path).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    Tiled,
    Floating,
}

struct Render {
    pixels: Vec<u8>,
    /// The dragged window (at rest: where it sits in the layout).
    moved: Rectangle<f64, Logical>,
    /// The dragged tile's alpha; `None` when it carries no alpha animation.
    alpha: Option<f64>,
}

/// Opens two windows, drags the second (focused) one to TARGET as `drag`
/// says, and renders the frozen instant. `None` renders at rest.
fn scene(drag: Option<Drag>, material: bool) -> Render {
    let mut f = Fixture::with_config(config(material));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    let id = f.add_client();
    wallpaper(&mut f, id);
    open(&mut f, id);
    open(&mut f, id);
    f.niri_state().update_keyboard_focus();
    f.double_roundtrip(id);
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();

    if drag == Some(Drag::Floating) {
        f.niri().layout.toggle_window_floating(None);
        f.double_roundtrip(id);
        f.niri_complete_animations();
    }

    let moved = focused_rect(&mut f);
    if drag.is_none() {
        return Render {
            pixels: render_at(&mut f, AFTER),
            moved,
            alpha: None,
        };
    }

    let output = f.niri_output(1);
    let niri = f.niri();
    let window = niri.layout.focus().unwrap().window.clone();
    let start = moved.loc + Point::from((moved.size.w / 2., moved.size.h / 2.));
    assert!(niri
        .layout
        .interactive_move_begin(window.clone(), &output, start));
    let target = Point::from(TARGET);
    assert!(niri
        .layout
        .interactive_move_update(&window, target - start, output, target));
    f.double_roundtrip(id);

    // The clock stands still; rendering at AFTER runs every animation the
    // drag started (alpha, move) to its end.
    let pixels = render_at(&mut f, AFTER);
    let (moved, alpha) = f.niri().layout.interactive_move_window_rect().unwrap();
    Render {
        pixels,
        moved,
        alpha,
    }
}

#[test]
fn tiled_drag_keeps_glass_opaque() {
    let rest = scene(None, true);
    let tiled = scene(Some(Drag::Tiled), true);
    let floating = scene(Some(Drag::Floating), true);
    dump("rest", &rest.pixels);
    dump("tiled", &tiled.pixels);
    dump("floating", &floating.pixels);

    // No drag alpha, so no alpha offscreen.
    assert_eq!(tiled.alpha, None, "tiled drag of a glass window fades");
    assert_eq!(
        floating.alpha, None,
        "floating drag of a glass window fades"
    );

    // The bare checker under the dragged window carries energy the frost
    // removes.
    assert_eq!(
        tiled.moved, floating.moved,
        "the drags end in different places"
    );
    let bare = hf(&rest.pixels, tiled.moved);
    assert!(bare > 4., "bare checker energy {bare:.2}");
    let face = hf(&tiled.pixels, tiled.moved);
    assert!(
        face < 0.05,
        "tiled drag face energy {face:.2} (bare {bare:.2})"
    );

    // The tiled drag draws its glass exactly as the floating drag does.
    let differing = differing_pixels(&tiled.pixels, &floating.pixels, tiled.moved);
    assert_eq!(
        differing, 0,
        "tiled and floating drags differ in {differing} pixels"
    );
}

#[test]
fn tiled_drag_fades_window_without_material() {
    let tiled = scene(Some(Drag::Tiled), false);
    let alpha = tiled
        .alpha
        .expect("tiled drag of a plain window does not fade");
    assert!((alpha - 0.75).abs() < 1e-9, "drag alpha {alpha}");
}
