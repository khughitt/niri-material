//! Frozen-clock renders of the rigid tilt (docs/specs/2026-10-09-transient-rigid-tilt-design.md
//! §8.2), through the real material shader in the headless backend, as ring_pair.rs renders.
//! Set `VIEW_TILT_DUMP=<dir>` to write each render as `<name>.png`. Every fixture gets the next
//! jelly seed; nextest runs each test in its own process, so dumps compare across commits.
//! Run through `just test-one`, never `cargo test`.

use std::time::Duration;

use glam::DVec2;
use niri_config::Config;

use super::glass_edge::{self, renderer_name, BACKDROP};
pub(super) use super::glass_edge::{CLEAR, OPAQUE, OUT_H, OUT_W, TRANSLUCENT};
pub(super) use super::ring_look::gain;
use super::ring_pair::{diff, render_at, set_time};
use super::*;
use crate::render_helpers::material::view::{hooks, ViewPose};

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
}

impl Scene {
    pub fn new(glass: &str, response: &str) -> Self {
        Self {
            glass: glass.to_owned(),
            response: response.to_owned(),
            rules: String::new(),
            content: TRANSLUCENT,
            size: (640, 480),
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
            background-color "{BACKDROP}"
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

/// The fixture for `scene`: the pane, then a second window that holds focus until the gain.
pub(super) fn fixture(scene: &Scene) -> Fixture {
    let mut f = glass_edge::fixture(config(scene));
    let id = f.add_client();
    glass_edge::open(&mut f, id, scene.size, scene.content);
    glass_edge::open(&mut f, id, (400, 300), scene.content);
    f.niri_state().update_keyboard_focus();
    f.niri().refresh_window_rules();
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    f
}

/// Renders `scene` at `frames` after a gain at `GAIN`, under the pose hook `pose`.
pub(super) fn renders(scene: &Scene, pose: Option<ViewPose>, frames: &[Duration]) -> Vec<Vec<u8>> {
    let _hook = PoseHook::set(pose);
    let mut f = fixture(scene);
    gain(&mut f);
    frames.iter().map(|t| render_at(&mut f, *t)).collect()
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
