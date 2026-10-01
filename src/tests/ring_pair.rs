//! Matched ring-on/off renders at one frozen mid-resize instant (material-0e80c1).
//!
//! The headless backend's surfaceless GLES renderer draws the real material
//! shader into a texture, the way a screenshot does, with the clock frozen by
//! `set_time` immediately before each render (dispatch and reload both unfreeze
//! it). Response-only and glass-parameter reloads keep the material state, its
//! seed and the resize animation, so every variant is rendered from one
//! fixture at one instant. The client attaches shm buffers: a single-pixel
//! buffer never becomes a texture, its resize snapshot is empty, and the tile
//! then falls back to a plain render without the material.
//!
//! The test records the light map (on minus off) per edge; it does not grade
//! the ring. Set `RING_PAIR_DUMP=<dir>` to write each render as a PNG.

use std::fmt::Write as _;
use std::time::Duration;

use niri_config::Config;
use niri_ipc::SizeChange;
use smithay::backend::allocator::Fourcc;
use smithay::utils::{Logical, Rectangle, Scale, Transform};

use super::*;
use crate::render_helpers::{render_to_vec, RenderCtx, RenderTarget};

const OUT_W: u16 = 1280;
const OUT_H: u16 = 720;
const W: u16 = 640;
const H: u16 = 360;
const MID: Duration = Duration::from_millis(500);
const REST: Duration = Duration::from_millis(2000);

struct Glass {
    name: &'static str,
    ior: f64,
    thickness: f64,
    bevel: f64,
    gap: f64,
}

#[derive(Clone, Copy)]
struct Variant {
    ring: bool,
    light_ior: f64,
    jelly_flex: f64,
}

const BASE: Variant = Variant {
    ring: true,
    light_ior: 6.,
    jelly_flex: 0.,
};

fn config(glass: &Glass, v: Variant) -> Config {
    let Glass {
        ior,
        thickness,
        bevel,
        gap,
        ..
    } = glass;
    let Variant {
        ring,
        light_ior,
        jelly_flex,
    } = v;
    let focus = if ring { "ring-light" } else { "none" };
    Config::parse_mem(&format!(
        r##"
        hotkey-overlay {{ skip-at-startup; }}
        layout {{
            gaps {gap}
            focus-ring {{ off; }}
            border {{ off; }}
            shadow {{ off; }}
        }}
        animations {{
            window-resize {{ duration-ms 1000; curve "linear"; }}
        }}
        material "pair" {{
            glass {{
                ior {ior}
                thickness {thickness}
                bevel {bevel}
                light-ior {light_ior}
                chromatic-aberration 0
                distortion 0 scale=0.5
                jelly-flex {jelly_flex}
                jelly-ripple 0
                backdrop-blur false
                aurora 0 {{ drift-hz 0; }}
            }}
            response "default" {{
                focus "{focus}"
                accent "none"
                ring-beam-speed 0
                ring-gap {gap}
            }}
        }}
        window-rule {{
            material "pair"
            background-effect {{ blur false; noise 0; saturation 1; }}
        }}
        "##
    ))
    .unwrap()
}

fn set_time(f: &mut Fixture, time: Duration) {
    let niri = f.niri();
    let now = niri.clock.now();
    niri.clock.set_unadjusted(now);
    let _ = niri.clock.now();
    niri.clock.set_unadjusted(Duration::ZERO);
    niri.clock.set_rate(1.0);
    let _ = niri.clock.now();
    niri.clock.set_unadjusted(time);
    let _ = niri.clock.now();
    niri.clock.set_rate(0.0);
}

fn reload(f: &mut Fixture, glass: &Glass, v: Variant) {
    f.niri_state().reload_config(Ok(config(glass, v)));
    f.niri_state().refresh_and_flush_clients();
}

