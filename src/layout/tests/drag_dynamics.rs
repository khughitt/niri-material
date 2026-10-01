//! What jelly stimulus a pointer drag, hold and release produce, next to a
//! native column move. Records the per-frame motion residual the renderer
//! would pass to `jelly_state` and pins the phases (material-b3ce14).
//! `-- --nocapture` prints the traces.

use crate::render_helpers::material::{bevel_depth, jelly_state};

use super::*;

const FRAME_MS: i32 = 16;
/// A mid-range flex from the motion sweep, on its pinned bevel 12 / thickness 20.
const FLEX: f64 = 0.01;

/// The motion residual the renderer passes for `id` this frame.
fn residual(layout: &Layout<TestWindow>, id: usize) -> Point<f64, Logical> {
    if let Some(InteractiveMoveState::Moving(move_)) = &layout.interactive_move {
        if *move_.tile.window().id() == id {
            return move_.tile.motion_residual();
        }
    }
    layout
        .workspaces()
        .find_map(|(_, _, ws)| ws.unmap_snapshot_motion_residual(&id))
        .unwrap()
}

fn tile(layout: &Layout<TestWindow>, id: usize) -> &Tile<TestWindow> {
    if let Some(InteractiveMoveState::Moving(move_)) = &layout.interactive_move {
        if *move_.tile.window().id() == id {
            return &move_.tile;
        }
    }
    layout
        .workspaces()
        .find_map(|(_, _, ws)| ws.tiles().find(|t| *t.window().id() == id))
        .unwrap()
}

fn live_spring() -> crate::layout::drag_follower::FollowSpring {
    crate::layout::drag_follower::FollowSpring::from_config(
        &niri_config::Animations::default().window_movement.0,
    )
    .unwrap()
}

/// Magnitude of the directional flex the shader receives for `residual`.
fn flex(residual: Point<f64, Logical>) -> f64 {
    let max_flex = 0.25 * bevel_depth(12., 20.);
    let j = jelly_state(residual, (0., 0.), Size::from((100., 200.)), FLEX, max_flex);
    f64::from(j.move_[0]).hypot(f64::from(j.move_[1]))
}

#[derive(Default)]
struct Trace {
    rows: Vec<(String, i32, f64, f64)>,
    ms: i32,
}

impl Trace {
    fn record(&mut self, layout: &Layout<TestWindow>, id: usize, phase: &str) {
        let r = residual(layout, id);
        self.rows
            .push((phase.to_owned(), self.ms, r.x.hypot(r.y), flex(r)));
    }

    /// Advance `frames` frames, recording each one.
    fn run(&mut self, layout: &mut Layout<TestWindow>, id: usize, phase: &str, frames: usize) {
        for _ in 0..frames {
            self.ms += FRAME_MS;
            Op::AdvanceAnimations {
                msec_delta: FRAME_MS,
            }
            .apply(layout);
            self.record(layout, id, phase);
        }
    }

    /// Drag the pointer by `step` per frame for `frames` frames.
    fn drag(
        &mut self,
        layout: &mut Layout<TestWindow>,
        id: usize,
        phase: &str,
        pointer: &mut Point<f64, Logical>,
        step: Point<f64, Logical>,
        frames: usize,
    ) {
        for _ in 0..frames {
            *pointer += step;
            Op::InteractiveMoveUpdate {
                window: id,
                dx: step.x,
                dy: step.y,
                output_idx: 1,
                px: pointer.x,
                py: pointer.y,
            }
            .apply(layout);
            self.run(layout, id, phase, 1);
        }
    }

    fn peak(&self, phase: &str) -> f64 {
        self.rows
            .iter()
            .filter(|row| row.0 == phase)
            .map(|row| row.3)
            .fold(0., f64::max)
    }

    /// Frames in `phase` until flex falls below 1% of its peak.
    fn settle_ms(&self, phase: &str) -> i32 {
        let peak = self.peak(phase);
        let rows: Vec<_> = self.rows.iter().filter(|row| row.0 == phase).collect();
        let start = rows.first().map_or(0, |row| row.1 - FRAME_MS);
        rows.iter()
            .rev()
            .find(|row| row.3 >= 0.01 * peak)
            .map_or(0, |row| row.1 - start)
    }

    fn print(&self, name: &str) {
        eprintln!("# {name}: phase, ms, |residual| px, |flex| px");
        for (phase, ms, r, f) in &self.rows {
            eprintln!("{name}\t{phase}\t{ms}\t{r:.3}\t{f:.5}");
        }
    }
}

