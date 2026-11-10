//! The Layout starts the rigid tilt's swing from the focus origin
//! (docs/materials/motion.md#swing-origin).

use super::*;

fn swing_options() -> Options {
    let material = niri_config::ResolvedMaterial {
        name: String::from("frost"),
        glass: niri_config::ResolvedGlass::default(),
        responses: vec![(
            String::from("default"),
            niri_config::ResolvedResponse {
                view_tilt: 10.,
                view_tilt_settle: 900.,
                view_distance: 6.,
                ring_beam_speed: 0.,
                ..Default::default()
            },
        )],
    };
    let mut layout = niri_config::Layout::default();
    layout.focus_ring.off = true;
    Options {
        layout,
        materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
        ..Default::default()
    }
}

fn glass_window(id: usize) -> Op {
    let mut params = TestWindowParams::new(id);
    params.rules = Some(ResolvedWindowRules {
        material: Some(niri_config::MaterialRef {
            name: String::from("frost"),
            response: None,
        }),
        ..Default::default()
    });
    Op::AddWindow { params }
}

fn swing_layout(ops: impl IntoIterator<Item = Op>) -> Layout<TestWindow> {
    check_ops_with_options(swing_options(), ops)
}

/// Every tile's swing, by window id: (start, direction).
fn swings(layout: &Layout<TestWindow>) -> Vec<(usize, Duration, [f64; 2])> {
    layout
        .monitors()
        .flat_map(|mon| mon.workspaces.iter())
        .flat_map(|ws| ws.tiles())
        .filter_map(|tile| {
            tile.view_swing_start()
                .map(|(t, d)| (*tile.window().id(), t, d))
        })
        .collect()
}

/// The windows whose swing started at the clock's current instant.
fn started_now(layout: &Layout<TestWindow>) -> Vec<usize> {
    let now = layout.clock.now_unadjusted();
    swings(layout)
        .into_iter()
        .filter(|(_, t, _)| *t == now)
        .map(|(id, _, _)| id)
        .collect()
}

fn tile_of(layout: &Layout<TestWindow>, id: usize) -> &Tile<TestWindow> {
    layout
        .monitors()
        .flat_map(|mon| mon.workspaces.iter())
        .flat_map(|ws| ws.tiles())
        .find(|tile| *tile.window().id() == id)
        .unwrap_or_else(|| panic!("no tile for window {id}"))
}

/// Asserts that window `id` started a fallback swing at the current instant.
fn assert_fallback(layout: &Layout<TestWindow>, id: usize) {
    let now = layout.clock.now_unadjusted();
    assert_eq!(
        direction_of(layout, id),
        tile_of(layout, id).fallback_direction_at(now),
        "window {id} should swing in its seeded direction"
    );
}

/// The centre of window `id`'s hit area on the output, found by walking out
/// from `inside` until `window_under` stops naming it. This is independent of
/// how `tile_centre_on_output` computes its centre: hit-testing is the other
/// consumer of the render geometry.
fn hit_centre(
    mon: &Monitor<TestWindow>,
    id: usize,
    inside: Point<f64, Logical>,
) -> Point<f64, Logical> {
    let hits = |p: Point<f64, Logical>| mon.window_under(p).is_some_and(|(w, _)| *w.id() == id);
    assert!(hits(inside), "the computed centre is not over window {id}");
    let edge = |dx: f64, dy: f64| {
        let step = Point::from((dx, dy));
        let mut p = inside;
        while hits(p + step) {
            p += step;
        }
        p
    };
    let (l, r) = (edge(-0.25, 0.).x, edge(0.25, 0.).x);
    let (t, b) = (edge(0., -0.25).y, edge(0., 0.25).y);
    Point::from(((l + r) / 2., (t + b) / 2.))
}

fn assert_near(a: Point<f64, Logical>, b: Point<f64, Logical>, what: &str) {
    assert!(
        (a.x - b.x).abs() <= 1. && (a.y - b.y).abs() <= 1.,
        "{what}: {a:?} vs {b:?}"
    );
}

fn direction_of(layout: &Layout<TestWindow>, id: usize) -> [f64; 2] {
    swings(layout)
        .into_iter()
        .find(|(w, _, _)| *w == id)
        .map(|(_, _, d)| d)
        .unwrap_or_else(|| panic!("window {id} has no swing"))
}

