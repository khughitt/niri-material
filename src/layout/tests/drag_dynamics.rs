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
            return move_.tile.animation_residual();
        }
    }
    layout
        .workspaces()
        .find_map(|(_, _, ws)| ws.unmap_snapshot_motion_residual(&id))
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
fn scrolling_drag_flexes_on_lift_and_release_but_not_while_held() {
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
    assert_eq!(trace.peak("drag"), 0.);
    assert_eq!(trace.peak("hold"), 0.);
    assert!(trace.peak("release") > 0.);
    eprintln!(
        "scrolling peaks: lift {:.4} ({} ms), release {:.4} ({} ms)",
        trace.peak("lift"),
        trace.settle_ms("lift"),
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
fn floating_drag_flexes_only_on_lift() {
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
    for phase in ["drag", "hold", "release"] {
        assert!(trace.peak(phase) < 1e-9, "{phase}: {}", trace.peak(phase));
    }
    eprintln!(
        "floating peak: lift {:.4} ({} ms)",
        trace.peak("lift"),
        trace.settle_ms("lift")
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
