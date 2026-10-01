# Drag follow-lag implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** draft, awaiting plan review round 1.

**Goal:** during an interactive move, the glass flexes with a spring-lagged follower of the
pointer. The window keeps rendering at the pointer, and the lag decays to zero on a hold
and after release.

**Architecture:** a pure `DragFollower` (anchor time, lag and velocity per axis) evaluates
a damped spring in closed form, clamped at its anchor. `Tile` owns an optional follower,
shifts it from the `Moving` branch of `Layout::interactive_move_update`, and adds its lag
to the residual handed to the jelly through a new `Tile::motion_residual()`. Window
geometry is untouched.

**Tech Stack:** Rust (niri, smithay `Point`, std `Duration`), nextest through `just`,
Bash and Python stdlib for the clip driver. No new dependency, config key or IPC.

**Spec:** [accepted design, round 6](../specs/2026-10-01-drag-follow-lag-design.md).
Read it first; section references below (§3.4 and so on) point into it.

## Global constraints

- Settle rule: `|L| < 0.05` logical px **and** `|V| < 1` logical px/s, both on the vector
  magnitude.
- Spring: window-movement's `SpringParams` (`damping_ratio`, `stiffness`), mass 1. No
  follower when window-movement is `off` or `Kind::Easing`.
- Time: reads at `now < t₀` return the anchor state. Shifts apply at `max(now, t₀)`, so
  the anchor time never decreases (§3.3–3.4).
- `render_offset()` never includes the lag. `are_transitions_ongoing()` never includes
  the follower. `are_animations_ongoing()` includes it whenever it is present.
- Under `clock.should_complete_instantly()`: `drag_lag()` returns zero, `drag_follow`
  creates nothing, and `advance_animations` drops the follower.
- `Spring` (`src/animation/spring.rs`) is upstream code and stays unchanged.
- Tests run through the front door: `just test-one -p niri <filter>` while you edit,
  then `just test-fast` before each commit. Never call `cargo test` or `cargo nextest`
  directly.
- Commits are conventional, with no AI-attribution trailer. Every commit runs
  `tasks check`. The step task's `tasks done` lands in the same commit as its last
  change.

## Review focus

The spec implies these inputs, and the task tests below pin each one in the task that
owns the code:

1. **The pointer stops but the button stays down for a long time.** The follower must
   be dropped and `are_animations_ongoing()` must go false, so the idle budget holds.
   Task 2, `hold_drops_follower_and_stops_frames`.
2. **A drag in the overview, where zoom is below 1.** The shift is in workspace units
   (`delta.downscale(zoom)`), so the lag matches the unzoomed one scaled by `1/zoom`.
   Task 2, `overview_drag_shifts_in_workspace_units`.
3. **A window closed in the middle of a drag.** The unmap snapshot receives the motion
   residual, lag included, without panicking. Task 2, `unmap_mid_drag_snapshots_lag`.
4. **`animations { off }` turned on while a lag decays.** It completes like every other
   animation. Task 3, `complete_instantly_drops_follower`.
5. **High-rate pointers (1000 Hz).** The lag does not depend on event rate. Task 3,
   `staggered_events_match_reference_and_euler`.

---

### Task 1: `DragFollower` closed-form follower

