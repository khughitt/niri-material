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
                assert!(
                    (l.x - ex).abs() < 1e-3,
                    "ratio {ratio} t {t}: {} vs {ex}",
                    l.x
                );
                assert!(
                    (l.y - ey).abs() < 1e-3,
                    "ratio {ratio} t {t}: {} vs {ey}",
                    l.y
                );
                assert!(
                    (v.x - evx).abs() < 1e-1,
                    "ratio {ratio} t {t}: {} vs {evx}",
                    v.x
                );
                assert!(
                    (v.y - evy).abs() < 1e-1,
                    "ratio {ratio} t {t}: {} vs {evy}",
                    v.y
                );
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
                assert!(
                    (fd - v).abs() < 1e-2 * v.abs().max(1.),
                    "ratio {ratio} t {t}"
                );
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
            FollowSpring::from_config(&Animation {
                off: true,
                ..spring
            }),
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
