# Drag follow-lag implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** revised after plan review rounds 1 (revise; P1 4, P2 2, P3 1) and 2
(revise; P1 1, P2 4); awaiting round 3.

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
3. **A window closed in the middle of a drag, or just after release.** The unmap
   snapshot receives the motion residual with the lag included, through the same
   selection functions `store_unmap_snapshot` uses. Task 2,
   `moving_tile_snapshot_residual_includes_lag` and
   `placed_tile_snapshot_residual_includes_lag`.
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

        // An overdamped anchor with velocity reads back exactly, before and at t₀.
        f.shift(ms(40), p(-12., 3.));
        let (al, av) = (f.lag(ms(40)), f.velocity(ms(40)));
        assert!(av.x != 0.);
        assert_eq!(f.lag(ms(20)), al);
        assert_eq!(f.velocity(ms(20)), av);
        assert_eq!(f.velocity(ms(40)), av);

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
        // At or before the anchor: the stored state exactly, never a formula at 0.
        if now <= self.t0 {
            return (self.x, self.y);
        }
        let dt = (now - self.t0).as_secs_f64();
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
  - `#[cfg(test)] Tile::clear_drag_follower(&mut self)`, for paired runs without the
    follower.
  - `InteractiveMoveData::unmap_snapshot_motion_residual(&self) -> Point<f64, Logical>`,
    the residual the moving tile's unmap snapshot receives.

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

/// One release run: lift, drag `steps` (each a per-frame pointer step), release in
/// the frame of the last step, then 40 frames. With `follow == false` the moving
/// tile's follower is cleared after every update, so the run is the pre-follower
/// behaviour with identical layout ops. Returns per-frame (render offset, tile
/// position in the workspace, motion residual, animation residual), the first row
/// read at the release instant.
fn release_run(steps: &[Point<f64, Logical>], follow: bool) -> Vec<(Point<f64, Logical>, Point<f64, Logical>, Point<f64, Logical>, Point<f64, Logical>)> {
    let mut layout = two_columns();
    let mut pointer = Point::from((300., 100.));
    lifted(&mut layout, 1, &mut pointer);
    for step in steps {
        pointer += *step;
        Op::InteractiveMoveUpdate { window: 1, dx: step.x, dy: step.y, output_idx: 1, px: pointer.x, py: pointer.y }.apply(&mut layout);
        if !follow {
            let Some(InteractiveMoveState::Moving(move_)) = &mut layout.interactive_move else { unreachable!() };
            move_.tile.clear_drag_follower();
        }
        Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(&mut layout);
    }
    end(&mut layout, 1);
    let mut rows = Vec::new();
    for frame in 0..=40 {
        if frame > 0 {
            Op::AdvanceAnimations { msec_delta: FRAME_MS }.apply(&mut layout);
        }
        let (_, _, ws) = layout.workspaces().find(|(_, _, ws)| ws.has_window(&1)).unwrap();
        let (t, pos, _) = ws.tiles_with_render_positions().find(|(t, _, _)| *t.window().id() == 1).unwrap();
        rows.push((t.render_offset(), pos, t.motion_residual(), t.animation_residual()));
    }
    rows
}

/// The pinned opposing fixture (§4.4): a drag right, then a sharp reversal left,
/// released moving left. Pin `OPPOSING` from the probe in Step 5b.
const OPPOSING: &[(f64, f64)] = &[(40., 0.), (40., 0.), (40., 0.), (40., 0.), (-40., 0.), (-40., 0.)];

#[test]
fn opposing_release_lowers_flex_and_keeps_the_trajectory() {
    let steps: Vec<_> = OPPOSING.iter().map(|&(x, y)| Point::from((x, y))).collect();
    let with = release_run(&steps, true);
    let without = release_run(&steps, false);
    // The fixture is opposing: lag against the release term, smaller than it.
    let (_, _, motion, release) = with[0];
    let lag = motion - release;
    let dot = lag.x * release.x + lag.y * release.y;
    assert!(dot < 0., "fixture no longer opposing: lag {lag:?} release {release:?}");
    assert!(lag.x.hypot(lag.y) < release.x.hypot(release.y), "lag {lag:?} reverses, not lowers");
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
    // release term. Pin `ALIGNED` from the same probe.
    const ALIGNED: &[(f64, f64)] = &[(40., 0.), (40., 0.), (40., 0.), (40., 0.)];
    let steps: Vec<_> = ALIGNED.iter().map(|&(x, y)| Point::from((x, y))).collect();
    let with = release_run(&steps, true);
    let without = release_run(&steps, false);
    let (_, _, motion, release) = with[0];
    let lag = motion - release;
    assert!(lag.x * release.x + lag.y * release.y > 0., "fixture no longer aligned: lag {lag:?} release {release:?}");
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
    Op::InteractiveMoveUpdate { window: 1, dx: step.x, dy: step.y, output_idx: 1, px: pointer.x + step.x, py: pointer.y }.apply(&mut layout);
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
    let Some(InteractiveMoveState::Moving(move_)) = &layout.interactive_move else { unreachable!() };
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
        layout.workspaces().find_map(|(_, _, ws)| ws.unmap_snapshot_motion_residual(&1)).unwrap()
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
    assert!((with_lag.x - without.x - lag.x).abs() < 1e-9 && (with_lag.y - without.y - lag.y).abs() < 1e-9);
}
```

Before running, check `src/layout/tests.rs` and `src/layout/workspace.rs` for the exact
names: `Op::ToggleOverview`, `Workspace::has_window`,
`Workspace::tiles_with_render_positions` (it yields `(&Tile, Point, visible)`),
`workspaces_mut` and `tiles_mut`. The tests module is a child of `layout`, so it can reach `pub(super)`
items.

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

    #[cfg(test)]
    pub(super) fn clear_drag_follower(&mut self) {
        self.drag_follower = None;
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

In `src/layout/mod.rs`, add to `impl<W: LayoutElement> InteractiveMoveData<W>` (next to
`tile_render_location`):

```rust
    /// The motion residual the moving tile's unmap snapshot receives.
    fn unmap_snapshot_motion_residual(&self) -> Point<f64, Logical> {
        self.tile.motion_residual()
    }