**Files:**
- Create: `src/layout/drag_follower.rs`
- Modify: `src/layout/mod.rs` (module list near line 79: add `pub mod drag_follower;`)
- Test: unit tests in `src/layout/drag_follower.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Produces:
  - `pub struct FollowSpring { pub damping_ratio: f64, pub stiffness: f64 }`, which is
    `Clone + Copy + Debug + PartialEq`.
  - `FollowSpring::from_config(anim: &niri_config::Animation) -> Option<FollowSpring>`.
  - `pub struct DragFollower`, which is `Clone + Debug`.
  - `DragFollower::new(spring: FollowSpring, now: Duration) -> Self`
  - `DragFollower::shift(&mut self, now: Duration, delta: Point<f64, Logical>)`
  - `DragFollower::lag(&self, now: Duration) -> Point<f64, Logical>`
  - `DragFollower::velocity(&self, now: Duration) -> Point<f64, Logical>`
  - `DragFollower::is_settled(&self, now: Duration) -> bool`
  - `DragFollower::set_spring(&mut self, now: Duration, spring: FollowSpring)`
  - `DragFollower::spring(&self) -> FollowSpring`

- [ ] **Step 1: Write the failing tests**

Create `src/layout/drag_follower.rs` containing only the test module below. Add
`pub mod drag_follower;` after `pub mod closing_window;` in `src/layout/mod.rs`.

```rust
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use smithay::utils::{Logical, Point};

    use super::*;

    const LIVE: FollowSpring = FollowSpring {
        damping_ratio: 1.,
        stiffness: 800.,
    };

    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }

    fn p(x: f64, y: f64) -> Point<f64, Logical> {
        Point::from((x, y))
    }

    /// Semi-implicit Euler at 1 µs on one axis, from (x, v) over `secs`.
    fn euler(spring: FollowSpring, mut x: f64, mut v: f64, secs: f64) -> (f64, f64) {
        let k = spring.stiffness;
        let c = 2. * spring.damping_ratio * k.sqrt();
        let h = 1e-6;
        let n = (secs / h).round() as u64;
        for _ in 0..n {
            v += (-k * x - c * v) * h;
            x += v * h;
        }
        (x, v)
    }

    #[test]
    fn closed_form_matches_euler_in_all_regimes() {
        for ratio in [0.6, 1., 1.5] {
            let spring = FollowSpring {
                damping_ratio: ratio,
                stiffness: 800.,
            };
            let mut f = DragFollower::new(spring, ms(0));
            f.shift(ms(0), p(40., -25.));
            // Give it velocity: a second shift 10 ms later.
            f.shift(ms(10), p(15., 0.));
            let (l0, v0) = (f.lag(ms(10)), f.velocity(ms(10)));
            for t in [5u64, 16, 50, 120] {
                let (ex, evx) = euler(spring, l0.x, v0.x, t as f64 / 1000.);
                let (ey, evy) = euler(spring, l0.y, v0.y, t as f64 / 1000.);
                let l = f.lag(ms(10 + t));
                let v = f.velocity(ms(10 + t));
                assert!((l.x - ex).abs() < 1e-3, "ratio {ratio} t {t}: {} vs {ex}", l.x);
                assert!((l.y - ey).abs() < 1e-3, "ratio {ratio} t {t}: {} vs {ey}", l.y);
                assert!((v.x - evx).abs() < 1e-1, "ratio {ratio} t {t}: {} vs {evx}", v.x);
                assert!((v.y - evy).abs() < 1e-1, "ratio {ratio} t {t}: {} vs {evy}", v.y);
            }
        }
    }

    #[test]
    fn velocity_is_the_derivative_of_lag() {
        for ratio in [0.6, 1., 1.5] {
            let spring = FollowSpring {
                damping_ratio: ratio,
                stiffness: 800.,
            };
            let mut f = DragFollower::new(spring, ms(0));
            f.shift(ms(0), p(-60., 30.));
            for t in [3u64, 20, 70] {
                let h = Duration::from_micros(1);
                let a = f.lag(ms(t) - h);
                let b = f.lag(ms(t) + h);
                let fd = (b.x - a.x) / 2e-6;
                let v = f.velocity(ms(t)).x;
                assert!((fd - v).abs() < 1e-2 * v.abs().max(1.), "ratio {ratio} t {t}");
            }
        }
    }

    #[test]
    fn shift_moves_lag_and_carries_velocity() {
        let mut f = DragFollower::new(LIVE, ms(0));
        f.shift(ms(0), p(-40., 0.));
        let before_l = f.lag(ms(16));
        let before_v = f.velocity(ms(16));
        f.shift(ms(16), p(10., 0.));
        assert!((f.lag(ms(16)).x - (before_l.x - 10.)).abs() < 1e-12);
        assert!((f.velocity(ms(16)).x - before_v.x).abs() < 1e-12);
        assert!(before_v.x != 0.);
    }

    #[test]
    fn same_instant_events_compose() {
        let mut one = DragFollower::new(LIVE, ms(0));
        let mut two = DragFollower::new(LIVE, ms(0));
        one.shift(ms(5), p(20., 6.));
        two.shift(ms(5), p(10., 3.));
        two.shift(ms(5), p(10., 3.));
        assert_eq!(one.lag(ms(40)), two.lag(ms(40)));
    }

    #[test]
    fn reads_before_the_anchor_return_the_anchor_state() {
        let mut f = DragFollower::new(LIVE, ms(0));
        f.shift(ms(100), p(30., 0.));
        f.shift(ms(116), p(30., 0.));
        let anchor_l = f.lag(ms(116));
        let anchor_v = f.velocity(ms(116));
        assert_eq!(f.lag(ms(100)), anchor_l);
        assert_eq!(f.velocity(ms(100)), anchor_v);
        // Continuous at the boundary.
        let after = f.lag(ms(116) + Duration::from_nanos(1));
        assert!((after.x - anchor_l.x).abs() < 1e-6);
        // A shift before the anchor applies at the anchor; the anchor time holds.
        f.shift(ms(108), p(5., 0.));
        assert!((f.lag(ms(116)).x - (anchor_l.x - 5.)).abs() < 1e-12);
        assert_eq!(f.lag(ms(90)), f.lag(ms(116)));
    }

    #[test]
    fn high_damping_and_stiffness_stay_finite() {
        let heavy = FollowSpring {
            damping_ratio: 10.,
            stiffness: 800.,
        };
        let mut f = DragFollower::new(heavy, ms(0));
        f.shift(ms(0), p(40., 0.));
        let mut settled_at = None;
        for i in 0..=(60_000 / 16) {
            let t = ms(i * 16);
            let (l, v) = (f.lag(t), f.velocity(t));
            assert!(l.x.is_finite() && v.x.is_finite(), "t {i}: {l:?} {v:?}");
            if settled_at.is_none() && f.is_settled(t) {
                settled_at = Some(t);
            }
        }
        assert!(settled_at.is_some(), "never settled");
        let (ex, _) = euler(heavy, -40., 0., 0.5);
        assert!((f.lag(ms(500)).x - ex).abs() < 1e-3);

        let stiff = FollowSpring {
            damping_ratio: 10.,
            stiffness: 1e8,
        };
        let mut g = DragFollower::new(stiff, ms(1000));
        g.shift(ms(1000), p(40., 0.));
        let back = g.lag(ms(984));
        assert!(back.x.is_finite());
        assert_eq!(back, g.lag(ms(1000)));
    }

    #[test]
    fn settle_rule_uses_both_thresholds() {
        let mut f = DragFollower::new(LIVE, ms(0));
        assert!(f.is_settled(ms(0)));
        f.shift(ms(0), p(-100., 0.));
        assert!(!f.is_settled(ms(0)));
        let mut t = 0;
        while !f.is_settled(ms(t)) {
            t += 1;
            assert!(t < 5000, "did not settle in 5 s");
        }
        assert!(f.lag(ms(t)).x.hypot(f.lag(ms(t)).y) < 0.05);
        assert!(f.velocity(ms(t)).x.hypot(f.velocity(ms(t)).y) < 1.);
    }

    #[test]
    fn steady_drag_reaches_the_discrete_fixed_point() {
        // 50 events of 40 px, each followed by a 16 ms read: the harness ordering.
        let mut f = DragFollower::new(LIVE, ms(0));
        let mut recorded = 0.;
        for i in 0..50u64 {
            f.shift(ms(i * 16), p(40., 0.));
            recorded = f.lag(ms((i + 1) * 16)).x;
        }
        // Fixed point of the same closed form, iterated to convergence.
        let mut g = DragFollower::new(LIVE, ms(0));
        let mut fixed = 0.;
        for i in 0..400u64 {
            g.shift(ms(i * 16), p(40., 0.));
            fixed = g.lag(ms((i + 1) * 16)).x;
        }
        assert!((recorded - fixed).abs() < 1e-6, "{recorded} vs {fixed}");
        assert!((fixed + 156.79).abs() < 0.01, "{fixed}");
    }

    #[test]
    fn set_spring_keeps_state_continuous() {
        let mut f = DragFollower::new(LIVE, ms(0));
        f.shift(ms(0), p(-80., 20.));
        let (l, v) = (f.lag(ms(30)), f.velocity(ms(30)));
        f.set_spring(
            ms(30),
            FollowSpring {
                damping_ratio: 0.7,
                stiffness: 300.,
            },
        );
        assert_eq!(f.lag(ms(30)), l);
        assert_eq!(f.velocity(ms(30)), v);
    }

    #[test]
    fn from_config_takes_springs_only() {
        use niri_config::animations::{Animation, Curve, EasingParams, Kind, SpringParams};
        let spring = Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 1.,
                stiffness: 800,
                epsilon: 0.0001,
            }),
        };
        assert_eq!(FollowSpring::from_config(&spring), Some(LIVE));
        assert_eq!(
            FollowSpring::from_config(&Animation { off: true, ..spring }),
            None
        );
        let eased = Animation {
            off: false,
            kind: Kind::Easing(EasingParams {
                duration_ms: 250,
                curve: Curve::EaseOutCubic,
            }),
        };
        assert_eq!(FollowSpring::from_config(&eased), None);
    }
}
```

Before running, check `niri-config/src/animations.rs` for the exact `EasingParams`
fields and a `Curve` variant name. If `EaseOutCubic` does not exist, use any variant
listed there.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-one -p niri drag_follower`
Expected: compile errors (`cannot find type FollowSpring`, `DragFollower`).

- [ ] **Step 3: Implement**

Put this above the test module in `src/layout/drag_follower.rs`:

```rust
//! Follow-lag stimulus for an interactive move: a damped spring point that chases the
//! pointer, whose lag the jelly receives as motion residual
//! (docs/specs/2026-10-01-drag-follow-lag-design.md).

use std::time::Duration;

use smithay::utils::{Logical, Point};

/// Below both, the follower is settled and the tile drops it (§3.8).
const SETTLE_LAG: f64 = 0.05;
const SETTLE_VELOCITY: f64 = 1.;
/// Damping ratios this close to 1 use the critically damped form.
const CRITICAL_BAND: f64 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FollowSpring {
    pub damping_ratio: f64,
    pub stiffness: f64,
}

impl FollowSpring {
    /// Window-movement's spring; none when it is off or eased (§4.3).
    pub fn from_config(anim: &niri_config::Animation) -> Option<Self> {
        if anim.off {
            return None;
        }
        match anim.kind {
            niri_config::animations::Kind::Spring(p) => Some(Self {
                damping_ratio: p.damping_ratio.max(0.),
                stiffness: f64::from(p.stiffness),
            }),
            niri_config::animations::Kind::Easing(_) => None,
        }
    }

    /// Position and velocity after `t` seconds (t >= 0) from (x0, v0), unit mass.
    fn evolve(self, x0: f64, v0: f64, t: f64) -> (f64, f64) {
        debug_assert!(t >= 0.);
        let w0 = self.stiffness.sqrt();
        let z = self.damping_ratio;
        if w0 == 0. {
            return (x0 + v0 * t, v0);
        }
        let (x, v) = if (z - 1.).abs() <= CRITICAL_BAND {
            let e = (-w0 * t).exp();
            let b = v0 + w0 * x0;
            ((x0 + b * t) * e, (v0 - b * w0 * t) * e)
        } else if z < 1. {
            let beta = z * w0;
            let w1 = w0 * (1. - z * z).sqrt();
            let e = (-beta * t).exp();
            let (s, c) = (w1 * t).sin_cos();
            (
                e * (x0 * c + (v0 + beta * x0) / w1 * s),
                e * (v0 * c - (beta * v0 + w0 * w0 * x0) / w1 * s),
            )
        } else {
            // Two decaying exponentials; never cosh/sinh, which overflow (§5).
            let beta = z * w0;
            let w2 = w0 * (z * z - 1.).sqrt();
            // λ1 = −β + ω2 written without cancellation: λ1·λ2 = ω0².
            let l2 = -beta - w2;
            let l1 = w0 * w0 / l2;
            let c1 = (v0 - l2 * x0) / (l1 - l2);
            let c2 = x0 - c1;
            let (e1, e2) = ((l1 * t).exp(), (l2 * t).exp());
            (c1 * e1 + c2 * e2, l1 * c1 * e1 + l2 * c2 * e2)
        };
        debug_assert!(x.is_finite() && v.is_finite(), "follower state {x} {v}");
        (x, v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Axis {
    lag: f64,
    velocity: f64,
}

/// The anchor (t₀, L₀, V₀) is the only state; reads evaluate the closed form forward
/// from it, and reads before it return it (§3.1–3.4).
#[derive(Debug, Clone)]
pub struct DragFollower {
    spring: FollowSpring,
    t0: Duration,
    x: Axis,
    y: Axis,
}

impl DragFollower {
    pub fn new(spring: FollowSpring, now: Duration) -> Self {
        let rest = Axis {
            lag: 0.,
            velocity: 0.,
        };
        Self {
            spring,
            t0: now,
            x: rest,
            y: rest,
        }
    }

    pub fn spring(&self) -> FollowSpring {
        self.spring
    }

    fn state_at(&self, now: Duration) -> (Axis, Axis) {
        let dt = now.saturating_sub(self.t0).as_secs_f64();
        let step = |a: Axis| {
            let (lag, velocity) = self.spring.evolve(a.lag, a.velocity, dt);
            Axis { lag, velocity }
        };
        (step(self.x), step(self.y))
    }

    fn reanchor(&mut self, now: Duration) {
        let t = now.max(self.t0);
        let (x, y) = self.state_at(t);
        self.t0 = t;
        self.x = x;
        self.y = y;
    }

    /// The target moved by `delta`; the follower did not (§3.4).
    pub fn shift(&mut self, now: Duration, delta: Point<f64, Logical>) {
        self.reanchor(now);
        self.x.lag -= delta.x;
        self.y.lag -= delta.y;
    }

    pub fn lag(&self, now: Duration) -> Point<f64, Logical> {
        let (x, y) = self.state_at(now);
        Point::from((x.lag, y.lag))
    }

    pub fn velocity(&self, now: Duration) -> Point<f64, Logical> {
        let (x, y) = self.state_at(now);
        Point::from((x.velocity, y.velocity))
    }

    pub fn is_settled(&self, now: Duration) -> bool {
        let (x, y) = self.state_at(now);
        x.lag.hypot(y.lag) < SETTLE_LAG && x.velocity.hypot(y.velocity) < SETTLE_VELOCITY
    }

    /// New spring parameters from `now` on, with lag and velocity continuous.
    pub fn set_spring(&mut self, now: Duration, spring: FollowSpring) {
        self.reanchor(now);
        self.spring = spring;
    }
}
```

The `w0 == 0.` branch covers a configured stiffness of 0. Such a follower never settles
on its own. Task 3 makes `drag_follow` refuse to create one, so this branch only keeps
the type total.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just test-one -p niri drag_follower`
Expected: all ten tests PASS. If `closed_form_matches_euler_in_all_regimes` misses on
velocity by a small margin, check the Euler step before you touch the tolerance. The
1 µs semi-implicit step is accurate to about 1e-4 there.

- [ ] **Step 5: Gate and commit**

Run: `just test-fast`, `cargo fmt --all`, `tasks check`. Then mark the step task done
and commit:

```bash
tasks done material-0daa80 "DragFollower closed-form follower with anchor clamp"
git add src/layout/drag_follower.rs src/layout/mod.rs tasks/
git commit -m "feat(material): closed-form drag follower (material-4354cf)"
```

---

### Task 2: Tile owns the follower; drag drives it; jelly reads it

**Files:**
- Modify: `src/layout/tile.rs`. Add the field near `move_y_animation` (line ~108) and its
  initialiser (~452). Change `animation_residual` (~1174) and add `drag_follow`,
  `drag_lag` and `motion_residual`. Change `advance_animations` (~913) and
  `are_animations_ongoing` (~973).
- Modify: `src/layout/mod.rs`. In the `interactive_move_update` `Moving` branch
  (~4088), call the follower. Switch two call sites (~4749, ~4908) to
  `motion_residual()`.
- Modify: `src/layout/scrolling.rs`. Switch two call sites (~2315, ~2982).
- Modify: `src/layout/floating.rs`. Switch one call site (~1093).
- Modify: `src/layout/workspace.rs`. Switch one call site (~1696).
- Test: `src/layout/tests/drag_dynamics.rs`.

**Interfaces:**
- Consumes: `FollowSpring::from_config`, `DragFollower::{new, shift, lag, is_settled}`
  (Task 1).
- Produces:
  - `Tile::drag_follow(&mut self, delta: Point<f64, Logical>)`
  - `Tile::drag_lag(&self) -> Point<f64, Logical>`
  - `Tile::motion_residual(&self) -> Point<f64, Logical>`
  - `Tile::has_drag_follower(&self) -> bool`. This one is test-visible
    (`pub(super)`).

- [ ] **Step 1: Write the failing tests**

In `src/layout/tests/drag_dynamics.rs`:

(a) Change the `residual` helper to read the stimulus the renderer now passes. Replace
`move_.tile.animation_residual()` with `move_.tile.motion_residual()`. The workspace
fallback goes through `unmap_snapshot_motion_residual`, which Step 3 switches over.

(b) Add a tile accessor and a reference-follower helper below `residual`:

```rust
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
```

(c) Rewrite the three baseline assertions that pinned the old zero stimulus (§6).
In `scrolling_drag_flexes_on_lift_and_release_but_not_while_held`, rename the test to
`scrolling_drag_flexes_through_lift_drag_and_release`, then replace

```rust
    assert_eq!(trace.peak("drag"), 0.);
    assert_eq!(trace.peak("hold"), 0.);