fn advance(layout: &mut Layout<TestWindow>, msec: i32) {
    Op::AdvanceAnimations { msec_delta: msec }.apply(layout);
}

fn output_named(layout: &Layout<TestWindow>, name: &str) -> Output {
    layout.outputs().find(|o| o.name() == name).unwrap().clone()
}

#[test]
fn a_focus_change_swings_from_the_old_centre() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![2], "the first focus falls back");
    assert_fallback(&layout, 2);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![1]);
    let d = direction_of(&layout, 1);
    assert!(
        d[0] < -0.99,
        "from window 2 on the right to window 1: {d:?}"
    );
}

#[test]
fn a_b_c_without_a_pass_swings_c_from_a() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        glass_window(2),
        glass_window(3),
        Op::FocusColumn(1),
    ]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusColumn(2).apply(&mut layout);
    Op::FocusColumn(3).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![3], "B was never observed");
    assert!(direction_of(&layout, 3)[0] > 0.99, "from A on the left");
}

#[test]
fn a_b_a_without_a_pass_starts_no_swing() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    Op::FocusColumnRight.apply(&mut layout);
    layout.update_render_elements(None);
    assert!(started_now(&layout).is_empty());
}

#[test]
fn a_pass_over_an_inactive_output_leaves_the_origin() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        Op::AddOutput(2),
        Op::FocusOutput(1),
        glass_window(1),
        glass_window(2),
    ]);
    layout.update_render_elements(None);
    let before = layout.focus_origin.clone();
    assert!(before.is_some(), "the first pass records an origin");
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    let inactive = output_named(&layout, "output2");
    layout.update_render_elements(Some(&inactive));
    assert_eq!(layout.focus_origin, before);
    assert!(started_now(&layout).is_empty());
}

#[test]
fn a_pass_for_a_capture_of_the_focused_output_counts() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    // Screencopy and screenshots call this with their output.
    let focused = output_named(&layout, "output1");
    layout.update_render_elements(Some(&focused));
    assert_eq!(started_now(&layout), vec![1]);
    advance(&mut layout, 16);
    layout.update_render_elements(None);
    assert!(started_now(&layout).is_empty(), "observed once, not twice");
}

#[test]
fn a_focus_on_another_output_takes_the_fallback() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        Op::AddOutput(2),
        Op::FocusOutput(1),
        glass_window(1),
        Op::FocusOutput(2),
        glass_window(2),
        Op::FocusOutput(1),
    ]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusOutput(2).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![2]);
    assert_fallback(&layout, 2);
    assert_eq!(
        layout.focus_origin.as_ref().map(|o| o.output.name()),
        Some(String::from("output2"))
    );
}

#[test]
fn closing_the_origin_window_is_harmless() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::CloseWindow(2).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![1]);
    let d = direction_of(&layout, 1);
    assert!(
        d[0] < -0.99,
        "from the closed window 2's last centre, on the right: {d:?}"
    );
}

#[test]
fn switching_tabs_in_one_column_takes_the_fallback() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        glass_window(2),
        Op::ConsumeOrExpelWindowLeft { id: None },
        Op::ToggleColumnTabbedDisplay,
    ]);
    advance(&mut layout, 1000);
    layout.update_render_elements(None);
    let origin = layout.focus_origin.clone().unwrap();
    assert_eq!(origin.window, 2);
    advance(&mut layout, 100);
    Op::FocusWindowUp.apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![1]);
    let mon = layout.monitors().next().unwrap();
    assert_near(
        mon.tile_centre_on_output(&1).unwrap(),
        origin.centre,
        "tabs share one centre",
    );
    assert_fallback(&layout, 1);
}

#[test]
fn an_empty_workspace_keeps_the_origin_for_the_next_window() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    advance(&mut layout, 1000);
    layout.update_render_elements(None);
    let seen = layout.focus_origin.clone().unwrap();
    assert_eq!(seen.window, 2);
    Op::FocusWorkspaceDown.apply(&mut layout);
    advance(&mut layout, 1000);
    layout.update_render_elements(None);
    assert_eq!(
        layout.focus_origin.as_ref(),
        Some(&seen),
        "no focused window leaves the origin"
    );
    assert!(started_now(&layout).is_empty());
    advance(&mut layout, 100);
    glass_window(3).apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![3]);
    let centre = layout
        .monitors()
        .next()
        .unwrap()
        .tile_centre_on_output(&3)
        .unwrap();
    let (dx, dy) = (centre.x - seen.centre.x, centre.y - seen.centre.y);
    let len = dx.hypot(dy);
    assert!(len > 1., "the centres must differ for an origin swing");
    let d = direction_of(&layout, 3);
    assert!(
        (d[0] - dx / len).abs() < 1e-9 && (d[1] - dy / len).abs() < 1e-9,
        "from window 2's last seen centre {:?} to {centre:?}: {d:?}",
        seen.centre
    );
}