```

In the moving-tile branch of `Layout::store_unmap_snapshot` (~4749), pass
`move_.unmap_snapshot_motion_residual()` in place of `move_.tile.animation_residual()`.
Compute it into a local before `move_.tile.store_unmap_snapshot_if_empty(..)`, because
that call borrows `move_.tile` mutably.

Then replace `.animation_residual()` with `.motion_residual()` at the remaining five
render and snapshot call sites: `src/layout/mod.rs` (two), `src/layout/scrolling.rs` (two),
`src/layout/floating.rs` (one), `src/layout/workspace.rs` (one): the moving tile's
render at ~4908, the two in `scrolling.rs`, and the one each in `floating.rs` and
`workspace.rs`. Find them with `grep -n 'animation_residual()' src/layout`. Leave the one in
`src/layout/tests.rs:2017` (`scrolling_unmap_snapshot_keeps_view_animation_residual`)
as it is: it pins the move-animation part.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `just test-one -p niri drag_dynamics`, then `just test-one -p niri tile`.
Expected: all PASS. `-- --nocapture` prints the traces. Record the new drag peak and
the hold settle time from the scrolling and floating traces. Task 4 puts them in the
brief.

If `opposing_release_projects_below_the_release_term` finds no opposing case, change
the drag directions or lengths until it does. The asserted relation must not change.

- [ ] **Step 5b: Probe and pin the release fixtures**

This step runs once Steps 3–4 have wired `motion_residual`, `clear_drag_follower` and
the follower into the layout, because `release_run` needs them. Until then, the two
release tests may fail on their fixture-shape assertions and nothing else. The opposing
and aligned fixtures depend on where the scrolling layout puts the drop, so pin them
from a probe rather than trusting the candidates written in Step 1. Temporarily add an
`#[ignore]` test that runs `release_run` with `follow == true` for the candidates:
right ×4 then left ×2, left ×4, and right ×4. Have it print `lag`, `release` and their
dot product. Run it with `just test-one -p niri drag_dynamics -- --ignored --nocapture`.
Choose an `OPPOSING` sequence with `dot < 0` and `|lag| < |release|`, and an `ALIGNED`
sequence with `dot > 0`. Lengthen or shorten the reversal until both hold. Write the
chosen sequences into the two constants, then delete the probe test. Rerun
`just test-one -p niri drag_dynamics`: both release tests must now pass and assert
unconditionally.

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

The virtual keyboard cannot set niri's grab modifier. The pinned Smithay forwards
virtual-keyboard modifiers to clients without updating the seat state that niri checks
for Mod+LMB. The driver therefore takes the spec's §7 move-client path. One stdlib
client maps its own window (app-id `vdrag`), walks a `zwlr_virtual_pointer_v1` across
the output until it enters that window, and presses the button. On the
`wl_pointer.button` it receives, it calls `xdg_toplevel.move`, which starts niri's
interactive move with no modifier. It then sends the scripted motion, hold and release.
The client knows its own window, so it needs no window coordinates over IPC.
`tile_pos_in_workspace_view` covers floating windows only.

**Files:**
- Create: `docs/materials/scripts/vdrag.py`, a stdlib Wayland client: `wl_shm` window,
  `xdg_shell` and a virtual pointer. It lifts the window vertically before the timed
  horizontal segment.
- Create: `docs/materials/scripts/drag-lag-clips.sh`, written in full below. It does
  not copy `ring-motion-clips.sh`.
- Create: `tools/test_vdrag.py`, tests for wire encoding and event decoding.
- Modify: `docs/notes/2026-09-29-material-dynamics-brief.md` ("Drag baseline finding":
  a follow-lag column and a clip section)
- Modify: `docs/specs/2026-10-01-drag-follow-lag-design.md`: the status line, and §7's
  sentence about the driver, which now records that the move client is the primary
  path.