```

with

```rust
    assert!(trace.peak("drag") > 0.5, "drag {}", trace.peak("drag"));
    // The hold starts with the lag still decaying, and ends settled.
    assert!(trace.rows.iter().filter(|r| r.0 == "hold").last().unwrap().3 < 1e-3);
```

In `floating_drag_flexes_only_on_lift`, rename the test to
`floating_drag_flexes_on_lift_and_drag`, then replace the `for phase in ["drag", "hold",
"release"]` loop with

```rust
    assert!(trace.peak("drag") > 0.5, "drag {}", trace.peak("drag"));
    assert!(trace.rows.iter().filter(|r| r.0 == "hold").last().unwrap().3 < 1e-3);
    // Dropped in place after a settled hold: no release stimulus.
    assert!(trace.peak("release") < 1e-3, "release {}", trace.peak("release"));
```

Keep the rubber-band and native assertions as they are. The rubber band must still read 0.

(d) Add these tests:

```rust
/// Drives a lifted tile at `step` per frame, comparing every recorded lag against a
/// reference follower given the same deltas at the same clock times. Returns the
/// reference, which later checks keep evaluating.
fn drag_against_reference(layout: &mut Layout<TestWindow>, id: usize, mut pointer: Point<f64, Logical>, step: Point<f64, Logical>, frames: usize) -> crate::layout::drag_follower::DragFollower {
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
        Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(layout);
        let got = tile(layout, id).drag_lag();
        let want = reference.as_ref().unwrap().lag(layout.clock.now());
        assert!((got.x - want.x).abs() < 1e-9 && (got.y - want.y).abs() < 1e-9, "{got:?} vs {want:?}");
    }
    reference.unwrap()
}

fn lifted(layout: &mut Layout<TestWindow>, id: usize, pointer: &mut Point<f64, Logical>) {
    begin(layout, id, *pointer);
    let mut trace = Trace::default();
    trace.drag(layout, id, "rubber-band", pointer, Point::from((20., 0.)), 13);
    assert!(matches!(layout.interactive_move, Some(InteractiveMoveState::Moving(_))));
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
        Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(&mut layout);
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
    assert_eq!(t.render_offset(), t.animation_residual() + t.interactive_move_offset);
}

/// Releases in the frame of the last drag step, in each layout (§6, release continuity).
fn release_in_motion(mut layout: Layout<TestWindow>, id: usize, mut pointer: Point<f64, Logical>, floating: bool) {
    if floating {
        begin(&mut layout, id, pointer);
        let mut trace = Trace::default();
        trace.drag(&mut layout, id, "lift", &mut pointer, Point::from((20., 0.)), 1);
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
    Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(&mut layout);
    let t = tile(&layout, id);
    let got = t.motion_residual() - t.animation_residual();
    let want = reference.lag(layout.clock.now());
    assert!((got.x - want.x).abs() < 1e-9 && (got.y - want.y).abs() < 1e-9, "{got:?} vs {want:?}");
}

#[test]
fn scrolling_release_in_motion_keeps_lag_continuous() {
    release_in_motion(two_columns(), 1, Point::from((50., 100.)), false);
}

#[test]
fn floating_release_in_motion_keeps_lag_continuous() {
    let layout = check_ops([
        Op::AddOutput(1),
        Op::AddWindow { params: TestWindowParams { is_floating: true, ..TestWindowParams::new(1) } },
        Op::CompleteAnimations,
    ]);
    let (_, pos) = layout.active_workspace().unwrap().floating().tiles_with_offsets().next().unwrap();
    release_in_motion(layout, 1, pos + Point::from((50., 100.)), true);
}

#[test]
fn opposing_release_projects_below_the_release_term() {
    // Two drags: rightward and leftward from the lifted spot. For each, the lag's sign
    // against the release term decides whether the stimulus along the release
    // direction falls short of or exceeds the release term alone (§4.4).
    let mut signs = Vec::new();
    for dir in [1., -1.] {
        let mut layout = two_columns();
        let mut pointer = Point::from((300., 100.));
        lifted(&mut layout, 1, &mut pointer);
        drag_against_reference(&mut layout, 1, pointer, Point::from((40. * dir, 0.)), 6);
        end(&mut layout, 1);
        let t = tile(&layout, 1);
        let r = t.animation_residual();
        let lag = t.motion_residual() - r;
        let r_len = r.x.hypot(r.y);
        assert!(r_len > 1., "release term {r:?}");
        let along = (t.motion_residual().x * r.x + t.motion_residual().y * r.y) / r_len;
        let dot = lag.x * r.x + lag.y * r.y;
        if dot < 0. {
            assert!(along < r_len);
            if lag.x.hypot(lag.y) < r_len {
                assert!(flex(t.motion_residual()) < flex(r));
            }
        } else {
            assert!(along >= r_len);
        }
        signs.push(dot.signum());
    }
    assert!(signs.contains(&-1.), "no opposing case: {signs:?}; lengthen or reverse the drags");
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
    Op::InteractiveMoveUpdate { window: 1, dx: step.x, dy: step.y, output_idx: 1, px: pointer.x + step.x, py: pointer.y }.apply(&mut layout);
    let lag = tile(&layout, 1).drag_lag();
    let mut reference = crate::layout::drag_follower::DragFollower::new(live_spring(), now);
    reference.shift(now, step.downscale(zoom));
    assert!((lag.x - reference.lag(layout.clock.now()).x).abs() < 1e-9);
}

#[test]
fn unmap_mid_drag_snapshots_lag() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 10.)), 6);
    let lag = tile(&layout, 1).motion_residual();
    Op::CloseWindow(1).apply(&mut layout);
    // The snapshot path took `motion_residual()`; closing must not panic and the
    // layout stays consistent.
    layout.verify_invariants();
    assert!(lag.x.abs() > 1.);
}
```

Before running, check `src/layout/tests.rs` for the exact op names: `ToggleOverview`,
`CloseWindow`, and whether `verify_invariants` is the invariant check. Use the names you
find there. The intent is to enter the overview and to remove the window mid-drag. If
`overview_zoom` is private to the layout module, the tests module can already reach it,
since `tests` is a child module.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-one -p niri drag_dynamics`
Expected: compile errors for `motion_residual`, `drag_lag` and `has_drag_follower`.

- [ ] **Step 3: Implement in `Tile`**

In `src/layout/tile.rs`, add the import
`use super::drag_follower::{DragFollower, FollowSpring};`. Then add the field after
`move_y_animation`:

```rust
    /// Follow-lag of an interactive move: decays on the tile through release
    /// (docs/specs/2026-10-01-drag-follow-lag-design.md §3).
    drag_follower: Option<DragFollower>,
```

Initialise it as `drag_follower: None,` next to `move_y_animation: None,`.

Below `animation_residual`, add:

```rust
    /// The target moved by `delta` (workspace logical units) during an interactive
    /// move; the follower lags behind it.
    pub fn drag_follow(&mut self, delta: Point<f64, Logical>) {
        if self.clock.should_complete_instantly() {
            return;
        }
        let Some(spring) = FollowSpring::from_config(&self.options.animations.window_movement.0)
        else {
            return;
        };
        let now = self.clock.now();
        self.drag_follower
            .get_or_insert_with(|| DragFollower::new(spring, now))
            .shift(now, delta);
    }

    pub fn drag_lag(&self) -> Point<f64, Logical> {
        match &self.drag_follower {
            Some(f) if !self.clock.should_complete_instantly() => f.lag(self.clock.now()),
            _ => Point::from((0., 0.)),
        }
    }

    /// The jelly's motion stimulus: move animations plus the drag follow-lag.
    pub fn motion_residual(&self) -> Point<f64, Logical> {
        self.animation_residual() + self.drag_lag()
    }

    pub(super) fn has_drag_follower(&self) -> bool {
        self.drag_follower.is_some()
    }
```

