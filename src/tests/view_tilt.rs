//! Frozen-clock renders of the rigid tilt (docs/specs/2026-10-09-transient-rigid-tilt-design.md
//! §8.2), through the real material shader in the headless backend, as ring_pair.rs renders.
//! Set `VIEW_TILT_DUMP=<dir>` to write each render as `<name>.png`. Every fixture gets the next
//! jelly seed; nextest runs each test in its own process, so dumps compare across commits.
//! Run through `just test-one`, never `cargo test`.

use std::time::Duration;

use glam::{DVec2, DVec3};
use niri_config::{Config, ResolvedGlass};
use smithay::backend::renderer::element::Element as _;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
use super::glass_edge::{self, renderer_name, BACKDROP};
pub(super) use super::glass_edge::{CLEAR, OPAQUE, OUT_H, OUT_W, TRANSLUCENT};
pub(super) use super::ring_look::gain;
use super::ring_pair::{diff, render_at, render_through, set_time, with_elements_at};
use super::*;
use crate::layout::tile::Tile;
use crate::niri::Niri;
use crate::render_helpers::material::view::{footprint, hooks, Camera, ViewPose, VIEW_NEAR_CLIP};
use crate::render_helpers::material::{drawn_area, material_frame, ElementView, MaterialFrame};
use crate::render_helpers::RenderTarget;
use crate::window::mapped::MappedId;
use crate::window::Mapped;

/// The instant of the focus gain, and the swing's and the beam's start.
pub(super) const GAIN: Duration = Duration::from_secs(1);
/// Long after any run.
pub(super) const REST: Duration = Duration::from_secs(4);

/// The stock light glass with every optic §8.2 names on: reflection, the edge highlight, film
/// and glass grain, aberration.
pub(super) const STOCK: &str = r##"
                ior 1.5
                thickness 20
                attenuation-color "#dfe8ff"
                attenuation-distance 60
                bevel 12
                offset-x 6
                offset-y 6
                bevel-profile 2
                chromatic-aberration 0.3
                reflection 0.6
                edge-highlight 0.5
                noise 0.15 type="fine" site="film"
                noise 0.1 type="white" site="glass""##;
/// The accepted dark look's values (ring_look.rs), with grain at scale 8 so value noise runs.
pub(super) const LIVE: &str = r##"
                ior 1.28
                light-ior 4.5
                thickness 31.2
                attenuation-color "#0D1D1E"
                attenuation-distance 11
                chromatic-aberration 0.36
                roughness 0.24
                bevel 12
                bevel-profile 2
                offset-x -6
                offset-y -5
                noise 0.2 type="fine" site="film" scale=8"##;
/// The beam mid-run 150 ms after the gain.
pub(super) const BEAM: &str = r#"focus "ring-light"
                ring-beam-speed 600"#;

/// One case: the glass lines, the response lines, extra window-rule lines, the pane's content
/// and size.
#[derive(Clone)]
pub(super) struct Scene {
    pub glass: String,
    pub response: String,
    pub rules: String,
    pub content: u32,
    pub size: (u16, u16),
    /// The workspace background behind the pane.
    pub backdrop: &'static str,
    /// An opaque red subsurface of the pane at (x, y), w × h, relative to its top-left: the
    /// texture then reaches past the slab (§6, §8.2 "Containment").
    pub subsurface: Option<[i32; 4]>,
    /// The output's size.
    pub output: (u16, u16),
    /// Whether a second window opens to hold focus until the gain. A pose-hook case without a
    /// gain can leave it out, so nothing but backdrop lies beside the pane.
    pub second: bool,
}

impl Scene {
    pub fn new(glass: &str, response: &str) -> Self {
        Self {
            glass: glass.to_owned(),
            response: response.to_owned(),
            rules: String::new(),
            content: TRANSLUCENT,
            size: (640, 480),
            backdrop: BACKDROP,
            subsurface: None,
            output: (OUT_W, OUT_H),
            second: true,
        }
    }
}

fn config(scene: &Scene) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps 40
            center-focused-column "never"
            background-color "{backdrop}"
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        material "tilt" {{
            glass {{
                {glass}
                backdrop-blur false
            }}
            response "default" {{
                accent "none"
                {response}
            }}
        }}
        window-rule {{
            material "tilt"
            background-effect {{ blur false; noise 0; saturation 1; }}
            {rules}
        }}
        "##,
        backdrop = scene.backdrop,
        glass = scene.glass,
        response = scene.response,
        rules = scene.rules,
    ))
    .unwrap()
}