/// The output as a screenshot renders it, at `time`, as RGBA bytes.
fn render_at(f: &mut Fixture, time: Duration) -> Vec<u8> {
    set_time(f, time);
    f.niri().advance_animations();
    let output = f.niri_output(1);
    let crate::niri::State { backend, niri } = f.niri_state();
    niri.update_render_elements(Some(&output));
    let size = output.current_mode().unwrap().size;
    let scale = Scale::from(output.current_scale().fractional_scale());
    backend
        .with_primary_renderer(|renderer| {
            let ctx = RenderCtx {
                renderer,
                target: RenderTarget::ScreenCapture,
                xray: None,
                signal_ticks: None,
            };
            let elements = niri.render_to_vec(ctx, &output, false);
            render_to_vec(
                renderer,
                size,
                scale,
                Transform::Normal,
                Fourcc::Abgr8888,
                elements.iter().rev(),
            )
            .unwrap()
        })
        .unwrap()
}

/// The window's animated rectangle on the output, in logical px (scale 1).
fn window_rect(f: &mut Fixture) -> Rectangle<f64, Logical> {
    let niri = f.niri();
    let (_, _, workspace) = niri.layout.workspaces().next().unwrap();
    let (tile, pos, _) = workspace.tiles_with_render_positions().next().unwrap();
    Rectangle::new(pos + tile.window_loc(), tile.animated_window_size())
}

fn dump(name: &str, pixels: &[u8]) {
    let Some(dir) = std::env::var_os("RING_PAIR_DUMP") else {
        return;
    };
    let path = std::path::Path::new(&dir).join(format!("{name}.png"));
    let file = std::fs::File::create(path).unwrap();
    crate::utils::write_png_rgba8(file, OUT_W.into(), OUT_H.into(), pixels).unwrap();
}

/// Differing pixels and the largest channel delta.
fn diff(a: &[u8], b: &[u8]) -> (usize, u8) {
    assert_eq!(a.len(), b.len());
    let map = light_map(a, b);
    (
        map.iter().filter(|&&v| v > 0).count(),
        map.into_iter().max().unwrap(),
    )
}

/// Per pixel, the largest RGB delta between two renders.
fn light_map(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.chunks_exact(4)
        .zip(b.chunks_exact(4))
        .map(|(pa, pb)| (0..3).map(|c| pa[c].abs_diff(pb[c])).max().unwrap())
        .collect()
}

/// The light map's profile across each window edge, on the line through the
/// window centre: the peak's distance inward from the edge (pixel centres,
/// negative outside the window), its value, the half-maximum run around it
/// as inward distances, and that run's value-weighted centroid.
fn profile(map: &[u8], rect: Rectangle<f64, Logical>) -> String {
    let cx = (rect.loc.x + rect.size.w / 2.).floor() as i32;
    let cy = (rect.loc.y + rect.size.h / 2.).floor() as i32;
    let (l, t) = (rect.loc.x, rect.loc.y);
    let (r, b) = (l + rect.size.w, t + rect.size.h);
    // (name, horizontal, edge, +1 when inward is increasing pixel index).
    let sides = [
        ("left", true, l, 1.),
        ("right", true, r, -1.),
        ("top", false, t, 1.),
        ("bottom", false, b, -1.),
    ];
    let mut out = String::new();
    for (name, horizontal, edge, inward) in sides {
        let limit = i32::from(if horizontal { OUT_W } else { OUT_H });
        let first = edge.floor() as i32;
        let samples: Vec<(u8, f64)> = (first - 40..=first + 40)
            .filter(|&i| (0..limit).contains(&i))
            .map(|i| {
                let (x, y) = if horizontal { (i, cy) } else { (cx, i) };
                let value = map[y as usize * usize::from(OUT_W) + x as usize];
                (value, inward * (f64::from(i) + 0.5 - edge))
            })
            .collect();
        let (peak_idx, &(peak, peak_in)) = samples
            .iter()
            .enumerate()
            .max_by(|a, b| a.1 .0.cmp(&b.1 .0).then(b.0.cmp(&a.0)))
            .unwrap();
        let half = peak.div_ceil(2);
        let mut lo = peak_idx;
        while lo > 0 && samples[lo - 1].0 >= half {
            lo -= 1;
        }
        let mut hi = peak_idx;
        while hi + 1 < samples.len() && samples[hi + 1].0 >= half {
            hi += 1;
        }
        let (a, b) = (samples[lo].1, samples[hi].1);
        let run = &samples[lo..=hi];
        let weight: f64 = run.iter().map(|&(v, _)| f64::from(v)).sum();
        let centroid = run.iter().map(|&(v, d)| f64::from(v) * d).sum::<f64>() / weight;
        let _ = write!(
            out,
            "{name} peak {peak_in:+.1} ({peak}) half {:+.1}..{:+.1} c {centroid:+.2}; ",
            a.min(b),
            a.max(b),
        );
    }
    out.trim_end_matches("; ").to_owned()
}