In `advance_animations`, after the move-animation blocks:

```rust
        if self.drag_follower.as_ref().is_some_and(|f| {
            self.clock.should_complete_instantly() || f.is_settled(self.clock.now())
        }) {
            self.drag_follower = None;
        }
```

In `are_animations_ongoing`, add `|| self.drag_follower.is_some()` after
`self.are_transitions_ongoing()`, and extend the doc comment with: "The drag follower
lives here for the same reason: it is cosmetic and must not hold up pointer focus after
a drop."

Do **not** touch `are_transitions_ongoing`, `render_offset` or `stop_move_animations`.

- [ ] **Step 4: Drive it from the layout and switch the call sites**

In `src/layout/mod.rs`, `interactive_move_update`, `InteractiveMoveState::Moving(mut
move_)` branch: right after the `if window != move_.tile.window().id() { … return
false; }` guard, add:

```rust
                // The follower sees the pointer's own motion, in workspace units (§3.4).
                let zoom = self.overview_zoom();
                move_.tile.drag_follow(delta.downscale(zoom));
```

If a `zoom` binding already exists later in that branch, reuse it there or rename this
one. Keep only one `overview_zoom()` call.

Replace `.animation_residual()` with `.motion_residual()` at the six render and snapshot
call sites: `src/layout/mod.rs` (two), `src/layout/scrolling.rs` (two),
`src/layout/floating.rs` (one), `src/layout/workspace.rs` (one). Find them with
`grep -n 'animation_residual()' src/layout`. Leave the one in
`src/layout/tests.rs:2017` (`scrolling_unmap_snapshot_keeps_view_animation_residual`)
as it is: it pins the move-animation part.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `just test-one -p niri drag_dynamics`, then `just test-one -p niri tile`.
Expected: all PASS. `-- --nocapture` prints the traces. Record the new drag peak and
the hold settle time from the scrolling and floating traces. Task 4 puts them in the
brief.

If `opposing_release_projects_below_the_release_term` finds no opposing case, change
the drag directions or lengths until it does. The asserted relation must not change.

- [ ] **Step 6: Gate and commit**

Run: `just test-fast`, `cargo fmt --all`, `cargo clippy --all --all-targets`,
`tasks check`.

```bash
tasks done material-8540ac "Tile follower driven by interactive move; jelly reads motion_residual"
git add src/layout tasks/
git commit -m "feat(material): drag follow-lag jelly stimulus (material-4354cf)"
```

---

### Task 3: Lifecycle edges: regrab, reload, complete-instantly, event rate

**Files:**
- Modify: `src/layout/tile.rs` (`update_config`, ~473; `drag_follow`)
- Test: `src/layout/tests/drag_dynamics.rs`

**Interfaces:**
- Consumes: `Tile::{drag_follow, drag_lag, has_drag_follower}` (Task 2) and
  `DragFollower::{set_spring, spring}` (Task 1).
- Produces: no new API.

- [ ] **Step 1: Write the failing tests**

```rust
fn options_with_movement(anim: niri_config::Animation) -> Options {
    let mut options = Options::default();
    options.animations.window_movement.0 = anim;
    options
}

#[test]
fn regrab_keeps_the_decaying_follower() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 0.)), 6);
    end(&mut layout, 1);
    Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(&mut layout);
    let before = tile(&layout, 1).drag_lag();
    assert!(before.x.abs() > 1.);
    // Grab again: Starting must neither drop nor shift it.
    begin(&mut layout, 1, pointer);
    Op::InteractiveMoveUpdate { window: 1, dx: 5., dy: 0., output_idx: 1, px: pointer.x + 5., py: pointer.y }.apply(&mut layout);
    assert!(matches!(layout.interactive_move, Some(InteractiveMoveState::Starting { .. })));
    let t = tile(&layout, 1);
    assert!(t.has_drag_follower());
    assert_eq!(t.drag_lag(), before);
}

#[test]
fn stop_move_animations_leaves_the_follower() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 0.)), 4);
    let Some(InteractiveMoveState::Moving(move_)) = &mut layout.interactive_move else { unreachable!() };
    let lag = move_.tile.drag_lag();
    move_.tile.stop_move_animations();
    assert_eq!(move_.tile.drag_lag(), lag);
}

#[test]
fn no_follower_without_a_movement_spring_or_in_starting() {
    use niri_config::animations::{Animation, Curve, EasingParams, Kind};
    for anim in [
        Animation { off: true, ..niri_config::Animations::default().window_movement.0 },
        Animation { off: false, kind: Kind::Easing(EasingParams { duration_ms: 250, curve: Curve::EaseOutCubic }) },
    ] {
        let mut layout = two_columns();
        layout.update_options(options_with_movement(anim));
        let mut pointer = Point::from((50., 100.));
        lifted(&mut layout, 1, &mut pointer);
        drag_against_reference_unchecked(&mut layout, 1, pointer, Point::from((40., 0.)), 6);
        assert!(!tile(&layout, 1).has_drag_follower());
    }
    // Starting alone on a fresh tile.
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    begin(&mut layout, 1, pointer);
    let mut trace = Trace::default();
    trace.drag(&mut layout, 1, "rubber-band", &mut pointer, Point::from((20., 0.)), 12);
    assert!(!tile(&layout, 1).has_drag_follower());
    assert_eq!(trace.peak("rubber-band"), 0.);
}

#[test]
fn reload_mid_drag_drops_or_reanchors() {
    use niri_config::animations::{Animation, Curve, EasingParams, Kind, SpringParams};
    // Off and eased drop the follower.
    for anim in [
        Animation { off: true, ..niri_config::Animations::default().window_movement.0 },
        Animation { off: false, kind: Kind::Easing(EasingParams { duration_ms: 250, curve: Curve::EaseOutCubic }) },
    ] {
        let mut layout = two_columns();
        let mut pointer = Point::from((50., 100.));
        lifted(&mut layout, 1, &mut pointer);
        drag_against_reference(&mut layout, 1, pointer, Point::from((40., 0.)), 4);
        layout.update_options(options_with_movement(anim));
        assert!(!tile(&layout, 1).has_drag_follower());
    }
    // A new spring keeps the lag continuous and takes the new parameters.
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 0.)), 4);
    let lag = tile(&layout, 1).drag_lag();
    layout.update_options(options_with_movement(Animation {
        off: false,
        kind: Kind::Spring(SpringParams { damping_ratio: 0.7, stiffness: 300, epsilon: 0.0001 }),
    }));
    assert_eq!(tile(&layout, 1).drag_lag(), lag);
}

#[test]
fn complete_instantly_drops_follower() {
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    drag_against_reference(&mut layout, 1, pointer, Point::from((40., 0.)), 4);
    end(&mut layout, 1);
    assert!(tile(&layout, 1).has_drag_follower());
    Op::CompleteAnimations.apply(&mut layout);
    assert!(!tile(&layout, 1).has_drag_follower());
    assert_eq!(tile(&layout, 1).drag_lag(), Point::from((0., 0.)));
}

#[test]
fn staggered_events_match_reference_and_euler() {
    use crate::layout::drag_follower::DragFollower;
    let mut layout = two_columns();
    let mut pointer = Point::from((50., 100.));
    lifted(&mut layout, 1, &mut pointer);
    let start = layout.clock.now_unadjusted();
    layout.clock.set_unadjusted(start);
    let base = layout.clock.now().as_secs_f64();
    let quarter = Point::from((10., 2.5));
    let mut reference: Option<DragFollower> = None;
    // Euler reference on x: integrate the same impulses at 1 µs.
    let spring = live_spring();
    let (k, c) = (spring.stiffness, 2. * spring.damping_ratio * spring.stiffness.sqrt());
    let (mut ex, mut ev, mut et) = (0f64, 0f64, 0f64);
    for frame in 0..15u64 {
        for q in 0..4u64 {
            let t = start + Duration::from_millis(frame * 16 + q * 4);
            layout.clock.set_unadjusted(t);
            let now = layout.clock.now();
            reference.get_or_insert_with(|| DragFollower::new(spring, now)).shift(now, quarter);
            while et < now.as_secs_f64() - base - 1e-12 {
                ev += (-k * ex - c * ev) * 1e-6;
                ex += ev * 1e-6;
                et += 1e-6;
            }
            ex -= quarter.x;
            pointer += quarter;
            Op::InteractiveMoveUpdate { window: 1, dx: quarter.x, dy: quarter.y, output_idx: 1, px: pointer.x, py: pointer.y }.apply(&mut layout);
        }
        let boundary = start + Duration::from_millis((frame + 1) * 16);
        layout.clock.set_unadjusted(boundary);
        layout.advance_animations();
        let got = tile(&layout, 1).drag_lag();
        let frame_now = layout.clock.now();
        let want = reference.as_ref().unwrap().lag(frame_now);
        assert!((got.x - want.x).abs() < 1e-9 && (got.y - want.y).abs() < 1e-9, "frame {frame}");
        while et < frame_now.as_secs_f64() - base - 1e-12 {
            ev += (-k * ex - c * ev) * 1e-6;
            ex += ev * 1e-6;
            et += 1e-6;
        }
        assert!((got.x - ex).abs() < 1e-3, "frame {frame}: {} vs euler {ex}", got.x);
    }
}
```