fn begin(layout: &mut Layout<TestWindow>, id: usize, pointer: Point<f64, Logical>) {
    Op::InteractiveMoveBegin {
        window: id,
        output_idx: 1,
        px: pointer.x,
        py: pointer.y,
    }
    .apply(layout);
}

fn end(layout: &mut Layout<TestWindow>, id: usize) {
    Op::InteractiveMoveEnd { window: id }.apply(layout);
}

fn two_columns() -> Layout<TestWindow> {
    check_ops([
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        Op::AddWindow {
            params: TestWindowParams::new(2),
        },
        Op::FocusColumnFirst,
        Op::CompleteAnimations,
    ])
}

#[test]
fn scrolling_drag_flexes_through_lift_drag_and_release() {
    let mut layout = two_columns();
    let mut trace = Trace::default();
    let mut pointer = Point::from((50., 100.));
    begin(&mut layout, 1, pointer);

    // Below the start threshold the tile rubber-bands with the grab offset.
    trace.drag(
        &mut layout,
        1,
        "rubber-band",
        &mut pointer,
        Point::from((20., 0.)),
        12,
    );
    assert!(matches!(
        layout.interactive_move,
        Some(InteractiveMoveState::Starting { .. })
    ));
    // Crossing it lifts the tile out of the column to the pointer.
    trace.drag(
        &mut layout,
        1,
        "lift",
        &mut pointer,
        Point::from((20., 0.)),
        1,
    );
    assert!(matches!(
        layout.interactive_move,
        Some(InteractiveMoveState::Moving(_))
    ));
    trace.run(&mut layout, 1, "lift", 30);
    // A fast drag, then a hold with the pointer still.
    trace.drag(
        &mut layout,
        1,
        "drag",
        &mut pointer,
        Point::from((40., 10.)),
        15,
    );
    trace.run(&mut layout, 1, "hold", 30);
    end(&mut layout, 1);
    trace.record(&layout, 1, "release");
    trace.run(&mut layout, 1, "release", 60);
    trace.print("scrolling");

    assert_eq!(trace.peak("rubber-band"), 0.);
    assert!(trace.peak("lift") > 0.);
    assert!(trace.peak("drag") > 0.5, "drag {}", trace.peak("drag"));
    // The hold starts with the lag still decaying, and ends settled.
    assert!(trace.rows.iter().rfind(|r| r.0 == "hold").unwrap().3 < 1e-3);
    assert!(trace.peak("release") > 0.);
    eprintln!(
        "scrolling peaks: lift {:.4} ({} ms), drag {:.4}, hold settle {} ms, release {:.4} ({} ms)",
        trace.peak("lift"),
        trace.settle_ms("lift"),
        trace.peak("drag"),
        trace.settle_ms("hold"),
        trace.peak("release"),
        trace.settle_ms("release"),
    );
}

#[test]
fn scrolling_drag_cancelled_below_threshold_springs_back_with_flex() {
    let mut layout = two_columns();
    let mut trace = Trace::default();
    let mut pointer = Point::from((50., 100.));
    begin(&mut layout, 1, pointer);
    trace.drag(
        &mut layout,
        1,
        "rubber-band",
        &mut pointer,
        Point::from((20., 0.)),
        12,
    );
    end(&mut layout, 1);
    trace.record(&layout, 1, "cancel");
    trace.run(&mut layout, 1, "cancel", 60);
    trace.print("cancel");

    assert_eq!(trace.peak("rubber-band"), 0.);
    assert!(trace.peak("cancel") > 0.);
}

#[test]
fn floating_drag_flexes_on_lift_and_drag() {
    let mut layout = check_ops([
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams {
                is_floating: true,
                ..TestWindowParams::new(1)
            },
        },
        Op::CompleteAnimations,
    ]);
    let (_, pos) = layout
        .active_workspace()
        .unwrap()
        .floating()
        .tiles_with_offsets()
        .next()
        .unwrap();
    let mut trace = Trace::default();
    let mut pointer = pos + Point::from((50., 100.));
    begin(&mut layout, 1, pointer);
    trace.drag(
        &mut layout,
        1,
        "lift",
        &mut pointer,
        Point::from((20., 0.)),
        1,
    );
    trace.run(&mut layout, 1, "lift", 30);
    trace.drag(
        &mut layout,
        1,
        "drag",
        &mut pointer,
        Point::from((40., 10.)),
        15,
    );
    trace.run(&mut layout, 1, "hold", 30);
    end(&mut layout, 1);
    trace.record(&layout, 1, "release");
    trace.run(&mut layout, 1, "release", 60);
    trace.print("floating");

    // Floating skips the start threshold: the tile catches up with the
    // first pointer step from its rubber-banded spot, then follows exactly.
    assert!(trace.peak("lift") > 0.);
    assert!(trace.peak("drag") > 0.5, "drag {}", trace.peak("drag"));
    assert!(trace.rows.iter().rfind(|r| r.0 == "hold").unwrap().3 < 1e-3);
    // Dropped in place after a settled hold: no release stimulus.
    assert!(
        trace.peak("release") < 1e-3,
        "release {}",
        trace.peak("release")
    );
    eprintln!(
        "floating peaks: lift {:.4} ({} ms), drag {:.4}, hold settle {} ms",
        trace.peak("lift"),
        trace.settle_ms("lift"),
        trace.peak("drag"),
        trace.settle_ms("hold"),
    );
}

