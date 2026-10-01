//! Matched ring-on/off renders at one frozen mid-resize instant (material-0e80c1).
//! Work in progress: the repeat and the shifted control work, but the ring does
//! not light in this render yet (on/off identical), so the pair is not measured.
//!
//! The headless backend's surfaceless GLES renderer draws the real material
//! shader into a texture, the way a screenshot does, with the clock frozen by
//! `set_time` immediately before each render (dispatch and reload both unfreeze
//! it). A response-only reload switches the ring off and back on while keeping
//! the material state, its seed and the resize animation. The test records the
//! pair's difference; it does not grade the ring.

use std::time::Duration;

use niri_config::Config;
use niri_ipc::SizeChange;
use smithay::backend::allocator::Fourcc;
use smithay::utils::{Scale, Transform};

use super::*;
use crate::render_helpers::{render_to_vec, RenderCtx, RenderTarget};

const W: u16 = 640;
const H: u16 = 360;
const MID: Duration = Duration::from_millis(500);

fn config(response: &str, glass: &str, gap: f64) -> Config {
    Config::parse_mem(&format!(
        r##"
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
                {glass}
                chromatic-aberration 0
                distortion 0 scale=0.5
                jelly-flex 0
                jelly-ripple 0
                backdrop-blur false
                aurora 0 {{ drift-hz 0; }}
            }}
            response "default" {{
                {response}
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

const RING_ON: &str = r#"focus "ring-light"; accent "none";"#;
const RING_OFF: &str = r#"focus "none"; accent "none";"#;

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

/// Differing pixels, the largest channel delta, and the summed channel delta.
fn diff(a: &[u8], b: &[u8]) -> (usize, u8, u64) {
    assert_eq!(a.len(), b.len());
    let mut pixels = 0;
    let mut max = 0;
    let mut sum = 0u64;
    for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let d = pa
            .iter()
            .zip(pb)
            .map(|(x, y)| x.abs_diff(*y))
            .collect::<Vec<_>>();
        if d.iter().any(|&v| v > 0) {
            pixels += 1;
        }
        max = max.max(*d.iter().max().unwrap());
        sum += d.iter().map(|&v| u64::from(v)).sum::<u64>();
    }
    (pixels, max, sum)
}

fn matched_pair(glass: &str, gap: f64) -> String {
    let mut f = Fixture::with_config(config(RING_ON, glass, gap));
    f.niri_state().backend.headless().add_renderer().unwrap();
    f.add_output(1, (1280, 720));
    let id = f.add_client();
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.set_size(W, H);
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    // The ring lights the focused window; `is_focused` is cached and only
    // `update_keyboard_focus` writes it.
    f.niri_state().update_keyboard_focus();
    f.double_roundtrip(id);
    set_time(&mut f, Duration::ZERO);
    f.niri_complete_animations();
    let has_material = f
        .niri()
        .layout
        .workspaces()
        .flat_map(|(_, _, workspace)| workspace.tiles())
        .any(|tile| tile.material().is_some());
    let has_program = f
        .niri_state()
        .backend
        .with_primary_renderer(|r| crate::render_helpers::material::MaterialState::has_program(r))
        .unwrap();
    eprintln!("tile material: {has_material}, material program: {has_program}");
    let open = render_at(&mut f, Duration::ZERO);
    let lit = open.chunks_exact(4).filter(|p| p[3] > 0).count();

    // A 200 px wider column, half way through its linear resize.
    f.niri()
        .layout
        .set_column_width(SizeChange::AdjustFixed(200));
    f.double_roundtrip(id);
    let window = f.client(id).window(&surface);
    window.set_size(W + 200, H);
    window.ack_last_and_commit();
    f.roundtrip(id);

    let on1 = render_at(&mut f, MID);
    f.niri_state()
        .reload_config(Ok(config(RING_OFF, glass, gap)));
    f.niri_state().refresh_and_flush_clients();
    let off = render_at(&mut f, MID);
    f.niri_state()
        .reload_config(Ok(config(RING_ON, glass, gap)));
    f.niri_state().refresh_and_flush_clients();
    let on2 = render_at(&mut f, MID);
    // Negative control: 10 ms later the slab has moved about 1 px.
    let on3 = render_at(&mut f, MID + Duration::from_millis(10));

    let (rp, rm, _) = diff(&on1, &on2);
    let (sp, sm, _) = diff(&on1, &on3);
    let (lp, lm, ls) = diff(&on1, &off);
    format!(
        "drawn at rest: {lit} px with alpha\n\
         repeat on1/on2: {rp} px, max {rm}\n\
         shifted on1/on3: {sp} px, max {sm}\n\
         ring on1/off: {lp} px, max {lm}, sum {ls}\n"
    )
}

#[test]
fn ring_pair_is_reproducible_at_a_frozen_instant() {
    // Stock geometry: the shared shift stays under the cap (2.28 px vs 4).
    let stock = matched_pair("ior 1.5\nthickness 20\nbevel 12\nlight-ior 6", 8.);
    // Live-Prism-like geometry: the shift binds the cap (3.09 px vs 1).
    let binding = matched_pair("ior 1.28\nthickness 31.2\nbevel 10\nlight-ior 6", 2.);
    for report in [&stock, &binding] {
        let repeat = report.lines().nth(1).unwrap();
        assert!(repeat.starts_with("repeat on1/on2: 0 px"), "{report}");
    }
    eprintln!("stock:\n{stock}binding:\n{binding}");
}