**Interfaces:**
- Consumes: the built niri with Tasks 1–3.
- Produces:
  - `vdrag.py --socket PATH --extent W,H --dx PX --dy PX --frames N --hz HZ
    --hold-ms MS --lift-ms MS --ready FILE --go FILE --done FILE`. It maps a window
    and finds it with the pointer, then writes `--ready`. It waits for `--go` to exist,
    presses, starts the move, lifts 260 px vertically and pauses `--lift-ms`. It then
    runs the timed segment, holds, releases and writes `--done`. It stays mapped until
    SIGTERM.
  - `drag-lag-clips.sh` with `SEQUENCES` drawn from `scroll-fast scroll-slow float-fast
    float-slow native`.

- [ ] **Step 1: Write the failing tests**

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

    def test_split_messages_handles_partial_tail(self):
        a = vdrag.message(3, 0, vdrag.u32(5))
        b = vdrag.message(4, 1, vdrag.string("x"))
        msgs, rest = vdrag.split_messages(a + b[:6])
        self.assertEqual([(m[0], m[1]) for m in msgs], [(3, 0)])
        self.assertEqual(rest, b[:6])

    def test_parse_global(self):
        body = vdrag.u32(9) + vdrag.string("wl_seat") + vdrag.u32(7)
        self.assertEqual(vdrag.parse_global(body), (9, "wl_seat", 7))

    def test_stripes_fill_the_buffer_translucent_and_premultiplied(self):
        data = vdrag.stripes(64, 4)
        self.assertEqual(len(data), 64 * 4 * 4)
        for (px,) in struct.iter_unpack("<I", data):
            a = px >> 24
            self.assertLess(a, 255)
            for shift in (0, 8, 16):
                self.assertLessEqual((px >> shift) & 0xFF, a)


if __name__ == "__main__":
    unittest.main()
```

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_vdrag`
Expected: FAIL, because `vdrag.py` does not exist yet.

- [ ] **Step 2: Write `vdrag.py`**