#[test]
fn native_column_move_flexes() {
    let mut layout = two_columns();
    let mut trace = Trace::default();
    Op::MoveColumnRight.apply(&mut layout);
    trace.record(&layout, 1, "move");
    trace.run(&mut layout, 1, "move", 60);
    trace.print("native");

    assert!(trace.peak("move") > 0.);
    eprintln!(
        "native peak {:.4} ({} ms)",
        trace.peak("move"),
        trace.settle_ms("move")
    );
}

/// Drives a lifted tile at `step` per frame, comparing every recorded lag against a
/// reference follower given the same deltas at the same clock times. Returns the
/// reference, which later checks keep evaluating.
fn drag_against_reference(
    layout: &mut Layout<TestWindow>,
    id: usize,
    mut pointer: Point<f64, Logical>,
    step: Point<f64, Logical>,
    frames: usize,
) -> crate::layout::drag_follower::DragFollower {
    use crate::layout::drag_follower::DragFollower;
    let mut reference: Option<DragFollower> = None;
    for _ in 0..frames {
        pointer += step;
        let now = layout.clock.now();
        reference
            .get_or_insert_with(|| DragFollower::new(live_spring(), now))
            .shift(now, step);
        Op::InteractiveMoveUpdate {
            window: id,
            dx: step.x,
            dy: step.y,
            output_idx: 1,
            px: pointer.x,
            py: pointer.y,
        }
        .apply(layout);
        Op::AdvanceAnimations {
            msec_delta: FRAME_MS,
        }
        .apply(layout);
        let got = tile(layout, id).drag_lag();
        let want = reference.as_ref().unwrap().lag(layout.clock.now());
        assert!(
            (got.x - want.x).abs() < 1e-9 && (got.y - want.y).abs() < 1e-9,
            "{got:?} vs {want:?}"
        );
    }
    reference.unwrap()
}

fn lifted(layout: &mut Layout<TestWindow>, id: usize, pointer: &mut Point<f64, Logical>) {
    begin(layout, id, *pointer);
    let mut trace = Trace::default();
    trace.drag(
        layout,
        id,
        "rubber-band",
        pointer,
        Point::from((20., 0.)),
        13,
    );
    assert!(matches!(
        layout.interactive_move,
        Some(InteractiveMoveState::Moving(_))
    ));
    trace.run(layout, id, "lift", 30);
}

#[test]
fn drag_lag_matches_reference_follower() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 10.)), 15);
}

#[test]
fn hold_drops_follower_and_stops_frames() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 10.)), 15);
    assert!(tile(&layout, 1).has_drag_follower());
    let mut frames = 0;
    while tile(&layout, 1).has_drag_follower() {
        Op::AdvanceAnimations {
            msec_delta: FRAME_MS,
        }
        .apply(&mut layout);
        frames += 1;
        assert!(frames < 120, "follower alive after ~2 s of hold");
    }
    assert!(!tile(&layout, 1).are_animations_ongoing());
}

#[test]
fn render_location_ignores_lag() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 10.)), 5);
    let t = tile(&layout, 1);
    assert!(t.drag_lag().x.abs() > 1.);
    assert_eq!(
        t.render_offset(),
        t.animation_residual() + t.interactive_move_offset
    );
}