Also add `drag_against_reference_unchecked`. It is the same loop as
`drag_against_reference` without the reference comparison, and the `off` and easing
cases use it because they have no follower to compare. Write it out in full; do not
factor through a flag:

```rust
fn drag_against_reference_unchecked(layout: &mut Layout<TestWindow>, id: usize, mut pointer: Point<f64, Logical>, step: Point<f64, Logical>, frames: usize) {
    for _ in 0..frames {
        pointer += step;
        Op::InteractiveMoveUpdate { window: id, dx: step.x, dy: step.y, output_idx: 1, px: pointer.x, py: pointer.y }.apply(layout);
        Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(layout);
    }
}
```

Check `src/layout/tests.rs` for how tests swap options mid-run. It is
`layout.update_options(..)` or `Op::UpdateConfig`; use whichever exists. For the
`Curve` variant, use the one Task 1 used.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `just test-one -p niri drag_dynamics`
Expected: `reload_mid_drag_drops_or_reanchors` fails, because the follower survives
`off` and easing. The others may already pass: regrab, stop-moves and
complete-instantly follow from Task 2's design. A test that passes here is still kept,
because it pins behaviour the spec requires.

- [ ] **Step 3: Implement `update_config`**

In `Tile::update_config`, right after `self.options = options;`:

```rust
        // Window-movement off or eased ends a drag lag; a new spring takes over
        // continuously (§5). Complete-instantly is handled in advance_animations: a
        // reload sets that flag only after this runs.
        match FollowSpring::from_config(&self.options.animations.window_movement.0) {
            None => self.drag_follower = None,
            Some(spring) => {
                if let Some(f) = &mut self.drag_follower {
                    if f.spring() != spring {
                        f.set_spring(self.clock.now(), spring);
                    }
                }
            }
        }
```

In `drag_follow`, refuse a spring that cannot settle. After the `let Some(spring) = …`,
add:

```rust
        if spring.stiffness <= 0. {
            return;
        }
```

Add the same `stiffness <= 0.` → drop case to the `update_config` match.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `just test-one -p niri drag_dynamics`, then `just test-one -p niri drag_follower`.
Expected: all PASS.

- [ ] **Step 5: Gate and commit**

Run: `just test-fast`, `cargo fmt --all`, `cargo clippy --all --all-targets`,
`tasks check`.

```bash
tasks done material-e866b0 "Drag follower lifecycle: regrab, reload, complete-instantly, event rate"
git add src/layout tasks/
git commit -m "feat(material): drag follower lifecycle edges (material-4354cf)"
```

---

### Task 4: Clip driver, pilot, clips and the brief

This task needs an idle host for the capture. Build and pilot first. If the host is
busy, park with `--reason quiet` and give the phase minutes.

**Files:**
- Create: `docs/materials/scripts/vdrag.py`, a stdlib Wayland client that drives
  `zwlr_virtual_pointer_v1` and `zwp_virtual_keyboard_v1`.
- Create: `docs/materials/scripts/drag-lag-clips.sh`
- Create: `tools/test_vdrag.py`, unit tests for the wire encoding.
- Modify: `docs/notes/2026-09-29-material-dynamics-brief.md` ("Drag baseline finding":
  a follow-lag column and a clip section)
- Modify: `docs/specs/2026-10-01-drag-follow-lag-design.md` (status line only)

**Interfaces:**
- Consumes: the built niri with Tasks 1–3.
- Produces: `vdrag.py drag --to X,Y,W,H --dx PX --dy PX --frames N --hz HZ --hold-ms MS
  [--mod-mask M]`, which places the pointer at (X, Y) on a W×H output, presses Mod and
  the left button, moves N steps at HZ, holds, then releases. `drag-lag-clips.sh` with `SEQUENCES` drawn from `scroll-fast
  scroll-slow float-fast float-slow native`.

- [ ] **Step 1: Write the failing wire-encoding tests**

`tools/test_vdrag.py`:

```python
import importlib.util
import pathlib
import struct
import unittest

SPEC = importlib.util.spec_from_file_location(
    "vdrag", pathlib.Path(__file__).resolve().parent.parent / "docs/materials/scripts/vdrag.py"
)
vdrag = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(vdrag)


class WireTest(unittest.TestCase):
    def test_header_packs_object_size_and_opcode(self):
        msg = vdrag.message(7, 2, vdrag.u32(1), vdrag.u32(0x110), vdrag.u32(1))
        obj, word = struct.unpack("<II", msg[:8])
        self.assertEqual(obj, 7)
        self.assertEqual(word >> 16, len(msg))
        self.assertEqual(word & 0xFFFF, 2)

    def test_fixed_is_24_8(self):
        self.assertEqual(vdrag.fixed(1.5), struct.pack("<i", 384))
        self.assertEqual(vdrag.fixed(-2.0), struct.pack("<i", -512))

    def test_string_is_padded_and_nul_terminated(self):
        s = vdrag.string("wl_seat")
        (n,) = struct.unpack("<I", s[:4])
        self.assertEqual(n, 8)
        self.assertEqual(len(s) % 4, 0)
        self.assertEqual(s[4:12], b"wl_seat\0")


if __name__ == "__main__":
    unittest.main()
```

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_vdrag`
Expected: FAIL, because `vdrag.py` does not exist yet.

- [ ] **Step 2: Write `vdrag.py`**

```python
#!/usr/bin/env python3
"""Scripted interactive drag for nested niri: holds Mod and the left button through
zwp_virtual_keyboard_v1 and zwlr_virtual_pointer_v1, moves at a fixed cadence, holds,
releases (docs/specs/2026-10-01-drag-follow-lag-design.md §7). Stdlib only."""