```python
#!/usr/bin/env python3
"""Scripted interactive drag for nested niri (docs/specs/2026-10-01-drag-follow-lag-design.md
§7). Maps a striped wl_shm window (app-id vdrag), finds it with a zwlr_virtual_pointer_v1,
presses the left button on it and starts the move itself with xdg_toplevel.move, then
moves at a fixed cadence, holds and releases. Stdlib only; exits non-zero on any
protocol error or timeout."""

import argparse
import mmap
import os
import pathlib
import select
import signal
import socket
import struct
import sys
import time

BTN_LEFT = 0x110
ARGB8888 = 0


def u32(v):
    return struct.pack("<I", v & 0xFFFFFFFF)


def i32(v):
    return struct.pack("<i", v)


def fixed(v):
    return struct.pack("<i", int(round(v * 256)))


def string(s):
    b = s.encode() + b"\0"
    return u32(len(b)) + b + b"\0" * (-len(b) % 4)


def message(obj, opcode, *args):
    body = b"".join(args)
    return struct.pack("<II", obj, ((8 + len(body)) << 16) | opcode) + body


def split_messages(buf):
    """Complete (obj, opcode, body) messages from buf, and the incomplete tail."""
    out = []
    while len(buf) >= 8:
        obj, word = struct.unpack("<II", buf[:8])
        size = word >> 16
        if len(buf) < size:
            break
        out.append((obj, word & 0xFFFF, buf[8:size]))
        buf = buf[size:]
    return out, buf


def parse_global(body):
    name, n = struct.unpack("<II", body[:8])
    iface = body[8 : 8 + n - 1].decode()
    off = 8 + n + (-n % 4)
    (version,) = struct.unpack("<I", body[off : off + 4])
    return name, iface, version


def stripes(width, height):
    """Premultiplied ARGB8888 rows: translucent 32 px stripes at alpha 0.6. Opaque
    pixels skip the material shader and would show glass only on the bevel."""
    a = struct.pack("<I", 0x998B8B8B)  # (0xE8, 0xE8, 0xE8) × 0.6
    b = struct.pack("<I", 0x9919386A)  # (0x2A, 0x5D, 0xB0) × 0.6
    row = b"".join((a if (x // 32) % 2 == 0 else b) for x in range(width))
    return row * height


class Conn:
    def __init__(self, path):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)
        self.next_id = 2
        self.handlers = {1: self.on_display}
        self.buf = b""

    def new_id(self, handler=None):
        i = self.next_id
        self.next_id += 1
        if handler:
            self.handlers[i] = handler
        return i

    def send(self, obj, opcode, *args, fds=()):
        data = message(obj, opcode, *args)
        if fds:
            socket.send_fds(self.sock, [data], list(fds))
        else:
            self.sock.sendall(data)

    def on_display(self, op, body):
        if op == 0:
            oid, code = struct.unpack("<II", body[:8])
            sys.exit(f"vdrag: protocol error on object {oid}, code {code}")

    def dispatch(self, timeout):
        r, _, _ = select.select([self.sock], [], [], timeout)
        if not r:
            return
        chunk = self.sock.recv(65536)
        if not chunk:
            sys.exit("vdrag: compositor closed the connection")
        msgs, self.buf = split_messages(self.buf + chunk)
        for obj, op, body in msgs:
            h = self.handlers.get(obj)
            if h:
                h(op, body)

    def roundtrip(self, limit=5.0):
        done = []
        cb = self.new_id(lambda op, body: done.append(1))
        self.send(1, 0, u32(cb))
        end = time.monotonic() + limit
        while not done:
            if time.monotonic() > end:
                sys.exit("vdrag: roundtrip timed out")
            self.dispatch(0.1)


class Window:
    def __init__(self, c, globals_):
        self.c = c
        self.size = (640, 480)
        self.pending = None
        self.configured = False
        self.entered = False
        self.walk_y = 0
        self.button_serial = None

        def bind(iface, version, handler=None):
            name, _ = globals_[iface]
            new = c.new_id(handler)
            c.send(self.registry, 0, u32(name), string(iface), u32(version), u32(new))
            return new

        self.registry = globals_["__registry__"]
        self.compositor = bind("wl_compositor", 4)
        self.shm = bind("wl_shm", 1)
        self.wm = bind("xdg_wm_base", 1, self.on_wm)
        self.seat = bind("wl_seat", 5)
        self.vpm = bind("zwlr_virtual_pointer_manager_v1", 1)
        self.surface = c.new_id()
        c.send(self.compositor, 0, u32(self.surface))
        self.xdg_surface = c.new_id(self.on_xdg_surface)
        c.send(self.wm, 2, u32(self.xdg_surface), u32(self.surface))
        self.toplevel = c.new_id(self.on_toplevel)
        c.send(self.xdg_surface, 1, u32(self.toplevel))
        c.send(self.toplevel, 2, string("vdrag"))  # set_title
        c.send(self.toplevel, 3, string("vdrag"))  # set_app_id
        self.pointer = c.new_id(self.on_pointer)
        c.send(self.seat, 0, u32(self.pointer))  # get_pointer
        self.vp = c.new_id()
        c.send(self.vpm, 0, u32(self.seat), u32(self.vp))  # create_virtual_pointer
        c.send(self.surface, 6)  # initial commit, no buffer

    def on_wm(self, op, body):
        if op == 0:  # ping
            self.c.send(self.wm, 3, body[:4])

    def on_toplevel(self, op, body):
        if op == 0:  # configure(width, height, states)
            w, h = struct.unpack("<ii", body[:8])
            if w > 0 and h > 0:
                self.pending = (w, h)
        elif op == 1:
            sys.exit("vdrag: closed by the compositor")

    def on_xdg_surface(self, op, body):
        if op == 0:  # configure(serial)
            (serial,) = struct.unpack("<I", body[:4])
            if self.pending:
                self.size, self.pending = self.pending, None
            self.c.send(self.xdg_surface, 4, u32(serial))  # ack_configure
            self.draw()
            self.configured = True

    def on_pointer(self, op, body):
        if op == 0:  # enter(serial, surface, x, y)
            if struct.unpack("<I", body[4:8])[0] == self.surface:
                self.entered = True
        elif op == 3:  # button(serial, time, button, state)
            serial, _, button, state = struct.unpack("<IIII", body[:16])
            if button == BTN_LEFT and state == 1:
                self.button_serial = serial

    def draw(self):
        w, h = self.size
        stride = w * 4
        size = stride * h
        fd = os.memfd_create("vdrag-shm")
        os.ftruncate(fd, size)
        with mmap.mmap(fd, size) as m:
            m.write(stripes(w, h))
        pool = self.c.new_id()
        self.c.send(self.shm, 0, u32(pool), u32(size), fds=[fd])  # create_pool(id, fd, size)
        os.close(fd)
        buf = self.c.new_id()
        self.c.send(pool, 0, u32(buf), i32(0), i32(w), i32(h), i32(stride), u32(ARGB8888))
        self.c.send(pool, 1)  # destroy pool; the buffer keeps the memory
        self.c.send(self.surface, 1, u32(buf), i32(0), i32(0))  # attach
        self.c.send(self.surface, 2, i32(0), i32(0), i32(w), i32(h))  # damage
        self.c.send(self.surface, 6)  # commit


def win_entered_y(win):
    """The pointer's output y when the walk entered the window."""
    return win.walk_y


def wait_for(c, pred, secs, what):
    end = time.monotonic() + secs
    while not pred():
        if time.monotonic() > end:
            sys.exit(f"vdrag: timed out waiting for {what}")
        c.dispatch(0.02)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--socket", required=True, help="absolute path of the nested niri socket")
    ap.add_argument("--extent", required=True, help="W,H of the output, logical px")
    ap.add_argument("--dx", type=float, required=True)
    ap.add_argument("--dy", type=float, default=0.0)
    ap.add_argument("--frames", type=int, required=True)
    ap.add_argument("--hz", type=float, default=125.0)
    ap.add_argument("--hold-ms", type=int, default=600)
    ap.add_argument("--lift-ms", type=int, default=500, help="pause after the vertical lift")
    ap.add_argument("--ready", required=True)
    ap.add_argument("--go", required=True)
    ap.add_argument("--done", required=True)
    args = ap.parse_args()
    signal.signal(signal.SIGTERM, lambda *_: sys.exit(0))
    w_ext, h_ext = (int(v) for v in args.extent.split(","))

    c = Conn(args.socket)
    found = {}
    registry = c.new_id(
        lambda op, body: op == 0 and found.__setitem__(parse_global(body)[1], parse_global(body)[0::2])
    )
    c.send(1, 1, u32(registry))  # get_registry
    c.roundtrip()
    for need in ("wl_compositor", "wl_shm", "xdg_wm_base", "wl_seat", "zwlr_virtual_pointer_manager_v1"):
        if need not in found:
            sys.exit(f"vdrag: compositor lacks {need}")
    found["__registry__"] = registry
    win = Window(c, found)
    wait_for(c, lambda: win.configured, 5, "first configure")
    c.roundtrip()
    time.sleep(0.5)  # open animation

    t0 = time.monotonic()

    def ms():
        return int((time.monotonic() - t0) * 1000)

    # Walk the pointer across three rows until it enters our window.
    for y in (h_ext // 2, h_ext // 3, 2 * h_ext // 3):
        for x in range(8, w_ext, 16):
            win.walk_y = y
            c.send(win.vp, 1, u32(ms()), u32(x), u32(y), u32(w_ext), u32(h_ext))  # motion_absolute
            c.send(win.vp, 4)  # frame
            c.roundtrip()
            if win.entered:
                break
        if win.entered:
            break
    if not win.entered:
        sys.exit("vdrag: pointer never entered the window")
    pathlib.Path(args.ready).touch()
    wait_for(c, lambda: os.path.exists(args.go), 60, "go file")

    c.send(win.vp, 2, u32(ms()), u32(BTN_LEFT), u32(1))  # button press
    c.send(win.vp, 4)
    wait_for(c, lambda: win.button_serial is not None, 2, "button event")
    c.send(win.toplevel, 5, u32(win.seat), u32(win.button_serial))  # xdg_toplevel.move
    c.roundtrip()
    period = 1.0 / args.hz
    # Lift vertically first. The grab decides between a window move and a view pan
    # after 8 px: mostly horizontal motion on a tiled window pans the view, which never
    # reaches the follower. Vertical motion selects the move, and 260 px crosses the
    # 256 px lift threshold. Then a pause lets the lift's own flex settle before the
    # timed segment.
    lift_dy = -10.0 if win_entered_y(win) > h_ext // 3 else 10.0
    for _ in range(26):
        c.send(win.vp, 0, u32(ms()), fixed(0.0), fixed(lift_dy))
        c.send(win.vp, 4)
        end = time.monotonic() + period
        while time.monotonic() < end:
            c.dispatch(max(0.0, end - time.monotonic()))
    end = time.monotonic() + args.lift_ms / 1000
    while time.monotonic() < end:
        c.dispatch(max(0.0, end - time.monotonic()))
    for _ in range(args.frames):
        c.send(win.vp, 0, u32(ms()), fixed(args.dx), fixed(args.dy))  # motion
        c.send(win.vp, 4)
        end = time.monotonic() + period
        while time.monotonic() < end:
            c.dispatch(max(0.0, end - time.monotonic()))
    end = time.monotonic() + args.hold_ms / 1000
    while time.monotonic() < end:
        c.dispatch(max(0.0, end - time.monotonic()))
    c.send(win.vp, 2, u32(ms()), u32(BTN_LEFT), u32(0))  # release
    c.send(win.vp, 4)
    c.roundtrip()
    pathlib.Path(args.done).touch()
    while True:
        c.dispatch(1.0)


if __name__ == "__main__":
    main()
```