/// One matched pair: on and off at `time` under `v`, from the current state.
/// Returns the report and the ring-on render.
fn pair(
    f: &mut Fixture,
    glass: &Glass,
    v: Variant,
    time: Duration,
    tag: &str,
) -> (String, Vec<u8>) {
    reload(f, glass, v);
    let on = render_at(f, time);
    reload(f, glass, Variant { ring: false, ..v });
    let off = render_at(f, time);
    let rect = window_rect(f);
    dump(&format!("{}-{tag}-on", glass.name), &on);
    dump(&format!("{}-{tag}-off", glass.name), &off);
    let map = light_map(&on, &off);
    let lit = map.iter().filter(|&&v| v > 0).count();
    let max = map.iter().copied().max().unwrap();
    let report = format!(
        "{tag} (window {:.1}x{:.1} at {:.1},{:.1}): {lit} px lit, max {max}\n    {}\n",
        rect.size.w,
        rect.size.h,
        rect.loc.x,
        rect.loc.y,
        profile(&map, rect),
    );
    (report, on)
}

fn matched_pairs(glass: &Glass) -> String {
    let mut f = Fixture::with_config(config(glass, BASE));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (OUT_W, OUT_H));
    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(0);
    window.set_size(W, H);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    // The ring lights the focused window; `is_focused` is cached and only
    // `update_keyboard_focus` writes it.
    f.niri_state().update_keyboard_focus();
    f.double_roundtrip(id);
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    let _ = render_at(&mut f, Duration::ZERO);

    // A 200 px wider column, half way through its linear resize.
    f.niri()
        .layout
        .set_column_width(SizeChange::AdjustFixed(200));
    f.double_roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(0);
    window.set_size(W + 200, H);
    window.ack_last_and_commit();
    f.roundtrip(id);

    let mut report = String::new();
    let seed = {
        let (_, _, workspace) = f.niri().layout.workspaces().next().unwrap();
        let (tile, _, _) = workspace.tiles_with_render_positions().next().unwrap();
        tile.material().unwrap().jelly_seed()
    };
    let renderer = f
        .niri_state()
        .backend
        .with_primary_renderer(|r| {
            r.with_context(|gl| unsafe {
                let name = gl.GetString(smithay::backend::renderer::gles::ffi::RENDERER);
                std::ffi::CStr::from_ptr(name.cast())
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap()
        })
        .unwrap();
    let _ = writeln!(report, "renderer {renderer}, jelly seed {seed:?}");
    let on1 = render_at(&mut f, MID);
    let (mid, base_on) = pair(&mut f, glass, BASE, MID, "mid");
    report += &mid;
    reload(&mut f, glass, BASE);
    let on2 = render_at(&mut f, MID);
    // Negative control: 10 ms later the slab has moved 2 px.
    let shifted = render_at(&mut f, MID + Duration::from_millis(10));
    let (rp, rm) = diff(&on1, &on2);
    let (sp, sm) = diff(&on1, &shifted);
    let _ = writeln!(report, "repeat after off/on reload: {rp} px, max {rm}");
    let _ = writeln!(report, "shifted +10 ms: {sp} px, max {sm}");

    for (tag, v) in [
        (
            "mid flex 0.0066",
            Variant {
                jelly_flex: 0.0066,
                ..BASE
            },
        ),
        (
            "mid flex 0.02",
            Variant {
                jelly_flex: 0.02,
                ..BASE
            },
        ),
        (
            "mid light-ior 1",
            Variant {
                light_ior: 1.,
                ..BASE
            },
        ),
    ] {
        let (variant, on) = pair(&mut f, glass, v, MID, tag);
        report += &variant;
        // What the variant alone changes, ring on in both.
        let (vp, vm) = diff(&base_on, &on);
        let _ = writeln!(report, "    vs base, ring on: {vp} px, max {vm}");
    }
    // Every reload above kept the state: the base render comes back exactly.
    reload(&mut f, glass, BASE);
    let on3 = render_at(&mut f, MID);
    let (fp, fm) = diff(&on1, &on3);
    let _ = writeln!(report, "repeat after all variants: {fp} px, max {fm}");

    // At rest the resize has finished; this ends the animation, so it comes
    // after every mid-resize render.
    report += &pair(&mut f, glass, BASE, REST, "rest").0;

    // Opaque client pixels bypass the slab: no ring light inside the window.
    let window = f.client(id).window(&surface);
    window.attach_new_shm_buffer(0xff40_4040);
    window.commit();
    f.double_roundtrip(id);
    reload(&mut f, glass, BASE);
    let on = render_at(&mut f, REST);
    reload(
        &mut f,
        glass,
        Variant {
            ring: false,
            ..BASE
        },
    );
    let off = render_at(&mut f, REST);
    dump(&format!("{}-opaque-on", glass.name), &on);
    dump(&format!("{}-opaque-off", glass.name), &off);
    let rect = window_rect(&mut f);
    let (mut inside, mut outside) = (0, 0);
    for (i, &v) in light_map(&on, &off).iter().enumerate() {
        if v == 0 {
            continue;
        }
        let x = (i % usize::from(OUT_W)) as f64;
        let y = (i / usize::from(OUT_W)) as f64;
        let within = x >= rect.loc.x
            && x + 1. <= rect.loc.x + rect.size.w
            && y >= rect.loc.y
            && y + 1. <= rect.loc.y + rect.size.h;
        if within {
            inside += 1;
        } else {
            outside += 1;
        }
    }
    let _ = writeln!(
        report,
        "opaque client at rest: {inside} px lit inside the window, {outside} outside"
    );
    report
}

#[test]
fn ring_pair_is_reproducible_at_a_frozen_instant() {
    // Stock geometry: the shared shift stays under the cap (2.28 px vs 4).
    let stock = Glass {
        name: "stock",
        ior: 1.5,
        thickness: 20.,
        bevel: 12.,
        gap: 8.,
    };
    // Live-Prism-like geometry: the shift binds the cap (3.09 px vs 1).
    let binding = Glass {
        name: "binding",
        ior: 1.28,
        thickness: 31.2,
        bevel: 10.,
        gap: 2.,
    };
    for glass in [&stock, &binding] {
        let report = matched_pairs(glass);
        eprintln!("{}:\n{report}", glass.name);
        for check in [
            "repeat after off/on reload: 0 px",
            "repeat after all variants: 0 px",
        ] {
            assert!(report.contains(check), "{report}");
        }
        assert!(!report.contains("shifted +10 ms: 0 px"), "{report}");
        assert!(
            report.contains("opaque client at rest: 0 px lit inside"),
            "{report}"
        );
        assert!(!report.contains("): 0 px lit"), "{report}");
    }
}
