//! Frozen-clock renders of the rigid tilt (docs/specs/2026-10-09-transient-rigid-tilt-design.md
//! §8.2), through the real material shader in the headless backend, as ring_pair.rs renders.
//! Set `VIEW_TILT_DUMP=<dir>` to write each render as `<name>.png`. Every fixture gets the next
//! jelly seed; nextest runs each test in its own process, so dumps compare across commits.
//! Run through `just test-one`, never `cargo test`.

use std::sync::atomic::Ordering;
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
use crate::render_helpers::material::view::solid::tests::coverage;
use crate::render_helpers::material::view::solid::{outline, trace, Segment, Solid};
use crate::render_helpers::material::view::{
    footprint, hooks, swing_shape, Camera, ViewPose, VIEW_NEAR_CLIP,
};
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
    /// gain can leave it out, so nothing but backdrop lies beside the pane. It carries the
    /// title `second`, so a rule in `after` can match it.
    pub second: bool,
    /// Top-level config lines after `hotkey-overlay`.
    pub top: String,
    /// Config lines after the window rule, so a later rule can override it.
    pub after: String,
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
            top: String::new(),
            after: String::new(),
        }
    }
}

fn config(scene: &Scene) -> Config {
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        {top}
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
        {after}
        "##,
        top = scene.top,
        after = scene.after,
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

/// Sets the wall spill gain for its lifetime: a negative one is a shader probe (−1 walls, −2
/// coverage; see `probe`).
pub(super) struct SpillHook;

impl SpillHook {
    pub fn set(gain: f32) -> Self {
        hooks::WALL_SPILL.with(|g| g.set(gain));
        Self
    }
}

impl Drop for SpillHook {
    fn drop(&mut self) {
        hooks::WALL_SPILL.with(|g| g.set(1.));
    }
}

/// The pane a fixture opened: its client and surface, and the id its tile's window carries;
/// with `scene.second`, the second window's id.
pub(super) struct Pane {
    pub client: ClientId,
    pub surface: WlSurface,
    pub window: MappedId,
    pub second: Option<MappedId>,
}

/// The fixture for `scene`: the pane, then, with `scene.second`, a second window that holds
/// focus until the gain.
pub(super) fn fixture_pane(scene: &Scene) -> (Fixture, Pane) {
    let mut f = glass_edge::fixture_on(config(scene), scene.output);
    let client = f.add_client();
    let (surface, window) = open(&mut f, client, scene.size, scene.content, scene.subsurface);
    let second = scene.second.then(|| {
        let (surface, id) = open(&mut f, client, (400, 300), scene.content, None);
        let window = f.client(client).window(&surface);
        window.set_title("second");
        window.commit();
        f.double_roundtrip(client);
        id
    });
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
            second,
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

/// The pane's slab as the mirror sees it: a window with square corners, so the face radius is
/// 0 and the outer radius is the chamfer.
pub(super) fn mirror(
    frame: &MaterialFrame,
    glass: &ResolvedGlass,
    pose: &ViewPose,
) -> (Solid, Camera) {
    let slab = frame.slab;
    let half = DVec2::new(slab.size.w, slab.size.h) * 0.5;
    let center = DVec2::new(slab.loc.x, slab.loc.y) + half;
    let chamfer = f64::from(frame.chamfer);
    let solid = Solid {
        center,
        half,
        outer_r: [chamfer; 4],
        chamfer,
        thickness: glass.thickness,
        profile: glass.bevel_profile,
    };
    (solid, Camera::new(pose, center - half, 2. * half))
}

/// Renders the pane under `pose` with the probe `gain` (−1 walls, −2 coverage), clear content
/// over a black backdrop and no second window.
fn probe(look: &str, pose: ViewPose, gain: f32) -> (Vec<u8>, MaterialFrame, ResolvedGlass) {
    probe_sized(look, (640, 480), pose, gain)
}

fn probe_sized(
    look: &str,
    size: (u16, u16),
    pose: ViewPose,
    gain: f32,
) -> (Vec<u8>, MaterialFrame, ResolvedGlass) {
    let scene = Scene {
        content: CLEAR,
        backdrop: "#000000",
        second: false,
        size,
        ..Scene::new(look, "")
    };
    let _hook = PoseHook::set(Some(pose));
    let _probe = SpillHook::set(gain);
    let (mut f, pane) = fixture_pane(&scene);
    let pixels = render_at(&mut f, REST);
    let glass = pane_glass(&mut f, pane.window);
    let frame = slab_frame(&mut f, pane.window, &glass, None);
    (pixels, frame, glass)
}

fn px(pixels: &[u8], x: i32, y: i32) -> [u8; 4] {
    let i = ((y * i32::from(OUT_W) + x) * 4) as usize;
    [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
}

const MAGENTA: [u8; 4] = [255, 0, 255, 255];
/// The coverage probe's mark for a coverage outside [0, 1], NaN included.
const SENTINEL: [u8; 4] = [0, 255, 0, 255];

/// The coverage probe's out-of-range marks among `pixels`.
fn sentinels(pixels: &[u8]) -> usize {
    pixels.chunks_exact(4).filter(|p| *p == SENTINEL).count()
}

/// The coverage probe's tolerance against the mirror: the §4.5 side bound (1/16 px of outline
/// distance) times the smoothstep's steepest slope, 1.5, plus two 8-bit levels.
const COVERAGE_TOL: f64 = 1.5 / 16. + 2. / 255.;

#[test]
fn walls_show_where_the_mirror_predicts() {
    // §8.2 "Walls": at the peak, at 2° of a 10° swing at κ 6, and in the overshoot, along the
    // row through the pivot on each side edge: the wall pixels against the mirror's at pixel
    // centres, at most 1 px apart, and the wall's width against the mirror's ±1 px. The edge
    // along the lifting side (+x while θ > 0) is the advancing one: the mirror must show a wall
    // there, at least 1 px of it at the peak.
    let (peak_deg, kappa) = (10f64, 6.);
    for (name, shape) in [("peak", 1.), ("small", 0.2), ("overshoot", -0.1638)] {
        let pose = ViewPose {
            theta: peak_deg.to_radians() * shape,
            dir: DVec2::X,
            weight: f64::abs(shape),
            kappa,
        };
        let (pixels, frame, glass) = probe(LIVE, pose, -1.);
        dump(&format!("walls/{name}"), &pixels, (OUT_W, OUT_H));
        let (solid, camera) = mirror(&frame, &glass, &pose);
        let y = solid.center.y as i32;
        for sign in [1., -1.] {
            let edge =
                camera.to_screen((solid.center + DVec2::new(sign * solid.half.x, 0.)).extend(0.));
            let (lo, hi) = (edge.x as i32 - 40, edge.x as i32 + 40);
            let wall_at = |s: f64| {
                let seg = Segment::new(&camera, DVec2::new(s, f64::from(y) + 0.5))
                    .expect("a downward ray");
                let tr = trace(&solid, &seg);
                tr.wall && outline(&solid, &camera, &seg, &tr) < 0.
            };
            let apart = (lo..hi)
                .filter(|&x| (px(&pixels, x, y) == MAGENTA) != wall_at(f64::from(x) + 0.5))
                .count();
            assert!(
                apart <= 1,
                "{name} side {sign}: {apart} px where the render and the mirror disagree"
            );
            let rendered = (lo..hi).filter(|&x| px(&pixels, x, y) == MAGENTA).count() as f64;
            let mut predicted = 0.;
            let mut s = f64::from(lo);
            while s < f64::from(hi) {
                if wall_at(s) {
                    predicted += 1. / 64.;
                }
                s += 1. / 64.;
            }
            if sign == shape.signum() {
                let least = if name == "peak" {
                    1.
                } else {
                    f64::MIN_POSITIVE
                };
                assert!(
                    predicted >= least,
                    "{name}: the mirror predicts {predicted} px of wall on the advancing edge"
                );
            }
            assert!(
                (rendered - predicted).abs() <= 1.,
                "{name} side {sign}: {rendered} px rendered, {predicted} predicted"
            );
            if name == "peak" && sign < 0. {
                assert_eq!(rendered, 0., "the receding edge shows no wall at the peak");
            }
        }
    }
}

#[test]
fn a_swing_below_the_reveal_threshold_shows_no_wall() {
    // 11° at κ 2.5: θp ≤ 1 / (2κ), so no frame shows a wall (§3.2).
    for i in 1..20 {
        let shape = swing_shape(f64::from(i) / 20.);
        let pose = ViewPose {
            theta: 11f64.to_radians() * shape,
            dir: DVec2::X,
            weight: shape.abs(),
            kappa: 2.5,
        };
        let (pixels, _, _) = probe(LIVE, pose, -1.);
        let walls = pixels.chunks_exact(4).filter(|p| *p == MAGENTA).count();
        assert_eq!(walls, 0, "frame {i}");
    }
}

#[test]
fn a_diagonal_swing_shows_the_corner_wall_where_the_mirror_does() {
    let pose = ViewPose {
        theta: 14f64.to_radians(),
        dir: DVec2::new(0.6, 0.8),
        weight: 1.,
        kappa: 6.,
    };
    let (pixels, frame, glass) = probe(LIVE, pose, -1.);
    dump("walls/diagonal", &pixels, (OUT_W, OUT_H));
    let (solid, camera) = mirror(&frame, &glass, &pose);
    let corner = camera.to_screen((solid.center + solid.half).extend(0.));
    let (cx, cy) = (corner.x as i32, corner.y as i32);
    let mut rendered = 0;
    let mut predicted = 0;
    for y in cy - 30..cy + 30 {
        for x in cx - 30..cx + 30 {
            rendered += usize::from(px(&pixels, x, y) == MAGENTA);
            if let Some(seg) =
                Segment::new(&camera, DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5))
            {
                let tr = trace(&solid, &seg);
                predicted += usize::from(tr.wall && outline(&solid, &camera, &seg, &tr) < 0.);
            }
        }
    }
    assert!(predicted > 0, "the case must show a corner wall");
    assert!(
        rendered.abs_diff(predicted) * 10 <= predicted.max(10),
        "{rendered} rendered, {predicted} predicted"
    );
}

#[test]
fn the_face_coverage_matches_the_mirror_at_bevel_0() {
    // Plan review round 1: with bevel 0 the flat face reaches the silhouette, so face hits sit
    // on the outline. A 100 px slab at 20°, κ 1.5: at x = 52.5 the slab-local distance is
    // −0.4436 px against the projected −0.5305 px. The rendered coverage must follow the mirror
    // within the side bound (1/16 px of outline distance, times the smoothstep's slope 1.5).
    for glass in [thickness0(&bevel0(STOCK)), bevel0(STOCK)] {
        let pose = ViewPose {
            theta: 20f64.to_radians(),
            dir: DVec2::X,
            weight: 1.,
            kappa: 1.5,
        };
        let (pixels, frame, glass_r) = probe_sized(&glass, (100, 100), pose, -2.);
        // The frame holds the pane alone over the backdrop, so this covers its geometry.
        assert_eq!(sentinels(&pixels), 0, "coverage outside [0, 1]");
        let (solid, camera) = mirror(&frame, &glass_r, &pose);
        let y = solid.center.y as i32;
        for sign in [1., -1.] {
            let edge = camera
                .to_screen((solid.center + DVec2::new(sign * solid.half.x, 0.)).extend(0.))
                .x as i32;
            let mut band = 0;
            for x in edge - 4..edge + 4 {
                let s = DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
                let seg = Segment::new(&camera, s).expect("a downward ray");
                let want = coverage(outline(&solid, &camera, &seg, &trace(&solid, &seg)));
                band += usize::from(want > 0. && want < 1.);
                let got = f64::from(px(&pixels, x, y)[0]) / 255.;
                assert!(
                    (got - want).abs() <= COVERAGE_TOL,
                    "side {sign} x {x}: rendered coverage {got}, mirror {want}"
                );
            }
            assert!(band >= 1, "side {sign}: no pixel on the outline's ramp");
        }
    }
}

/// §8.2 "Zero rise in motion", read in the shader: under the coverage probe, a real swing at
/// bevel 0 and at thickness 0 (chamfer 12, so the rise is 0) marks no pixel of the pane's
/// geometry as a coverage outside [0, 1] (NaN included), and draws the mirror's coverage at
/// every one of them, within `the_face_coverage_matches_the_mirror_at_bevel_0`'s tolerance.
#[test]
fn a_swing_with_zero_rise_draws_the_mirror_coverage() {
    for (name, glass) in [("bevel0", bevel0(STOCK)), ("thickness0", thickness0(STOCK))] {
        let scene = Scene {
            content: CLEAR,
            backdrop: "#000000",
            ..Scene::new(&glass, "view-tilt 14\n                view-distance 2.5")
        };
        let (mut f, pane) = fixture_pane(&scene);
        let glass = pane_glass(&mut f, pane.window);
        let frame = slab_frame(&mut f, pane.window, &glass, None);
        assert_eq!(
            glass.thickness.min(f64::from(frame.chamfer)),
            0.,
            "{name}: the rise must be 0"
        );
        gain(&mut f);
        let _probe = SpillHook::set(-2.);
        let mut failures = Vec::new();
        let mut moving = 0;
        for (i, &t) in swing_frames().iter().enumerate() {
            let pixels = render_at(&mut f, t);
            dump(
                &format!("zero-rise-coverage/{name}-{i}"),
                &pixels,
                (OUT_W, OUT_H),
            );
            // The first and the settled frames are rest, which the probe does not reach.
            let Some(pose) = pane_tile(f.niri(), pane.window).0.view_pose() else {
                assert!(
                    t == GAIN || t == REST,
                    "{name} frame {i}: no pose mid-swing"
                );
                continue;
            };
            moving += 1;
            let (solid, camera) = mirror(&frame, &glass, &pose);
            let rect = pane_geometry(&mut f, pane.window, t, RenderTarget::ScreenCapture);
            let marked = sentinels(&region(&pixels, (OUT_W, OUT_H), rect));
            assert_eq!(marked, 0, "{name} frame {i}: coverage outside [0, 1]");
            let (mut band, mut bad) = (0, 0);
            for y in rect.loc.y..rect.loc.y + rect.size.h {
                for x in rect.loc.x..rect.loc.x + rect.size.w {
                    let s = DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
                    // A ray that does not point down hits no plane and takes no glass.
                    let want = Segment::new(&camera, s).map_or(0., |seg| {
                        coverage(outline(&solid, &camera, &seg, &trace(&solid, &seg)))
                    });
                    band += usize::from(want > 0. && want < 1.);
                    let got = f64::from(px(&pixels, x, y)[0]) / 255.;
                    if !((got - want).abs() <= COVERAGE_TOL) {
                        bad += 1;
                        if failures.len() < 20 {
                            failures.push(format!(
                                "{name} frame {i} ({x}, {y}): rendered {got}, mirror {want}"
                            ));
                        }
                    }
                }
            }
            assert!(
                band > 100,
                "{name} frame {i}: {band} outline pixels compared"
            );
            if bad > 0 {
                failures.push(format!("{name} frame {i}: {bad} px off the mirror"));
            }
        }
        assert_eq!(moving, SWING_MS.len(), "{name}: frames compared");
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}

#[test]
fn grazing_edges_change_coverage_smoothly() {
    // §8.2 "Grazing edges": across a slow sweep of θ, each outline pixel's coverage moves by no
    // more than 1.5 per px of the outline's motion plus twice the side bound (1/16 px), here
    // along rows through the middle of the straight sides.
    let mut prev: Option<(Vec<u8>, ViewPose)> = None;
    let mut changed = 0;
    for step in 2..=56 {
        let theta = 0.25 * f64::from(step);
        let pose = ViewPose {
            theta: theta.to_radians(),
            dir: DVec2::X,
            weight: theta / 14.,
            kappa: 6.,
        };
        let (pixels, frame, glass) = probe(LIVE, pose, -2.);
        // The frame holds the pane alone over the backdrop, so this covers its geometry.
        assert_eq!(sentinels(&pixels), 0, "θ {theta}: coverage outside [0, 1]");
        let (solid, camera) = mirror(&frame, &glass, &pose);
        let m_at = |pose: &ViewPose, x: i32, y: i32| {
            let (solid, camera) = mirror(&frame, &glass, pose);
            let seg = Segment::new(&camera, DVec2::new(f64::from(x) + 0.5, f64::from(y) + 0.5))
                .expect("a downward ray");
            outline(&solid, &camera, &seg, &trace(&solid, &seg))
        };
        let mut ramp = 0;
        let rows = (solid.center.y - 0.25 * solid.half.y) as i32
            ..(solid.center.y + 0.25 * solid.half.y) as i32;
        for y in rows.step_by(8) {
            for sign in [1., -1.] {
                let edge = camera
                    .to_screen((solid.center + DVec2::new(sign * solid.half.x, 0.)).extend(0.))
                    .x as i32;
                for x in edge - 20..edge + 20 {
                    let c = f64::from(px(&pixels, x, y)[0]) / 255.;
                    ramp += usize::from(c > 0. && c < 1.);
                    let m = m_at(&pose, x, y);
                    if m < -1. {
                        assert!(
                            c > 0.,
                            "zero coverage inside the outline at ({x}, {y}), θ {theta}"
                        );
                    }
                    if let Some((before, last)) = &prev {
                        let c0 = f64::from(px(before, x, y)[0]) / 255.;
                        changed += usize::from(c != c0);
                        let moved = (m - m_at(last, x, y)).abs();
                        assert!(
                            (c - c0).abs() <= 1.5 * moved + 0.125 + 2. / 255.,
                            "({x}, {y}) θ {theta}: coverage {c0} → {c}, outline moved {moved} px"
                        );
                    }
                }
            }
        }
        assert!(
            ramp > 0,
            "θ {theta}: no sampled pixel on the outline's ramp"
        );
        prev = Some((pixels, pose));
    }
    assert!(changed > 0, "no sampled coverage changed across the sweep");
}

/// An inactive material twice as thick, picked by an is-focused rule after the main one: the
/// focus loss mid-swing is a hard cut of the slab (§5.1).
const INACTIVE: &str = r##"
        material "tilt-inactive" {
            glass {
                ior 1.28
                thickness 58.1
                attenuation-color "#0D1D1E"
                attenuation-distance 11
                bevel 12
                bevel-profile 2
                backdrop-blur false
            }
        }
        window-rule {
            match is-focused=false
            material "tilt-inactive"
        }"##;

/// The second window's own material, after every other rule: LIVE with no swing and no beam,
/// so focusing it starts nothing of its own and the sequences show the pane's swing alone.
fn flat_second() -> String {
    format!(
        r##"
        material "tilt-flat" {{
            glass {{
                {LIVE}
                backdrop-blur false
            }}
            response "default" {{
                accent "none"
                {STILL}
                view-tilt 0
            }}
        }}
        window-rule {{
            match title="^second$"
            material "tilt-flat"
        }}"##
    )
}

/// `Fixture::double_roundtrip` on the frozen clock at `time`. The event loop clears the clock
/// at the end of every iteration, so the next would read the real monotonic time, and a swing
/// that started on the frozen clock would end there. Each iteration here starts back at `time`.
fn double_roundtrip_at(f: &mut Fixture, id: ClientId, time: Duration) {
    for _ in 0..2 {
        let data = f.client(id).send_sync();
        while !data.done.load(Ordering::Relaxed) {
            f.niri().clock.set_unadjusted(time);
            f.dispatch();
        }
    }
}

/// A render at `time`, and `window`'s drawn geometry in it.
fn shot(f: &mut Fixture, window: MappedId, time: Duration) -> (Vec<u8>, Rectangle<i32, Physical>) {
    let pixels = render_at(f, time);
    (
        pixels,
        pane_geometry(f, window, time, RenderTarget::ScreenCapture),
    )
}

#[test]
fn interaction_sequences_finish_at_rest() {
    let ms = |m: u64| GAIN + Duration::from_millis(m);
    // Black pixels in the pane's drawn geometry. LIVE's film grain turns some of its dark
    // pixels pure black at rest already, and its glass is black over parts of the second
    // window, so these sequences run without grain and look at the pane alone.
    let mut black_frames = Vec::new();
    let mut check = |name: &str, shots: &[(Vec<u8>, Rectangle<i32, Physical>)]| {
        for (i, (pixels, rect)) in shots.iter().enumerate() {
            dump(&format!("sequences/{name}-{i}"), pixels, (OUT_W, OUT_H));
            if black(&region(pixels, (OUT_W, OUT_H), *rect)) > 0 {
                black_frames.push(format!("{name} frame {i}"));
            }
        }
    };
    let quiet = grain0(LIVE);
    let swing_of =
        |f: &mut Fixture, window: MappedId| pane_tile(f.niri(), window).0.view_swing_start();

    // Focus loss mid-swing: the material swap is the accepted hard cut, then the swing
    // finishes on the unfocused pane (§5.1). The second window takes focus without a swing.
    let scene = Scene {
        after: format!("{INACTIVE}{}", flat_second()),
        ..Scene::new(&quiet, SWING)
    };
    let (mut f, pane) = fixture_pane(&scene);
    let second = pane.second.unwrap();
    gain(&mut f);
    let mut frames = vec![
        shot(&mut f, pane.window, ms(0)),
        shot(&mut f, pane.window, ms(100)),
    ];
    assert!(
        swing_of(&mut f, pane.window).is_some(),
        "the gain started no swing"
    );
    f.niri().layout.focus_right();
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    frames.push(shot(&mut f, pane.window, ms(171)));
    assert_eq!(
        swing_of(&mut f, second),
        None,
        "the second window swung on its focus"
    );
    frames.extend([ms(300), ms(900), REST].map(|t| shot(&mut f, pane.window, t)));
    assert_eq!(
        pane_tile(f.niri(), pane.window).0.view_pose(),
        None,
        "a pose after settle"
    );
    assert!(
        diff(&frames[2].0, &frames[5].0).0 > 0,
        "the swing went on after the loss"
    );
    assert_eq!(
        diff(&frames[4].0, &frames[5].0),
        (0, 0),
        "the swing did not finish at settle"
    );
    check("focus-loss", &frames);

    // A refocus mid-swing keeps the swing it has: the frames match an uninterrupted run.
    let plain = Scene {
        after: flat_second(),
        ..Scene::new(LIVE, SWING)
    };
    let straight = renders(&plain, None, &[ms(0), ms(450), ms(600)]);
    let (mut f, pane) = fixture_pane(&plain);
    let second = pane.second.unwrap();
    gain(&mut f);
    render_at(&mut f, ms(0));
    let started = swing_of(&mut f, pane.window).expect("the gain started a swing");
    render_at(&mut f, ms(100));
    f.niri().layout.focus_right();
    f.niri_state().update_keyboard_focus();
    render_at(&mut f, ms(150));
    assert_eq!(
        swing_of(&mut f, second),
        None,
        "the second window swung on its focus"
    );
    gain(&mut f);
    let again = [render_at(&mut f, ms(450)), render_at(&mut f, ms(600))];
    assert_eq!(
        swing_of(&mut f, pane.window),
        Some(started),
        "the refocus restarted the swing"
    );
    assert_eq!(
        diff(&again[0], &straight[1]),
        (0, 0),
        "a restart moved the 450 ms frame"
    );
    assert_eq!(
        diff(&again[1], &straight[2]),
        (0, 0),
        "a restart moved the 600 ms frame"
    );

    // A resize mid-swing runs through (§5.1).
    let (mut f, pane) = fixture_pane(&Scene {
        after: flat_second(),
        ..Scene::new(&quiet, SWING)
    });
    gain(&mut f);
    let mut frames = vec![
        shot(&mut f, pane.window, ms(0)),
        shot(&mut f, pane.window, ms(100)),
    ];
    let started = swing_of(&mut f, pane.window).expect("the gain started a swing");
    let window = f.client(pane.client).window(&pane.surface);
    window.set_size(560, 420);
    window.ack_last_and_commit();
    double_roundtrip_at(&mut f, pane.client, ms(100));
    assert_eq!(
        swing_of(&mut f, pane.window),
        Some(started),
        "the resize changed the swing"
    );
    frames.push(shot(&mut f, pane.window, ms(171)));
    assert_eq!(
        swing_of(&mut f, pane.window),
        Some(started),
        "the resize changed the swing"
    );
    frames.extend(
        [ms(600), REST, REST + Duration::from_secs(1)].map(|t| shot(&mut f, pane.window, t)),
    );
    assert_eq!(
        pane_tile(f.niri(), pane.window).0.view_pose(),
        None,
        "a pose after settle"
    );
    assert_eq!(
        pane_rect(&mut f, pane.window).size,
        Size::from((560., 420.)),
        "the pane did not resize"
    );
    assert!(
        diff(&frames[2].0, &frames[4].0).0 > 0,
        "the swing did not run through the resize"
    );
    assert_eq!(
        diff(&frames[4].0, &frames[5].0),
        (0, 0),
        "not at rest after a resize mid-swing"
    );
    check("resize", &frames);

    // The policy cut: motion "reduced" mid-swing ends it at rest on the next frame.
    let mut f = fixture(&plain);
    gain(&mut f);
    render_at(&mut f, ms(0));
    let peak = render_at(&mut f, ms(100));
    let reduced = Scene {
        top: r#"signal { motion "reduced"; }"#.to_owned(),
        ..plain.clone()
    };
    f.niri_state().reload_config(Ok(config(&reduced)));
    let cut = render_at(&mut f, ms(171));
    let rest = renders(&reduced, None, &[ms(171)]);
    assert!(diff(&peak, &cut).0 > 0);
    assert_eq!(diff(&cut, &rest[0]), (0, 0), "the cut is not rest");

    // Animations off: no swing at all.
    let off = Scene {
        top: "animations { off; }".to_owned(),
        ..plain.clone()
    };
    let still = Scene {
        top: "animations { off; }".to_owned(),
        ..Scene::new(LIVE, STILL)
    };
    let a = renders(&off, None, &[ms(171)]);
    let b = renders(&still, None, &[ms(171)]);
    assert_eq!(
        diff(&a[0], &b[0]),
        (0, 0),
        "a swing ran with animations off"
    );

    // Ineligible panes take no swing (§5): border on, fullscreen, a visible background effect,
    // which a second rule adds (the main rule already has a `background-effect` node). The
    // eligible pane first, so the others cannot pass for want of a swing at all. Each pane is
    // drawn once before the gain, as any pane is before a refocus: whether its background
    // effect is visible is what its last render resolved.
    for (name, rules, after, swings) in [
        ("eligible", "", "", true),
        ("border", "border { on; }", "", false),
        ("fullscreen", "open-fullscreen true", "", false),
        (
            "background-effect",
            "",
            "window-rule {\n            background-effect { blur true; }\n        }",
            false,
        ),
    ] {
        let scene = Scene {
            rules: rules.to_owned(),
            after: after.to_owned(),
            ..plain.clone()
        };
        let (mut f, pane) = fixture_pane(&scene);
        render_at(&mut f, Duration::ZERO);
        gain(&mut f);
        render_at(&mut f, ms(0));
        render_at(&mut f, ms(50));
        assert_eq!(swing_of(&mut f, pane.window).is_some(), swings, "{name}");
    }

    assert!(black_frames.is_empty(), "black pixels in {black_frames:?}");
}