Run: `just --set one_cmd 'python3 -m unittest' test-one tools.test_vdrag`
Expected: PASS.

A compositor honours `xdg_toplevel.move` only for a serial from a press that is still
held, so the driver sends `move` as soon as the press event arrives and before any
motion. If niri rejects the request anyway, `niri.log` shows the move handler's
reason. Start the Step 4 pilot investigation there.

- [ ] **Step 3: Write `drag-lag-clips.sh`**

```bash
#!/usr/bin/env bash
# drag-lag-clips.sh: review clips for the drag follow-lag stimulus
# (docs/specs/2026-10-01-drag-follow-lag-design.md §7). Each sequence runs a nested
# headless niri with one kitty and one vdrag client (docs/materials/scripts/vdrag.py).
# vdrag drags its own window through xdg_toplevel.move, driven by a virtual pointer,
# while a burst of screenshots records it, under the capture protocol
# (tools/capture-meta: preflight, identity, settle before every launch, release).
# The clips are for the owner's judgment against the native column move; this script
# records, it does not grade. It does check that the timed segment ran as an
# interactive move (IPC reports no workspace for a window being moved), not a view pan.
#
# Sequences (SEQUENCES env, space-separated, default all):
#   scroll-fast  tiled: drag vdrag left 20 px per 8 ms event (2500 px/s), 24 events,
#                hold 600 ms, release
#   scroll-slow  tiled: 5 px per event (625 px/s), 60 events
#   float-fast   floating vdrag over kitty: right at 20 px per event, 24 events
#   float-slow   floating: 5 px per event, 60 events
#   native       tiled, no drag: move-column-left on vdrag (the control)
#
# Env: NIRI_MATERIAL_WORK_ROOT (artifacts land under it), CAPTURE_TASK (the task id
# authorizing the run), NIRI (default: this checkout's release build, built here).
# Requires: weston, kitty, swaybg, jq, ImageMagick, python3.
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root}
TASK=${CAPTURE_TASK:?task id authorizing this run}
RUN=drag-lag-$$-$(date +%s)
OUT=$EVIDENCE/drag-lag-clips-$(git rev-parse --short HEAD)/$RUN
RT=$XDG_RUNTIME_DIR/$RUN-rt       # short: nested niri panics on long socket paths
SEQUENCES=${SEQUENCES:-"scroll-fast scroll-slow float-fast float-slow native"}
RUN_S=4            # lift 0.7 s, slow segment 0.5 s, hold 0.6 s, release settling under 1 s
GO_DELAY=0.2       # burst start to the go file
LIFT_S=0.71        # 26 lift events at 125 Hz, then --lift-ms 500
VDRAG=$ROOT/docs/materials/scripts/vdrag.py
UNIT=; HOST_SOCKET=; NIRI_PID=; NIRI_SOCKET=; VDRAG_PID=; BURST_PID=; HOST_SEQ=0
mkdir -p "$OUT" "$RT"

capture_meta() { python3 "$ROOT/tools/capture-meta" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
pid_running() { local state; state=$(ps -o stat= -p "$1" 2>/dev/null) || return 1; [[ $state != Z* ]]; }
msg() { "$NIRI" msg "$@"; }
app_ids() { msg -j windows | jq -r '.[].app_id'; }
wait_app() { for _ in $(seq 100); do app_ids | grep -qx "$1" && return; sleep 0.1; done; fail "no $1 window"; }
app_window() { msg -j windows | jq -r --arg a "$1" '.[] | select(.app_id == $a) | .id'; }

stop_nested() {
    # The screenshot worker first: it talks to the niri being stopped.
    if [ -n "$BURST_PID" ]; then
        kill "$BURST_PID" 2>/dev/null || true; wait "$BURST_PID" 2>/dev/null || true; BURST_PID=
    fi
    if [ -n "$VDRAG_PID" ]; then
        kill "$VDRAG_PID" 2>/dev/null || true; wait "$VDRAG_PID" 2>/dev/null || true; VDRAG_PID=
    fi
    if [ -n "$NIRI_PID" ]; then
        if [ -S "$NIRI_SOCKET" ]; then msg action quit --skip-confirmation >/dev/null 2>&1 || true; fi
        for _ in $(seq 50); do pid_running "$NIRI_PID" || break; sleep 0.1; done
        if pid_running "$NIRI_PID"; then kill "$NIRI_PID" 2>/dev/null || true; fi
        wait "$NIRI_PID" 2>/dev/null || true
        NIRI_PID=; NIRI_SOCKET=
    fi
    if [ -n "$UNIT" ]; then
        timeout 10 systemctl --user stop "$UNIT" >/dev/null 2>&1 || true
        for _ in $(seq 50); do [ -e "$HOST_SOCKET" ] || break; sleep 0.1; done
        UNIT=; HOST_SOCKET=
        sleep 4   # weston's GL renderer steps the GPU down over about 2 s
    fi
    rm -rf "${RT:?}"/*
}
cleanup() {
    local rc=$?
    stop_nested
    rm -rf "$RT"
    capture_meta release "$OUT" || true
    exit "$rc"
}
trap cleanup EXIT INT TERM

# --- capture protocol -------------------------------------------------------
capture_meta preflight "$OUT" --lane headless --task "$TASK" --fixture "$(basename "$0")" \
    --owner-pid $$ --tool weston --tool kitty || fail "preflight refused; see $OUT/capture.json"

TARGET=$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
if [ -z "${NIRI:-}" ]; then
    cargo build --release
    NIRI=$TARGET/release/niri
fi
[ -x "$NIRI" ] || fail "niri binary not found at $NIRI"
cp "$NIRI" "$OUT/niri"; NIRI=$OUT/niri
sha256sum "$NIRI" "$VDRAG" | tee -a "$OUT/SHA256SUMS"

# --- configs -----------------------------------------------------------------
CHECKER=$OUT/checker.png
magick -size 160x90 pattern:checkerboard -scale 800% "$CHECKER"
# The glass is pinned (bevel 12, thickness 20, the live jelly-flex 0.0066, ripple
# off so flex is the only motion cue). The idle gate is off: the nested instance
# sees only the virtual pointer.
write_config() {   # $1 = path; FLOAT=1 opens vdrag floating
    cat > "$1" <<KDL
material "tg" { glass { bevel 12; thickness 20; jelly-flex 0.0066; jelly-ripple 0; }; }
window-rule { match app-id="^(kitty|vdrag)$"; material "tg"; }
window-rule { match app-id="^vdrag$"; open-floating ${FLOAT:-false}; }
layout { focus-ring { off; }; gaps 24; }
hotkey-overlay { skip-at-startup; }
signal { idle-after-ms 0; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" "-o" "cursor_blink_interval=0" "-o" "background_opacity=0.6" "--hold" "true"
KDL
    "$NIRI" validate -c "$1" >/dev/null 2>&1 || { "$NIRI" validate -c "$1"; fail "$1 does not validate"; }
}
write_config "$OUT/tiled.kdl"
FLOAT=true write_config "$OUT/floating.kdl"

capture_meta identity "$OUT" --source "$ROOT" --binary "$NIRI" --input "$0" --input "$VDRAG" \
    --input "$OUT/tiled.kdl" --input "$OUT/floating.kdl" || fail "identity refused"

# --- nested instance ---------------------------------------------------------
NESTED_WAYLAND=
start_nested() {   # $1 = config, $2 = sub-run name; sets NIRI_SOCKET and NESTED_WAYLAND
    capture_meta settle "$OUT" --sub-run "$2" --input "$1" || fail "settle refused before $2; see $OUT/capture.json"
    HOST_SEQ=$((HOST_SEQ + 1))
    local host=$RUN-h$HOST_SEQ
    UNIT=$host-weston; HOST_SOCKET=$XDG_RUNTIME_DIR/$host
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$host" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$HOST_SOCKET" ] && break; sleep 0.1; done
    [ -S "$HOST_SOCKET" ] || fail "Weston socket never appeared at $HOST_SOCKET"
    ln -s "$HOST_SOCKET" "$RT/$host"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$host "$NIRI" -c "$1" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1); export NIRI_SOCKET
    # The nested instance's own Wayland socket, by absolute path: the driver must not
    # resolve it against the outer XDG_RUNTIME_DIR.
    NESTED_WAYLAND=
    for f in "$RT"/wayland-*; do [ -S "$f" ] && NESTED_WAYLAND=$f; done
    [ -S "$NESTED_WAYLAND" ] || fail "nested Wayland socket not found under $RT"
    wait_app kitty
}
shot_request() { rm -f "$1"; msg action screenshot-screen --write-to-disk true --show-pointer false --path "$1"; }
shot_wait() { for _ in $(seq 200); do [ -s "$1" ] && magick identify "$1" >/dev/null 2>&1 && return; sleep 0.05; done; fail "shot $1"; }
shot() { shot_request "$1"; shot_wait "$1"; }
settle() {   # two shots 0.6 s apart agree
    local a=$OUT/settle-a.png b=$OUT/settle-b.png n
    for _ in $(seq 20); do
        shot "$a"; sleep 0.6; shot "$b"
        n=$(magick compare -metric AE "$a" "$b" null: 2>&1 | awk '{print $1}' || true)
        [ "${n%.*}" = 0 ] && return
    done
    fail "scene never settled"
}

# --- bursts ------------------------------------------------------------------
BURST_T0=
burst_start() {   # $1 = label, $2 = seconds
    local d=$OUT/$1; mkdir -p "$d"
    BURST_T0=$(date +%s.%N)
    (
        local now i=0 f
        while :; do
            now=$(date +%s.%N)
            awk -v a="$now" -v b="$BURST_T0" -v s="$2" 'BEGIN { exit !(a - b < s) }' || break
            i=$((i + 1)); f=$d/f$(printf %03d "$i").png
            printf '%s %s\n' "$(basename "$f")" "$(awk -v a="$now" -v b="$BURST_T0" 'BEGIN { printf "%.3f", a - b }')" >> "$d/frames.txt"
            shot "$f"
        done
        echo "$1: $i frames in $2 s" >> "$OUT/timing.txt"
    ) &
    BURST_PID=$!
}
burst_wait() {
    wait "$BURST_PID"; BURST_PID=
    local d=$OUT/$1
    [ -n "$(ls "$d"/f*.png 2>/dev/null)" ] || fail "$1: no frames"
    magick -delay 8 -loop 0 "$d"/f*.png -scale 50% "$OUT/$1.gif"
    magick montage "$d"/f*.png -tile 8x -geometry 320x180+2+2 -background '#111' "$OUT/$1-sheet.png"
    echo "$1: $d ($(ls "$d"/f*.png | wc -l) frames); gif $OUT/$1.gif" | tee -a "$OUT/clips.txt"
}

# --- sequences ---------------------------------------------------------------
drag_sequence() {   # $1 = label, $2 = config, $3 = dx per event, $4 = events
    local ready=$RT/$1.ready go=$RT/$1.go done=$RT/$1.done drag_s moving
    start_nested "$2" "$1"
    python3 "$VDRAG" --socket "$NESTED_WAYLAND" --extent 1280,720 --dx "$3" --frames "$4" \
        --hz 125 --hold-ms 600 --ready "$ready" --go "$go" --done "$done" >> "$OUT/vdrag.log" 2>&1 &
    VDRAG_PID=$!
    for _ in $(seq 150); do [ -e "$ready" ] && break; pid_running "$VDRAG_PID" || fail "$1: vdrag exited; see $OUT/vdrag.log"; sleep 0.1; done
    [ -e "$ready" ] || fail "$1: vdrag never found its window"
    settle
    shot "$OUT/$1-rest.png"
    burst_start "$1" "$RUN_S"
    sleep "$GO_DELAY"
    touch "$go"
    # During the timed segment the window must be under interactive move: niri's IPC
    # reports `workspace_id: null` only for the moving window (Layout::with_windows).
    # A view pan keeps it on its workspace, so this rejects the pan a pixel check
    # would accept.
    drag_s=$(awk -v n="$4" 'BEGIN { print n / 125 }')
    sleep "$LIFT_S"
    moving=0
    for _ in $(seq 20); do
        if [ "$(msg -j windows | jq -r '.[] | select(.app_id == "vdrag") | .workspace_id')" = null ]; then
            moving=1; break
        fi
        sleep "$(awk -v d="$drag_s" 'BEGIN { print d / 20 }')"
    done
    burst_wait "$1"
    [ -e "$done" ] || fail "$1: vdrag did not finish the drag; see $OUT/vdrag.log"
    [ "$moving" = 1 ] || fail "$1: the timed segment never ran as an interactive move (a view pan, or the lift failed)"
    echo "$1: interactive move confirmed during the timed segment" | tee -a "$OUT/clips.txt"
    stop_nested
}
seq_native() {
    start_nested "$OUT/tiled.kdl" native
    python3 "$VDRAG" --socket "$NESTED_WAYLAND" --extent 1280,720 --dx 0 --frames 0 \
        --ready "$RT/native.ready" --go "$RT/native.never" --done "$RT/native.done" >> "$OUT/vdrag.log" 2>&1 &
    VDRAG_PID=$!
    for _ in $(seq 150); do [ -e "$RT/native.ready" ] && break; sleep 0.1; done
    msg action focus-window --id "$(app_window vdrag)"; settle
    shot "$OUT/native-rest.png"
    burst_start native "$RUN_S"; sleep "$GO_DELAY"
    msg action move-column-left
    burst_wait native
    stop_nested
}

for s in $SEQUENCES; do
    case $s in
        scroll-fast) drag_sequence scroll-fast "$OUT/tiled.kdl" -20 24 ;;
        scroll-slow) drag_sequence scroll-slow "$OUT/tiled.kdl" -5 60 ;;
        float-fast)  drag_sequence float-fast "$OUT/floating.kdl" 20 24 ;;
        float-slow)  drag_sequence float-slow "$OUT/floating.kdl" 5 60 ;;
        native)      seq_native ;;
        *) fail "unknown sequence $s" ;;
    esac
done
echo "clips: OK; $OUT/clips.txt, capture record $OUT/capture.json"
```