/// Releases in the frame of the last drag step, in each layout (§6, release continuity).
fn release_in_motion(
    mut layout: Layout<TestWindow>,
    id: usize,
    mut pointer: Point<f64, Logical>,
    floating: bool,
) {
    if floating {
        begin(&mut layout, id, pointer);
        let mut trace = Trace::default();
        trace.drag(
            &mut layout,
            id,
            "lift",
            &mut pointer,
            Point::from((20., 0.)),
            1,
        );
        trace.run(&mut layout, id, "lift", 30);
    } else {
        lifted(&mut layout, id, &mut pointer);
    }
    let reference = drag_against_reference(&mut layout, id, pointer, Point::from((40., 10.)), 6);
    let lag_before = tile(&layout, id).drag_lag();
    assert!(lag_before.x.hypot(lag_before.y) > 10., "{lag_before:?}");
    end(&mut layout, id);
    let t = tile(&layout, id);
    let release_term = t.animation_residual();
    let lag_now = t.motion_residual() - release_term;
    assert!((lag_now.x - lag_before.x).abs() < 1e-9 && (lag_now.y - lag_before.y).abs() < 1e-9);
    if floating {
        assert!(release_term.x.hypot(release_term.y) < 1e-9);
    }
    // One frame later the lag term is the same follower evaluated 16 ms on.
    Op::AdvanceAnimations {
        msec_delta: FRAME_MS,
    }
    .apply(&mut layout);
    let t = tile(&layout, id);
    let got = t.motion_residual() - t.animation_residual();
    let want = reference.lag(layout.clock.now());
    assert!(
        (got.x - want.x).abs() < 1e-9 && (got.y - want.y).abs() < 1e-9,
        "{got:?} vs {want:?}"
    );
}

#[test]
fn scrolling_release_in_motion_keeps_lag_continuous() {
    release_in_motion(two_columns(), 1, Point::from((50., 100.)), false);
}

#[test]
fn floating_release_in_motion_keeps_lag_continuous() {
    let layout = check_ops([
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams {
                is_floating: true,
                ..TestWindowParams::new(1)
            },
        },
        Op::CompleteAnimations,
    ]);
    let (_, pos) = layout
        .active_workspace()
        .unwrap()
        .floating()
        .tiles_with_offsets()
        .next()
        .unwrap();
    release_in_motion(layout, 1, pos + Point::from((50., 100.)), true);
}

type ReleaseRow = (
    Point<f64, Logical>,
    Point<f64, Logical>,
    Point<f64, Logical>,
    Point<f64, Logical>,
);

/// One release run: lift, drag `steps` (each a per-frame pointer step), release in
/// the frame of the last step, then 40 frames. With `follow == false` the moving
/// tile's follower is cleared after every update, so the run is the pre-follower
/// behaviour with identical layout ops. Returns per-frame (render offset, tile
/// position in the workspace, motion residual, animation residual), the first row
/// read at the release instant.
fn release_run(steps: &[Point<f64, Logical>], follow: bool) -> Vec<ReleaseRow> {
    let mut layout = two_columns();
    let mut pointer = Point::from((300., 100.));
    lifted(&mut layout, 1, &mut pointer);
    for step in steps {
        pointer += *step;
        Op::InteractiveMoveUpdate {
            window: 1,
            dx: step.x,
            dy: step.y,
            output_idx: 1,
            px: pointer.x,
            py: pointer.y,
        }
        .apply(&mut layout);
        if !follow {
            let Some(InteractiveMoveState::Moving(move_)) = &mut layout.interactive_move else {
                unreachable!()
            };
            move_.tile.clear_drag_follower();
        }
        Op::AdvanceAnimations {
            msec_delta: FRAME_MS,
        }
        .apply(&mut layout);
    }
    end(&mut layout, 1);
    let mut rows = Vec::new();
    for frame in 0..=40 {
        if frame > 0 {
            Op::AdvanceAnimations {
                msec_delta: FRAME_MS,
            }
            .apply(&mut layout);
        }
        let (_, _, ws) = layout
            .workspaces()
            .find(|(_, _, ws)| ws.has_window(&1))
            .unwrap();
        let (t, pos, _) = ws
            .tiles_with_render_positions()
            .find(|(t, _, _)| *t.window().id() == 1)
            .unwrap();
        rows.push((
            t.render_offset(),
            pos,
            t.motion_residual(),
            t.animation_residual(),
        ));
    }
    rows
}

/// The pinned opposing fixture (§4.4): a rightward drag released with the drop
/// slot behind it. The probe gives lag -110.432 px and release +488 px.
const OPPOSING: &[(f64, f64)] = &[(40., 0.), (40., 0.), (40., 0.), (40., 0.)];