#[test]
fn one_swinger_mid_workspace_switch_and_its_centre_is_on_screen() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        Op::FocusWorkspaceDown,
        glass_window(2),
    ]);
    layout.update_render_elements(None);
    advance(&mut layout, 100);
    Op::FocusWorkspaceUp.apply(&mut layout);
    advance(&mut layout, 60); // mid switch
    layout.update_render_elements(None);
    let mon = layout.monitors().next().unwrap();
    assert!(
        mon.workspace_switch.is_some(),
        "the switch is still running"
    );
    assert_eq!(
        mon.workspaces_with_render_geo().count(),
        2,
        "both workspaces are on screen"
    );
    assert_eq!(
        started_now(&layout),
        vec![1],
        "only the focused window swings"
    );
    let centre = mon.tile_centre_on_output(&1).unwrap();
    assert_near(
        centre,
        hit_centre(mon, 1, centre),
        "computed vs hit-tested centre",
    );
    let origin = layout.focus_origin.as_ref().unwrap();
    assert_eq!(origin.window, 1);
    assert_near(origin.centre, hit_centre(mon, 1, centre), "recorded origin");
}

#[test]
fn one_swinger_in_the_overview_and_its_centre_is_on_screen() {
    let mut layout = swing_layout([
        Op::AddOutput(1),
        glass_window(1),
        Op::FocusWorkspaceDown,
        glass_window(2),
    ]);
    layout.update_render_elements(None);
    Op::ToggleOverview.apply(&mut layout);
    advance(&mut layout, 100);
    Op::FocusWorkspaceUp.apply(&mut layout);
    advance(&mut layout, 30);
    layout.update_render_elements(None);
    let mon = layout.monitors().next().unwrap();
    assert!(mon.overview_zoom() < 1., "the overview is zoomed out");
    assert!(
        mon.workspaces_with_render_geo().count() >= 2,
        "both workspaces are on screen"
    );
    assert_eq!(started_now(&layout), vec![1]);
    let centre = mon.tile_centre_on_output(&1).unwrap();
    assert_near(
        centre,
        hit_centre(mon, 1, centre),
        "computed vs hit-tested centre",
    );
    assert_near(
        layout.focus_origin.as_ref().unwrap().centre,
        hit_centre(mon, 1, centre),
        "recorded origin",
    );
}

#[test]
fn a_refocus_mid_swing_does_not_restart() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1), glass_window(2)]);
    layout.update_render_elements(None);
    advance(&mut layout, 16);
    Op::FocusColumnLeft.apply(&mut layout);
    layout.update_render_elements(None);
    assert_eq!(started_now(&layout), vec![1]);
    advance(&mut layout, 100);
    Op::FocusColumnRight.apply(&mut layout);
    layout.update_render_elements(None);
    let before = swings(&layout);
    advance(&mut layout, 100);
    Op::FocusColumnLeft.apply(&mut layout);
    layout.update_render_elements(None);
    let one = |s: &[(usize, Duration, [f64; 2])]| s.iter().find(|(id, ..)| *id == 1).cloned();
    assert_eq!(
        one(&swings(&layout)),
        one(&before),
        "window 1 keeps the swing it has"
    );
}

#[test]
fn an_opening_pane_takes_no_swing_for_that_gain() {
    let mut layout = swing_layout([Op::AddOutput(1), glass_window(1)]);
    layout.update_render_elements(None);
    glass_window(2).apply(&mut layout);
    layout.start_open_animation_for_window(&2);
    layout.update_render_elements(None);
    assert!(
        !started_now(&layout).contains(&2),
        "no swing while it opens"
    );
    advance(&mut layout, 2000);
    layout.update_render_elements(None);
    assert!(
        swings(&layout).iter().all(|(id, ..)| *id != 2),
        "and none later for that gain"
    );
}