`magick compare` exits 1 when the images differ. Under `set -e` and `pipefail` that
would abort the script, so the `compare` in `settle` ends in `|| true` and reads the
count from its output. The `material-8e3b73` run hit this failure. `stop_nested` stops
the screenshot worker before niri. It runs from the `EXIT`, `INT` and `TERM` trap, so a
failure partway through a burst, or an interrupt, still reaps it.

Check before running:
- `bash -n docs/materials/scripts/drag-lag-clips.sh`.
- `shellcheck` on the script, if it is installed.
- `"$NIRI" validate -c` on both configs, which the script also does. If `open-floating
  false` does not validate, drop that line from the tiled config and keep it as
  `open-floating true` for the floating one.

Commit:

```bash
git add docs/materials/scripts/vdrag.py docs/materials/scripts/drag-lag-clips.sh tools/test_vdrag.py
git commit -m "test(material): drag-lag move client and clip fixture (material-4354cf)"
```

- [ ] **Step 4: Pilot one sequence**

The host must be idle and the run must go on the headless weston host, never the
desktop session. Run:

```bash
CAPTURE_TASK=material-55f8a0 SEQUENCES=scroll-fast docs/materials/scripts/drag-lag-clips.sh
```

Read `clips.txt`, `vdrag.log` and `scroll-fast.gif`. The pilot passes when all of these
hold:
- vdrag found its window (`ready`).
- The drag finished (`done`).
- The interactive-move check passed (`clips.txt`).
- The frames show the glass flexing during the drag.