#[test]
fn opposing_release_lowers_flex_and_keeps_the_trajectory() {
    let steps: Vec<_> = OPPOSING.iter().map(|&(x, y)| Point::from((x, y))).collect();
    let with = release_run(&steps, true);
    let without = release_run(&steps, false);
    // The fixture is opposing: lag against the release term, smaller than it.
    let (_, _, motion, release) = with[0];
    let lag = motion - release;
    let dot = lag.x * release.x + lag.y * release.y;
    assert!(
        dot < 0.,
        "fixture no longer opposing: lag {lag:?} release {release:?}"
    );
    assert!(
        lag.x.hypot(lag.y) < release.x.hypot(release.y),
        "lag {lag:?} reverses, not lowers"
    );
    assert!(flex(motion) < flex(release));
    // The paired run without the follower has the same release term and no lag.
    assert_eq!(without[0].3, release);
    assert_eq!(without[0].2, without[0].3);
    // Window trajectory identical frame by frame.
    for (i, (a, b)) in with.iter().zip(&without).enumerate() {
        assert_eq!(a.0, b.0, "render offset, frame {i}");
        assert_eq!(a.1, b.1, "tile position, frame {i}");
        assert_eq!(a.3, b.3, "release term, frame {i}");
    }
}

#[test]
fn aligned_release_raises_flex_and_keeps_the_trajectory() {
    // A drag released while still moving the way it went, so the lag adds to the
    // release term. The probe gives lag +110.432 px and release +168 px.
    const ALIGNED: &[(f64, f64)] = &[(-40., 0.), (-40., 0.), (-40., 0.), (-40., 0.)];
    let steps: Vec<_> = ALIGNED.iter().map(|&(x, y)| Point::from((x, y))).collect();
    let with = release_run(&steps, true);
    let without = release_run(&steps, false);
    let (_, _, motion, release) = with[0];
    let lag = motion - release;
    assert!(
        lag.x * release.x + lag.y * release.y > 0.,
        "fixture no longer aligned: lag {lag:?} release {release:?}"
    );
    assert!(flex(motion) >= flex(release));
    for (i, (a, b)) in with.iter().zip(&without).enumerate() {
        assert_eq!(a.0, b.0, "render offset, frame {i}");
        assert_eq!(a.1, b.1, "tile position, frame {i}");
    }
}

#[test]
fn overview_drag_shifts_in_workspace_units() {
    let mut layout = two_columns();
    Op::ToggleOverview.apply(&mut layout);
    Op::CompleteAnimations.apply(&mut layout);
    let zoom = layout.overview_zoom();
    assert!(zoom < 1.);
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    let step = Point::from((40., 0.));
    let now = layout.clock.now();
    Op::InteractiveMoveUpdate {
        window: 1,
        dx: step.x,
        dy: step.y,
        output_idx: 1,
        px: pointer.x + step.x,
        py: pointer.y,
    }
    .apply(&mut layout);
    let lag = tile(&layout, 1).drag_lag();
    let mut reference = crate::layout::drag_follower::DragFollower::new(live_spring(), now);
    reference.shift(now, step.downscale(zoom));
    assert!((lag.x - reference.lag(layout.clock.now()).x).abs() < 1e-9);
}

#[test]
fn moving_tile_snapshot_residual_includes_lag() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 10.)), 6);
    let Some(InteractiveMoveState::Moving(move_)) = &layout.interactive_move else {
        unreachable!()
    };
    // The residual `store_unmap_snapshot` hands the moving tile's snapshot.
    let snap = move_.unmap_snapshot_motion_residual();
    let lag = move_.tile.drag_lag();
    assert!(lag.x.hypot(lag.y) > 1.);
    assert_eq!(snap, move_.tile.animation_residual() + lag);
}

#[test]
fn placed_tile_snapshot_residual_includes_lag() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 10.)), 6);
    end(&mut layout, 1);
    // The residual `Workspace::store_unmap_snapshot_if_empty` selects for a placed tile.
    let snap = |layout: &Layout<TestWindow>| {
        layout
            .workspaces()
            .find_map(|(_, _, ws)| ws.unmap_snapshot_motion_residual(&1))
            .unwrap()
    };
    let with_lag = snap(&layout);
    let lag = tile(&layout, 1).drag_lag();
    assert!(lag.x.hypot(lag.y) > 1.);
    for ws in layout.workspaces_mut() {
        for t in ws.tiles_mut() {
            if *t.window().id() == 1 {
                t.clear_drag_follower();
            }
        }
    }
    let without = snap(&layout);
    assert!(
        (with_lag.x - without.x - lag.x).abs() < 1e-9
            && (with_lag.y - without.y - lag.y).abs() < 1e-9
    );
}