/// Sets the pose hook for its lifetime.
pub(super) struct PoseHook;

impl PoseHook {
    pub fn set(pose: Option<ViewPose>) -> Self {
        hooks::POSE.with(|hook| hook.set(pose));
        Self
    }
}

impl Drop for PoseHook {
    fn drop(&mut self) {
        hooks::POSE.with(|hook| hook.set(None));
    }
}

/// The pane a fixture opened: its client and surface, and the id its tile's window carries.
pub(super) struct Pane {
    pub client: ClientId,
    pub surface: WlSurface,
    pub window: MappedId,
}

/// The fixture for `scene`: the pane, then, with `scene.second`, a second window that holds
/// focus until the gain.
pub(super) fn fixture_pane(scene: &Scene) -> (Fixture, Pane) {
    let mut f = glass_edge::fixture_on(config(scene), scene.output);
    let client = f.add_client();
    let (surface, window) = open(&mut f, client, scene.size, scene.content, scene.subsurface);
    if scene.second {
        open(&mut f, client, (400, 300), scene.content, None);
    }
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    (
        f,
        Pane {
            client,
            surface,
            window,
        },
    )
}

/// `fixture_pane`'s fixture alone.
pub(super) fn fixture(scene: &Scene) -> Fixture {
    fixture_pane(scene).0
}

pub(super) const RED: u32 = 0xffff_0000;

/// The ids of every mapped window.
fn window_ids(f: &mut Fixture) -> Vec<MappedId> {
    f.niri().layout.windows().map(|(_, w)| w.id()).collect()
}

/// Opens a window as `glass_edge::open` does and returns it with its id. With `subsurface`,
/// the window also gets an opaque red subsurface and an xdg window geometry of its own size:
/// without one, smithay takes the surface tree's bbox, and the window would grow to cover
/// the subsurface instead of the subsurface lying outside it.
fn open(
    f: &mut Fixture,
    id: ClientId,
    (w, h): (u16, u16),
    content: u32,
    subsurface: Option<[i32; 4]>,
) -> (WlSurface, MappedId) {
    let before = window_ids(f);
    let surface = glass_edge::open(f, id, (w, h), content);
    if let Some([x, y, sw, sh]) = subsurface {
        // Sync mode: the subsurface and the geometry land with the parent's next commit.
        let client = f.client(id);
        let sub = client.state.create_subsurface(&surface, x, y);
        let (shm, qh) = (client.state.shm.clone().unwrap(), client.qh.clone());
        let pixels = vec![RED; (sw * sh) as usize];
        super::client::attach_shm_pixels(&shm, &qh, &sub, sw, sh, &pixels);
        sub.commit();
        let window = f.client(id).window(&surface);
        window.set_geometry(0, 0, i32::from(w), i32::from(h));
        window.commit();
        f.double_roundtrip(id);
    }
    let mut new = window_ids(f).into_iter().filter(|w| !before.contains(w));
    let window = new.next().expect("the window mapped");
    assert_eq!(new.next(), None, "one window mapped");
    (surface, window)
}

/// Renders `scene` at `frames` after a gain at `GAIN`, under the pose hook `pose`.
pub(super) fn renders(scene: &Scene, pose: Option<ViewPose>, frames: &[Duration]) -> Vec<Vec<u8>> {
    let _hook = PoseHook::set(pose);
    let mut f = fixture(scene);
    gain(&mut f);
    frames.iter().map(|t| render_at(&mut f, *t)).collect()
}

/// Like `render_at`, through the output path: the render creates its signal sink, so the view
/// crop applies (§6).
pub(super) fn render_output_at(f: &mut Fixture, time: Duration) -> Vec<u8> {
    render_through(f, time, RenderTarget::Output)
}

/// The tile holding `window` on the first workspace, and its render position.
pub(super) fn pane_tile(niri: &Niri, window: MappedId) -> (&Tile<Mapped>, Point<f64, Logical>) {
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    workspace
        .tiles_with_render_positions()
        .find(|(tile, _, _)| tile.window().id() == window)
        .map(|(tile, pos, _)| (tile, pos))
        .expect("the pane's tile")
}