If vdrag reports a protocol error or that the move never started, read `niri.log` for
the xdg-shell move handling and fix the client. That case is a driver bug in this
task, not a design question. If the preflight refuses on host load, park with
`--reason quiet --waiting-on user --minutes 8` and list the phases: build 3, preflight
0.5, pilot 1, full 3.

Every attempt ends with a `tasks note material-55f8a0 "run: …"` line in the form the
tasks skill gives.

- [ ] **Step 5: Full run and the brief**

Run without `SEQUENCES`. Then:

1. Add to "Drag baseline finding" in `docs/notes/2026-09-29-material-dynamics-brief.md`
   a "Follow-lag" column holding the Task 2 trace peaks (drag, hold-settle ms, release)
   from `just test-one -p niri drag_dynamics -- --nocapture`. Keep the baseline column.
2. Add a "Follow-lag clips" subsection: the run directory, the five clips, the
   per-sequence interactive-move lines, and a `capture.json` reference.
3. Publish the five GIFs and contact sheets on a review page, the same way the
   `material-8e3b73` clips were published. Put its link in the subsection.
4. In the spec, change the status line to "accepted (spec round 6); implemented in
   material-4354cf, clips awaiting owner judgment". In §7, replace the virtual-keyboard
   driver sentence with: "The driver is a move client: it maps its own window, presses
   a virtual-pointer button on it and calls `xdg_toplevel.move`. The pinned Smithay does
   not let virtual-keyboard modifiers reach niri's Mod check (plan review round 1)."

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