import argparse
import os
import socket
import struct
import subprocess
import time

BTN_LEFT = 0x110
KEY_LEFTALT = 56
ALT_MASK = 8  # Mod1 in the standard keymap: niri's nested Mod is Alt.


def u32(v):
    return struct.pack("<I", v & 0xFFFFFFFF)


def fixed(v):
    return struct.pack("<i", int(round(v * 256)))


def string(s):
    b = s.encode() + b"\0"
    return u32(len(b)) + b + b"\0" * (-len(b) % 4)


def message(obj, opcode, *args):
    body = b"".join(args)
    return struct.pack("<II", obj, ((8 + len(body)) << 16) | opcode) + body


class Client:
    def __init__(self, path):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)
        self.next_id = 2  # 1 is wl_display
        self.buf = b""

    def new_id(self):
        i = self.next_id
        self.next_id += 1
        return i

    def send(self, data, fds=()):
        if fds:
            socket.send_fds(self.sock, [data], list(fds))
        else:
            self.sock.sendall(data)

    def roundtrip(self):
        """wl_display.sync, then read events until its wl_callback.done; returns them."""
        cb = self.new_id()
        self.send(message(1, 0, u32(cb)))
        events = []
        while True:
            while len(self.buf) < 8:
                chunk = self.sock.recv(65536)
                if not chunk:
                    raise SystemExit("compositor closed the connection")
                self.buf += chunk
            obj, word = struct.unpack("<II", self.buf[:8])
            size = word >> 16
            while len(self.buf) < size:
                self.buf += self.sock.recv(65536)
            msg, self.buf = self.buf[:size], self.buf[size:]
            if obj == 1 and (word & 0xFFFF) == 0:  # wl_display.error
                oid, code = struct.unpack("<II", msg[8:16])
                raise SystemExit(f"protocol error on object {oid}, code {code}")
            if obj == cb:
                return events
            events.append((obj, word & 0xFFFF, msg[8:]))


def globals_of(client):
    registry = client.new_id()
    client.send(message(1, 1, u32(registry)))
    found = {}
    for obj, op, body in client.roundtrip():
        if obj == registry and op == 0:
            name = struct.unpack("<I", body[:4])[0]
            n = struct.unpack("<I", body[4:8])[0]
            iface = body[8 : 8 + n - 1].decode()
            found[iface] = name
    return registry, found


def bind(client, registry, name, iface, version):
    new = client.new_id()
    client.send(message(registry, 0, u32(name), string(iface), u32(version), u32(new)))
    return new


def keymap_fd():
    text = subprocess.run(
        ["xkbcli", "compile-keymap", "--layout", "us"], check=True, capture_output=True
    ).stdout + b"\0"
    fd = os.memfd_create("vdrag-keymap")
    os.write(fd, text)
    return fd, len(text)


def drag(args):
    path = os.path.join(os.environ["XDG_RUNTIME_DIR"], os.environ["WAYLAND_DISPLAY"])
    c = Client(path)
    registry, g = globals_of(c)
    for need in ("wl_seat", "zwlr_virtual_pointer_manager_v1", "zwp_virtual_keyboard_manager_v1"):
        if need not in g:
            raise SystemExit(f"compositor lacks {need}")
    seat = bind(c, registry, g["wl_seat"], "wl_seat", 1)
    pmgr = bind(c, registry, g["zwlr_virtual_pointer_manager_v1"], "zwlr_virtual_pointer_manager_v1", 1)
    kmgr = bind(c, registry, g["zwp_virtual_keyboard_manager_v1"], "zwp_virtual_keyboard_manager_v1", 1)
    ptr = c.new_id()
    c.send(message(pmgr, 0, u32(seat), u32(ptr)))
    kbd = c.new_id()
    c.send(message(kmgr, 0, u32(seat), u32(kbd)))
    fd, size = keymap_fd()
    c.send(message(kbd, 0, u32(1), u32(size)), fds=[fd])  # format 1: xkb_v1; fd travels out of band
    c.roundtrip()

    t0 = time.monotonic()

    def ms():
        return int((time.monotonic() - t0) * 1000)

    x, y, w, h = (int(float(v)) for v in args.to.split(","))
    c.send(message(ptr, 1, u32(ms()), u32(x), u32(y), u32(w), u32(h)))  # motion_absolute
    c.send(message(ptr, 4))
    c.roundtrip()

    c.send(message(kbd, 1, u32(ms()), u32(KEY_LEFTALT), u32(1)))
    c.send(message(kbd, 2, u32(args.mod_mask), u32(0), u32(0), u32(0)))
    c.send(message(ptr, 2, u32(ms()), u32(BTN_LEFT), u32(1)))
    c.send(message(ptr, 4))
    c.roundtrip()
    period = 1.0 / args.hz
    for _ in range(args.frames):
        c.send(message(ptr, 0, u32(ms()), fixed(args.dx), fixed(args.dy)))
        c.send(message(ptr, 4))
        c.roundtrip()
        time.sleep(period)
    time.sleep(args.hold_ms / 1000)
    c.send(message(ptr, 2, u32(ms()), u32(BTN_LEFT), u32(0)))
    c.send(message(ptr, 4))
    c.send(message(kbd, 2, u32(0), u32(0), u32(0), u32(0)))
    c.send(message(kbd, 1, u32(ms()), u32(KEY_LEFTALT), u32(0)))
    c.roundtrip()


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    d = sub.add_parser("drag")
    d.add_argument("--dx", type=float, required=True)
    d.add_argument("--dy", type=float, default=0.0)
    d.add_argument("--frames", type=int, required=True)
    d.add_argument("--hz", type=float, default=125.0)
    d.add_argument("--hold-ms", type=int, default=600)
    d.add_argument("--mod-mask", type=int, default=ALT_MASK)
    d.add_argument("--to", required=True, help="X,Y,W,H: press point and output size, logical px")
    args = ap.parse_args()
    drag(args)


if __name__ == "__main__":
    main()
```

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_vdrag`
Expected: PASS.

- [ ] **Step 3: Write `drag-lag-clips.sh`**

Copy `docs/materials/scripts/ring-motion-clips.sh` to
`docs/materials/scripts/drag-lag-clips.sh`, then make these edits:

1. Replace the header comment with one that describes these sequences:
   - `scroll-fast`: two tiled panes. Drag the left pane right at 20 px per 8 ms event
     (2500 px/s) for 24 events, hold 600 ms, release.
   - `scroll-slow`: the same at 5 px per event (625 px/s) for 60 events.
   - `float-fast` and `float-slow`: the same two speeds on a floating window, which
     drops in place.
   - `native`: `move-column-right`, the control.

   Each sequence records one burst of `RUN_S` (default 3 s) that starts just before
   the press.