/// The material element geometry of `window`'s tile in a render at `time` through `target`.
pub(super) fn pane_geometry(
    f: &mut Fixture,
    window: MappedId,
    time: Duration,
    target: RenderTarget,
) -> Rectangle<i32, Physical> {
    let id = pane_tile(f.niri(), window)
        .0
        .material()
        .expect("a glass pane")
        .id()
        .clone();
    with_elements_at(f, time, target, |_, elements, _, scale| {
        elements
            .iter()
            .find(|e| e.id() == &id)
            .map(|e| e.geometry(scale))
            .expect("the pane's material element")
    })
}

/// `window`'s rect on the output, logical px.
pub(super) fn pane_rect(f: &mut Fixture, window: MappedId) -> Rectangle<f64, Logical> {
    let (tile, pos) = pane_tile(f.niri(), window);
    Rectangle::new(pos + tile.window_loc(), tile.animated_window_size())
}

/// `window`'s material frame as the element computes it, at scale 1, with the texture
/// covering the window and its subsurface.
pub(super) fn slab_frame(
    f: &mut Fixture,
    window: MappedId,
    glass: &ResolvedGlass,
    subsurface: Option<[i32; 4]>,
) -> MaterialFrame {
    let win = pane_rect(f, window);
    let tex = subsurface.map_or(win, |[x, y, w, h]| {
        win.merge(Rectangle::new(
            win.loc + Point::from((f64::from(x), f64::from(y))),
            Size::from((f64::from(w), f64::from(h))),
        ))
    });
    material_frame(win, tex, glass, 1.)
}

/// The resolved glass of `window`'s material.
pub(super) fn pane_glass(f: &mut Fixture, window: MappedId) -> ResolvedGlass {
    let (tile, _) = pane_tile(f.niri(), window);
    tile.material().expect("a glass pane").material().glass
}