2. Keep the capture-protocol, nested-instance, `shot*`, `settle` and burst functions
   unchanged. Delete `corner_crops`, `head_corner_crop`, the scratch-build block and the
   beam sequences.
3. Replace `write_config` with a fixture that pins the glass and the nested Mod:

```bash
write_config() {   # $1 = path; FLOAT=1 makes kitty float
    cat > "$1" <<EOF
material "tg" { glass { bevel 12; thickness 20; jelly-flex 0.0066; jelly-ripple 0; }; }
window-rule { match app-id="^kitty$"; material "tg"; ${FLOAT:+open-floating true;} }
input { mod-key-nested "Alt"; }
layout { focus-ring { off; }; gaps 24; }
hotkey-overlay { skip-at-startup; }
signal { idle-after-ms 0; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" $KITTY_OPTS "--hold" "true"
EOF
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; fail "$1 does not validate"; }
}
write_config "$OUT/tiled.kdl"
FLOAT=1 write_config "$OUT/floating.kdl"
```

4. Add the drag sequences and the drag-start check the spec's pilot needs (§7):

```bash
VDRAG="python3 $ROOT/docs/materials/scripts/vdrag.py"
window_x() { msg -j windows | jq -r --argjson id "$1" '.[] | select(.id == $id) | .layout.tile_pos_in_workspace_view[0] // empty'; }
press_point() {   # $1 = id: "X,Y,W,H" at the window's centre on its output
    local out
    out=$(msg -j outputs | jq -r '[.[]][0].logical | "\(.width),\(.height)"')
    msg -j windows | jq -r --argjson id "$1" --arg out "$out" '.[] | select(.id == $id) | .layout as $l
        | "\($l.tile_pos_in_workspace_view[0] + $l.tile_size[0] / 2),\($l.tile_pos_in_workspace_view[1] + $l.tile_size[1] / 2),\($out)"'
}
drag_sequence() {   # $1 = label, $2 = config, $3 = px per event, $4 = events, $5 = floating 0|1
    local ids before after
    start_nested "$2" "$1"
    spawn_kitty_to 2
    ids=($(kitty_ids)); msg action focus-window --id "${ids[0]}"; settle
    before=$(window_x "${ids[0]}")
    [ -n "$before" ] || fail "$1: no tile_pos_in_workspace_view for ${ids[0]}"
    shot "$OUT/$1-rest.png"
    burst_start "$1" "$RUN_S"
    sleep 0.2
    WAYLAND_DISPLAY=$(basename "$NESTED_WAYLAND") $VDRAG drag --to "$(press_point "${ids[0]}")" --dx "$3" --frames "$4" --hz 125 --hold-ms 600
    burst_wait "$1"
    after=$(window_x "${ids[0]}")
    [ "$before" != "$after" ] || fail "$1: the drag did not move the window (Mod not seen by niri bindings? see spec §7 fallback)"
    echo "$1: window x $before -> $after" | tee -a "$OUT/clips.txt"
    stop_nested
}
seq_native() {
    local ids
    start_nested "$OUT/tiled.kdl" native
    spawn_kitty_to 2
    ids=($(kitty_ids)); msg action focus-window --id "${ids[0]}"; settle
    shot "$OUT/native-rest.png"
    burst_start native "$RUN_S"; sleep 0.2
    msg action move-column-right
    burst_wait native
    stop_nested
}
SEQUENCES=${SEQUENCES:-"scroll-fast scroll-slow float-fast float-slow native"}
for s in $SEQUENCES; do
    case $s in
        scroll-fast) drag_sequence scroll-fast "$OUT/tiled.kdl" 20 24 0 ;;
        scroll-slow) drag_sequence scroll-slow "$OUT/tiled.kdl" 5 60 0 ;;
        float-fast)  drag_sequence float-fast "$OUT/floating.kdl" 20 24 1 ;;
        float-slow)  drag_sequence float-slow "$OUT/floating.kdl" 5 60 1 ;;
        native)      seq_native ;;
        *) fail "unknown sequence $s" ;;
    esac
done
echo "clips: OK; $OUT/clips.txt, capture record $OUT/capture.json"
```

`start_nested` in the copied script sets `NIRI_SOCKET`. Add one line there that exports
the nested instance's Wayland socket path as `NESTED_WAYLAND`. It is the
`WAYLAND_DISPLAY` niri reports in its startup log line "listening on Wayland socket",
which the copied `start_nested` already waits on. `tile_pos_in_workspace_view` and
`tile_size` are fields of the window layout in `niri-ipc`. On a single-output nested
instance, workspace-view coordinates are output coordinates, which is what
`motion_absolute` takes.

Run `bash -n docs/materials/scripts/drag-lag-clips.sh` and
`shellcheck docs/materials/scripts/drag-lag-clips.sh` if shellcheck is installed.
Commit the scripts and the test:

```bash
git add docs/materials/scripts/vdrag.py docs/materials/scripts/drag-lag-clips.sh tools/test_vdrag.py
git commit -m "test(material): drag-lag clip driver and fixture (material-4354cf)"
```

- [ ] **Step 4: Pilot one sequence**

The host must be idle and the run must go on the headless weston host, never the
desktop session. Run:

```bash
CAPTURE_TASK=material-55f8a0 SEQUENCES=scroll-fast docs/materials/scripts/drag-lag-clips.sh
```

Read `clips.txt` and `scroll-fast.gif`. The pilot passes when the window moved (the
script fails otherwise), the burst has frames, and the frames show the glass flexing
during the drag. If the window did not move, Mod did not reach niri's bindings. Stop,
record a `run:` note, and park the task with `--reason decision`, naming the spec §7
fallback (a minimal client that calls `xdg_toplevel.move`). That fallback is not part
of this plan. If the preflight refuses on host load, park with `--reason quiet
--waiting-on user --minutes 8` and list the phases: build 3, preflight 0.5, pilot 1,
full 3.

Every attempt ends with a `tasks note material-55f8a0 "run: …"` line in the form the tasks
skill gives.

- [ ] **Step 5: Full run and the brief**

Run without `SEQUENCES`. Then:

1. Add to "Drag baseline finding" in `docs/notes/2026-09-29-material-dynamics-brief.md`
   a "Follow-lag" column holding the Task 2 trace peaks (drag, hold-settle ms, release)
   from `just test-one -p niri drag_dynamics -- --nocapture`. Keep the baseline column.
2. Add a "Follow-lag clips" subsection: the run directory, the five clips, and a
   `capture.json` reference.
3. Publish the five GIFs and contact sheets on a review page, the same way the
   `material-8e3b73` clips were published. Put its link in the subsection.
4. Change the spec's status line to: "accepted (spec round 6); implemented in
   material-4354cf, clips awaiting owner judgment."

```bash
git add docs/notes/2026-09-29-material-dynamics-brief.md docs/specs/2026-10-01-drag-follow-lag-design.md
git commit -m "docs(material): drag follow-lag traces and clips (material-4354cf)"
```

- [ ] **Step 6: Owner judgment**

Park `material-4354cf` with `--waiting-on user --reason review`. The next step is for
the owner to judge the five clips against the native control: does the drag read as
glass responding to the hand, and does release read as one motion? On acceptance, the
agent closes the Task 4 step and the parent in one commit and merges the branch into
`materials-26.04`. If the owner rejects a §4 decision, the agent amends the spec first.
```