pub(super) fn dump(name: &str, pixels: &[u8], (w, h): (u16, u16)) {
    let Some(dir) = std::env::var_os("VIEW_TILT_DUMP") else {
        return;
    };
    let path = std::path::Path::new(&dir).join(format!("{name}.png"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    crate::utils::write_png_rgba8(file, w.into(), h.into(), pixels).unwrap();
}

/// `glass` with each of `nodes` in place of the line naming the same node, or appended. KDL
/// rejects a repeated single node, so a variant cannot simply append `bevel 0` to a look
/// that already sets `bevel`. Only for single nodes: `noise` may repeat and is not replaced.
pub(super) fn set_nodes(glass: &str, nodes: &[&str]) -> String {
    let key = |line: &str| line.split_whitespace().next().unwrap_or("").to_owned();
    let mut lines: Vec<String> = glass.lines().map(str::to_owned).collect();
    for node in nodes {
        let line = format!("                {node}");
        match lines.iter_mut().find(|l| key(l) == key(node)) {
            Some(existing) => *existing = line,
            None => lines.push(line),
        }
    }
    lines.join("\n")
}

/// No bevel and no offset (the offsets may not exceed the bevel).
pub(super) fn bevel0(glass: &str) -> String {
    set_nodes(glass, &["bevel 0", "offset-x 0", "offset-y 0"])
}

pub(super) fn thickness0(glass: &str) -> String {
    set_nodes(glass, &["thickness 0"])
}

/// `glass` without its `noise` layers.
pub(super) fn grain0(glass: &str) -> String {
    let lines: Vec<&str> = glass
        .lines()
        .filter(|l| l.split_whitespace().next() != Some("noise"))
        .collect();
    lines.join("\n")
}

/// A real swing through the gain: the moving frames of §8.2's sequence.
pub(super) const SWING_MS: [u64; 6] = [50, 171, 300, 450, 572, 800];

pub(super) fn swing_frames() -> Vec<Duration> {
    let mut frames = vec![GAIN];
    frames.extend(SWING_MS.map(|ms| GAIN + Duration::from_millis(ms)));
    frames.push(REST);
    frames
}

/// The RGBA pixels of `rect`, row by row, from a `w` × `h` render that contains it.
fn region(pixels: &[u8], (w, h): (u16, u16), rect: Rectangle<i32, Physical>) -> Vec<u8> {
    let (w, h) = (i32::from(w), i32::from(h));
    assert!(
        rect.loc.x >= 0
            && rect.loc.y >= 0
            && rect.loc.x + rect.size.w <= w
            && rect.loc.y + rect.size.h <= h,
        "{rect:?} leaves the {w} × {h} render"
    );
    (rect.loc.y..rect.loc.y + rect.size.h)
        .flat_map(|y| {
            let start = ((y * w + rect.loc.x) * 4) as usize;
            pixels[start..start + (rect.size.w * 4) as usize]
                .iter()
                .copied()
        })
        .collect()
}

/// Pixels that are pure black: what a NaN reaching the output shows as on llvmpipe, and what
/// no glass over this backdrop is.
fn black(pixels: &[u8]) -> usize {
    pixels
        .chunks_exact(4)
        .filter(|p| p[0] == 0 && p[1] == 0 && p[2] == 0)
        .count()
}

/// The identity cases of §8.2: the beam, aberration, reflection and grain on, and again for
/// bevel 0, thickness 0 and asymmetric corner radii, on translucent, clear and opaque content.
fn identity_scenes() -> Vec<(String, Scene)> {
    let mut out = Vec::new();
    for (look, glass) in [("stock", STOCK), ("live", LIVE)] {
        for (content, px) in [
            ("translucent", TRANSLUCENT),
            ("clear", CLEAR),
            ("opaque", OPAQUE),
        ] {
            out.push((
                format!("{look}-{content}"),
                Scene {
                    content: px,
                    ..Scene::new(glass, BEAM)
                },
            ));
        }
        out.push((format!("{look}-bevel0"), Scene::new(&bevel0(glass), BEAM)));
        out.push((
            format!("{look}-thickness0"),
            Scene::new(&thickness0(glass), BEAM),
        ));
        out.push((
            format!("{look}-radii"),
            Scene {
                rules: "geometry-corner-radius 88 12 12 12\n            clip-to-geometry true"
                    .to_owned(),
                ..Scene::new(glass, BEAM)
            },
        ));
        out.push((
            format!("{look}-radii-small"),
            Scene {
                rules: "geometry-corner-radius 88 12 12 12\n            clip-to-geometry true"
                    .to_owned(),
                size: (100, 100),
                ..Scene::new(glass, BEAM)
            },
        ));
    }
    out
}

#[test]
fn every_identity_scene_parses_and_replaces_its_nodes() {
    for (name, scene) in identity_scenes() {
        for node in ["bevel", "thickness", "offset-x", "offset-y"] {
            let count = scene
                .glass
                .lines()
                .filter(|l| l.split_whitespace().next() == Some(node))
                .count();
            assert!(count <= 1, "{name}: {count} `{node}` nodes");
        }
        let _ = config(&scene);
    }
    assert!(bevel0(STOCK).contains("bevel 0") && !bevel0(STOCK).contains("bevel 12"));
}

#[test]
fn the_view_branch_at_the_identity_is_byte_identical_to_rest() {
    eprintln!(
        "renderer: {}",
        renderer_name(&mut fixture(&Scene::new(STOCK, BEAM)))
    );
    let frames = [GAIN, GAIN + Duration::from_millis(150), REST];
    let identity = ViewPose::identity(DVec2::new(0.6, 0.8), 6.);
    let mut failures = Vec::new();
    for (name, scene) in identity_scenes() {
        let rest = renders(&scene, None, &frames);
        let forced = renders(&scene, Some(identity), &frames);
        for (i, (a, b)) in rest.iter().zip(&forced).enumerate() {
            dump(&format!("identity/{name}-{i}-rest"), a, (OUT_W, OUT_H));
            dump(&format!("identity/{name}-{i}-forced"), b, (OUT_W, OUT_H));
            let (px, max) = diff(a, b);
            if px != 0 {
                failures.push(format!("{name} frame {i}: {px} px differ, max {max}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §8.2 "Zero rise in motion": no NaN reaches a pixel, and coverage stays in [0, 1]. The render
/// shows a NaN in the shading as black, but not a bad coverage: a NaN one fails `coverage > 0`
/// and drops the glass, and one outside [0, 1] is clamped away over the opaque backdrop. So
/// `zero_rise_is_the_box_or_the_plane` holds the mirror's coverage in [0, 1] for these solids.
#[test]
fn a_swing_with_zero_rise_draws_no_nan() {
    for (name, glass) in [("bevel0", bevel0(STOCK)), ("thickness0", thickness0(STOCK))] {
        let scene = Scene::new(&glass, "view-tilt 14\n                view-distance 2.5");
        let frames = renders(&scene, None, &swing_frames());
        let rest = frames.last().unwrap();
        for (i, f) in frames.iter().enumerate() {
            dump(&format!("zero-rise/{name}-{i}"), f, (OUT_W, OUT_H));
            assert_eq!(black(f), 0, "{name} frame {i}: black pixels");
        }
        assert!(
            diff(&frames[2], rest).0 > 0,
            "{name}: the peak frame did not move"
        );
    }
}

/// The swing at the middle cell of the still sheet: 10° at κ 6, with no focus light, so a
/// frame shows the swing alone.
pub(super) const SWING: &str =
    "focus \"none\"\n                view-tilt 10\n                view-distance 6";
/// The same response without the swing.
pub(super) const STILL: &str = "focus \"none\"";

#[test]
fn a_swing_settles_to_the_rest_render_and_matches_view_tilt_zero_after() {
    let frames = vec![
        GAIN,
        GAIN + Duration::from_millis(171),
        GAIN + Duration::from_millis(900),
        GAIN + Duration::from_millis(1000),
        REST,
    ];
    let tilted = renders(&Scene::new(LIVE, SWING), None, &frames);
    let still = renders(&Scene::new(LIVE, STILL), None, &frames);
    for (i, f) in tilted.iter().enumerate() {
        dump(&format!("settle/{i}"), f, (OUT_W, OUT_H));
    }
    assert!(
        diff(&tilted[1], &still[1]).0 > 0,
        "the peak frame did not move"
    );
    for i in [2, 3, 4] {
        assert_eq!(
            diff(&tilted[i], &still[i]),
            (0, 0),
            "frame {i} after the run differs from view-tilt 0"
        );
    }
    assert_eq!(
        diff(&tilted[2], &tilted[4]),
        (0, 0),
        "the frame at settle is not rest"
    );
}

#[test]
fn after_the_run_nothing_animates_and_the_geometry_returns() {
    let scene = Scene::new(LIVE, SWING);
    let (mut f, pane) = fixture_pane(&scene);
    gain(&mut f);
    let glass = pane_glass(&mut f, pane.window);
    let settle = Duration::from_millis(900);
    // The gain's first frame starts the swing; its direction runs from the other window's
    // centre to the pane's, which the two heights tilt off the horizontal.
    pane_geometry(&mut f, pane.window, GAIN, RenderTarget::ScreenCapture);
    let (started, dir) = pane_tile(f.niri(), pane.window)
        .0
        .view_swing_start()
        .expect("the gain started a swing");
    assert_eq!(started, GAIN);
    let dir = DVec2::from(dir);
    for ms in [0u64, 50, 171, 300, 450, 572, 800, 900, 1500, 3000, 10_000] {
        let at = GAIN + Duration::from_millis(ms);
        let got = pane_geometry(&mut f, pane.window, at, RenderTarget::ScreenCapture);
        let frame = slab_frame(&mut f, pane.window, &glass, None);
        let view =
            ViewPose::at(10., settle, Duration::from_millis(ms), dir, 6.).map(|pose| ElementView {
                pose,
                crop: None,
                wall_spill: 1.,
                grow: 0.,
            });
        let want =
            drawn_area(&frame, view.as_ref(), glass.thickness, 1.).to_physical_precise_round(1.);
        assert_eq!(got, want, "{ms} ms");
        if ms >= 900 {
            assert_eq!(
                got,
                frame.area.to_physical_precise_round(1.),
                "{ms} ms: back to the resting area"
            );
            let (tile, _) = pane_tile(f.niri(), pane.window);
            assert!(!tile.are_animations_ongoing(), "{ms} ms");
        }
    }
}

#[test]
fn nothing_draws_outside_the_computed_geometry() {
    // §8.2 "Containment": a subsurface outside the slab, every projected pixel of it drawn; then
    // a subsurface past the near clip, drawn up to the clip and nowhere past it. The first lies
    // beyond the pane on the side that lifts, so its projection leaves both the resting area and
    // the slab box's own projection (§6: projecting the slab alone is not enough).
    let peak = |deg: f64, kappa: f64| ViewPose {
        theta: deg.to_radians(),
        dir: DVec2::X,
        weight: 1.,
        kappa,
    };
    for (name, scene, pose) in [
        (
            "subsurface",
            Scene {
                subsurface: Some([420, 100, 120, 80]),
                size: (400, 300),
                second: false,
                ..Scene::new(STOCK, "")
            },
            peak(10., 6.),
        ),
        (
            "near-clip",
            Scene {
                subsurface: Some([110, 20, 500, 60]),
                size: (100, 100),
                output: (2560, 1440),
                second: false,
                ..Scene::new(STOCK, "")
            },
            peak(20., 1.5),
        ),
    ] {
        let (mut f, pane) = fixture_pane(&scene);
        let glass = pane_glass(&mut f, pane.window);
        let win = pane_rect(&mut f, pane.window);
        let (w, h) = scene.size;
        assert_eq!(
            win.size,
            Size::from((f64::from(w), f64::from(h))),
            "{name}: the subsurface grew the window"
        );
        let frame = slab_frame(&mut f, pane.window, &glass, scene.subsurface);
        assert!(
            !frame.slab.contains_rect(frame.tex),
            "{name}: the texture must reach past the slab"
        );
        // The element's texture covers the subsurface as `slab_frame` takes it.
        assert_eq!(
            pane_geometry(&mut f, pane.window, REST, RenderTarget::Output),
            frame.area.to_physical_precise_round(1.),
            "{name}: the resting area"
        );
        let slab = frame.slab;
        let camera = Camera::new(
            &pose,
            DVec2::new(slab.loc.x, slab.loc.y),
            DVec2::new(slab.size.w, slab.size.h),
        );
        let [sx, sy, sw, sh] = scene.subsurface.unwrap().map(f64::from);
        let sub = (
            DVec2::new(win.loc.x + sx, win.loc.y + sy),
            DVec2::new(win.loc.x + sx + sw, win.loc.y + sy + sh),
        );
        let corners = [
            sub.0,
            DVec2::new(sub.1.x, sub.0.y),
            sub.1,
            DVec2::new(sub.0.x, sub.1.y),
        ];
        let past_clip = corners
            .iter()
            .filter(|c| camera.zeta(camera.to_world(c.extend(0.))) > VIEW_NEAR_CLIP)
            .count();
        if name == "near-clip" {
            assert!(
                past_clip > 0,
                "the case must put the subsurface past the near clip"
            );
        } else {
            assert_eq!(
                past_clip, 0,
                "the case must keep the subsurface inside the near clip"
            );
            let (lo, hi) = corners
                .map(|c| camera.to_screen(c.extend(0.)))
                .into_iter()
                .fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), s| {
                    (lo.min(s), hi.max(s))
                });
            let projected = Rectangle::from_extremities((lo.x, lo.y), (hi.x, hi.y));
            let box_ = (
                DVec2::new(slab.loc.x, slab.loc.y),
                DVec2::new(slab.loc.x + slab.size.w, slab.loc.y + slab.size.h),
            );
            let (lo, hi) = footprint(&camera, box_, box_, glass.thickness);
            let slab_alone = Rectangle::from_extremities((lo.x, lo.y), (hi.x, hi.y));
            assert!(
                !frame.area.contains_rect(projected) && !slab_alone.contains_rect(projected),
                "the subsurface's projection {projected:?} must leave the resting area {:?} and \
                 the slab's projection {slab_alone:?}",
                frame.area
            );
        }

        let _hook = PoseHook::set(Some(pose));
        let exact = render_output_at(&mut f, REST);
        let geometry = pane_geometry(&mut f, pane.window, REST, RenderTarget::Output);
        hooks::GROW.with(|g| g.set(64.));
        let grown = render_output_at(&mut f, REST);
        hooks::GROW.with(|g| g.set(0.));
        dump(&format!("containment/{name}-exact"), &exact, scene.output);
        dump(&format!("containment/{name}-grown"), &grown, scene.output);
        let (w, h) = (i32::from(scene.output.0), i32::from(scene.output.1));
        let at = |pixels: &[u8], x: i32, y: i32| {
            let i = ((y * w + x) * 4) as usize;
            [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
        };
        assert!(
            geometry.loc.x >= -1
                && geometry.loc.y >= -1
                && geometry.loc.x + geometry.size.w <= w + 1
                && geometry.loc.y + geometry.size.h <= h + 1,
            "{name}: geometry {geometry:?} leaves the output"
        );
        let mut outside = 0;
        for y in 0..h {
            for x in 0..w {
                if !geometry.contains((x, y)) && at(&exact, x, y) != at(&grown, x, y) {
                    outside += 1;
                }
            }
        }
        assert_eq!(
            outside, 0,
            "{name}: the oversized quad drew outside the geometry"
        );

        let red = [255, 0, 0, 255];
        if name == "subsurface" {
            // Every pixel whose centre lies inside the subsurface's projection by 1.5 px is red.
            let corners = corners.map(|c| camera.to_screen(c.extend(0.)));
            let inside = |p: DVec2| {
                (0..4).all(|i| {
                    let (a, b) = (corners[i], corners[(i + 1) % 4]);
                    let edge = (b - a).normalize();
                    edge.perp_dot(p - a) >= 1.5
                })
            };
            let mut checked = 0;
            for y in 0..h {
                for x in 0..w {
                    if inside(DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5)) {
                        assert_eq!(
                            at(&exact, x, y),
                            red,
                            "the subsurface is not drawn at ({x}, {y})"
                        );
                        checked += 1;
                    }
                }
            }
            assert!(checked > 1000, "{checked} subsurface pixels checked");
        } else {
            // The decal's projection ends where its depth reaches 3/4 of the camera distance.
            // The row through the pivot projects to one screen row at every magnification.
            let row = camera.pivot.y;
            assert!(
                row > sub.0.y && row < sub.1.y,
                "the case must put the pivot's row in the subsurface"
            );
            let mut x_clip = f64::NEG_INFINITY;
            let mut x = sub.0.x;
            while x <= sub.1.x {
                let e = camera.to_world(DVec3::new(x, row, 0.));
                if camera.zeta(e) <= VIEW_NEAR_CLIP {
                    x_clip = x_clip.max(camera.project(e).x);
                }
                x += 0.25;
            }
            let y = row as i32;
            let start = camera
                .to_screen(DVec2::new(sub.0.x, row).extend(0.))
                .x
                .ceil() as i32
                + 2;
            for px in start..(x_clip.floor() as i32 - 2) {
                assert_eq!(
                    at(&exact, px, y),
                    red,
                    "not drawn up to the clip at x {px} (clip {x_clip})"
                );
            }
            for px in (x_clip.ceil() as i32 + 2)..w {
                assert_ne!(
                    at(&exact, px, y),
                    red,
                    "drawn past the clip at x {px} (clip {x_clip})"
                );
            }
        }
    }
}

#[test]
fn the_view_crop_follows_the_tile_into_an_offscreen() {
    // §8.2 "View crop": a swing during an alpha animation, well away from the output's edges,
    // matches the same frame rendered directly; the bevel at negative offscreen coordinates is
    // drawn in full. LIVE's offsets put the slab up and left of the window, so the band lies
    // there; STOCK's put its corner on the tile's origin. Without grain: it is seeded per
    // framebuffer pixel, which the offscreen's origin moves, so with grain on the two renders
    // differ by up to 59 levels at rest already.
    let scene = Scene {
        size: (500, 400),
        ..Scene::new(&grain0(LIVE), "")
    };
    let pose = ViewPose {
        theta: 10f64.to_radians(),
        dir: DVec2::new(0.6, 0.8),
        weight: 1.,
        kappa: 6.,
    };
    let _hook = PoseHook::set(Some(pose));
    let (mut direct, pane) = fixture_pane(&scene);
    let want = render_output_at(&mut direct, REST);
    let full = pane_geometry(&mut direct, pane.window, REST, RenderTarget::ScreenCapture);
    let (_, pos) = pane_tile(direct.niri(), pane.window);
    let pos: Point<i32, Physical> = pos.to_physical_precise_round(1.);
    assert!(
        full.loc.x < pos.x - 1 && full.loc.y < pos.y - 1,
        "the case must draw past the offscreen's crop margin: {full:?} for a tile at {pos:?}"
    );
    let (mut through, pane) = fixture_pane(&scene);
    {
        let anim = through.niri().config.borrow().animations.window_movement.0;
        let niri = through.niri();
        let ws = niri.layout.workspaces_mut().next().unwrap();
        let tile = ws
            .tiles_mut()
            .find(|tile| tile.window().id() == pane.window)
            .unwrap();
        tile.animate_alpha(1., 1., anim);
        tile.hold_alpha_animation_after_done();
    }
    let got = render_output_at(&mut through, REST);
    dump("crop/direct", &want, (OUT_W, OUT_H));
    dump("crop/alpha", &got, (OUT_W, OUT_H));
    let (px, max) = diff(&want, &got);
    assert!(max <= 1, "{px} px differ, by up to {max} levels");
}

#[test]
fn a_tile_partly_off_the_left_edge_is_cropped_there_only() {
    let scene = Scene {
        size: (1000, 400),
        ..Scene::new(STOCK, "")
    };
    let pose = ViewPose {
        theta: -14f64.to_radians(),
        dir: DVec2::X,
        weight: 1.,
        kappa: 2.5,
    };
    let _hook = PoseHook::set(Some(pose));
    let (mut f, pane) = fixture_pane(&scene);
    // Focus stays on the second window, so the view scrolls the pane partly off the left.
    let full = pane_geometry(&mut f, pane.window, REST, RenderTarget::ScreenCapture);
    let cropped = pane_geometry(&mut f, pane.window, REST, RenderTarget::Output);
    assert!(
        full.loc.x < -1,
        "the case must reach past the left edge: {full:?}"
    );
    assert_eq!(cropped.loc.x, -1, "cropped one physical px past the edge");
    assert_eq!(cropped.loc.y, full.loc.y);
    assert_eq!(cropped.loc.y + cropped.size.h, full.loc.y + full.size.h);
    assert_eq!(
        cropped.loc.x + cropped.size.w,
        full.loc.x + full.size.w,
        "and nowhere else"
    );
}

#[test]
fn a_close_snapshot_mid_swing_keeps_the_whole_posed_footprint() {
    // §8.2 "View crop", through a real unmap mid-swing: the snapshot draws the tile at (0, 0)
    // with no view, so nothing crops it, and the close animation's first frame matches the
    // posed frame before it. LIVE's slab reaches up and left of the tile's origin, which a crop
    // taken as the output rect in the snapshot's coordinates would cut; grain is off as in
    // `the_view_crop_follows_the_tile_into_an_offscreen`. Only the pane's footprint is
    // compared: the other window takes focus at the close, which changes its own pixels.
    let scene = Scene::new(&grain0(LIVE), SWING);
    let (mut f, pane) = fixture_pane(&scene);
    gain(&mut f);
    let at = GAIN + Duration::from_millis(171);
    render_at(&mut f, GAIN);
    let before = render_at(&mut f, at);
    let full = pane_geometry(&mut f, pane.window, at, RenderTarget::ScreenCapture);
    let (tile, pos) = pane_tile(f.niri(), pane.window);
    assert!(tile.view_pose().is_some(), "the pane must be mid-swing");
    let pos: Point<i32, Physical> = pos.to_physical_precise_round(1.);
    assert!(
        full.loc.x < pos.x - 1 && full.loc.y < pos.y - 1,
        "the case must draw past the tile's origin: {full:?} for a tile at {pos:?}"
    );
    let window = f.client(pane.client).window(&pane.surface);
    window.attach_null();
    window.commit();
    f.double_roundtrip(pane.client);
    assert!(
        !window_ids(&mut f).contains(&pane.window),
        "the pane did not unmap"
    );
    let after = render_at(&mut f, at);
    dump("crop/close-before", &before, (OUT_W, OUT_H));
    dump("crop/close-after", &after, (OUT_W, OUT_H));
    let (px, max) = diff(
        &region(&before, (OUT_W, OUT_H), full),
        &region(&after, (OUT_W, OUT_H), full),
    );
    assert!(
        max <= 2,
        "the close animation does not start from the posed frame: {px} px, up to {max}"
    );
}
