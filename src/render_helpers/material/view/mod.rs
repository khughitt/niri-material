//! The transient rigid tilt in f64 (docs/specs/2026-10-09-transient-rigid-tilt-design.md):
//! the swing curve and its weight (§3.1), the pose and its direction (§3.2), the camera and
//! the slab-local ray, the projection back to the screen and its near clip (§4.1), and the
//! element footprint (§6). `prelude.frag` mirrors the camera and the projection under the same
//! names. Coordinates are element-local logical px, y down, +z toward the viewer.
//!
//! Every map is the identity plus a correction that is exactly zero at θ = 0 and w = 0, so the
//! identity reproduces its inputs bit for bit (§4.1).

use std::f64::consts::PI;
use std::time::Duration;

use glam::DVec2;

use super::ring::hash01;

/// The peak of g(u) = (1 − u)² sin 2πu over [0, 1], at u = 0.190381.
pub const SWING_PEAK: f64 = 0.610_028_508_853_155_6;
/// Centres closer than this give no direction to swing along.
pub const COINCIDENT_PX: f64 = 1.;

/// The swing's shape at `u` = elapsed / settle, g(u) / G: exactly 0 at u = 0, a peak of 1 at
/// u = 0.1904, one zero crossing at u = 0.5, an overshoot to −0.1638 at u = 0.636, and exactly
/// 0 from u = 1 on (§3.1).
pub fn swing_shape(u: f64) -> f64 {
    if u >= 1. {
        return 0.;
    }
    let u = u.max(0.);
    (1. - u).powi(2) * (2. * PI * u).sin() / SWING_PEAK
}

/// The swing's pose on one frame (§3.2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewPose {
    /// Signed angle, radians: the side along +`dir` lifts while it is positive.
    pub theta: f64,
    /// Unit direction in the glass plane, from the focus origin toward this pane.
    pub dir: DVec2,
    /// The perspective weight |θ| / θp, in [0, 1].
    pub weight: f64,
    /// κ: the camera's distance in multiples of the slab's extent along `dir`.
    pub kappa: f64,
}

impl ViewPose {
    /// The pose `elapsed` into a swing of `settle` that peaks at `peak_deg`. `None` where the
    /// curve is exactly zero: the first and the settled frames are rest.
    pub fn at(
        peak_deg: f64,
        settle: Duration,
        elapsed: Duration,
        dir: DVec2,
        kappa: f64,
    ) -> Option<Self> {
        let shape = swing_shape(elapsed.as_secs_f64() / settle.as_secs_f64());
        (shape != 0.).then(|| Self {
            theta: peak_deg.to_radians() * shape,
            dir,
            weight: shape.abs(),
            kappa,
        })
    }

    /// θ = 0 and w = 0 with the view path on: the identity hook of §8.2.
    #[cfg(test)]
    pub fn identity(dir: DVec2, kappa: f64) -> Self {
        Self {
            theta: 0.,
            dir,
            weight: 0.,
            kappa,
        }
    }
}

/// Where a swing comes from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwingFrom {
    /// From the previous focus: the unit vector from its centre to the new focus's centre, in
    /// output-local logical coordinates.
    Origin([f64; 2]),
    /// No usable origin: the tile seeds a direction of its own.
    Fallback,
}

/// The swing's direction from the previous focus's centre to the new one.
pub fn direction_between(from: [f64; 2], to: [f64; 2]) -> SwingFrom {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = d[0].hypot(d[1]);
    if len < COINCIDENT_PX {
        SwingFrom::Fallback
    } else {
        SwingFrom::Origin([d[0] / len, d[1] / len])
    }
}

/// A unit direction from `seed`, for a swing with no usable origin.
pub fn seeded_direction(seed: u32) -> [f64; 2] {
    let angle = hash01(u64::from(seed) ^ 0x5EED_D1EC) * 2. * PI;
    [angle.cos(), angle.sin()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_starts_at_rest_peaks_overshoots_once_and_ends_still() {
        assert_eq!(swing_shape(0.), 0.);
        let steps = 100_000;
        let (mut peak, mut peak_u) = (f64::NEG_INFINITY, 0.);
        let (mut low, mut low_u) = (f64::INFINITY, 0.);
        let mut crossings = Vec::new();
        let mut prev = swing_shape(1. / steps as f64);
        for i in 1..steps {
            let u = i as f64 / steps as f64;
            let g = swing_shape(u);
            if g > peak {
                (peak, peak_u) = (g, u);
            }
            if g < low {
                (low, low_u) = (g, u);
            }
            if g.signum() != prev.signum() {
                crossings.push(u);
            }
            prev = g;
        }
        assert!((peak - 1.).abs() < 1e-8, "peak {peak}");
        assert!((peak_u - 0.1904).abs() < 1e-3, "peak at {peak_u}");
        assert_eq!(crossings.len(), 1, "one zero crossing: {crossings:?}");
        assert!(
            (crossings[0] - 0.5).abs() < 1e-4,
            "crossing at {}",
            crossings[0]
        );
        assert!((low + 0.1638).abs() < 1e-3, "overshoot {low}");
        assert!((low_u - 0.636).abs() < 1e-3, "overshoot at {low_u}");
        assert_eq!(swing_shape(1.), 0.);
        assert_eq!(swing_shape(1.5), 0.);
        assert!(
            swing_shape(1. - 1e-4).abs() < 1e-8,
            "the last moving frame is still"
        );
        // A struck body: the initial angular velocity is 2π / G.
        let slope = swing_shape(1e-9) / 1e-9;
        assert!((slope - 10.30).abs() < 0.01, "initial slope {slope}");
    }

    #[test]
    fn the_peak_constant_is_the_maximum() {
        let g = |u: f64| (1. - u).powi(2) * (2. * PI * u).sin();
        // tan 2πu = π(1 − u) at the peak; Newton from 0.19.
        let mut u = 0.19;
        for _ in 0..50 {
            let f = (2. * PI * u).tan() - PI * (1. - u);
            let df = 2. * PI / (2. * PI * u).cos().powi(2) + PI;
            u -= f / df;
        }
        assert!(
            (g(u) - SWING_PEAK).abs() < 1e-15,
            "{} vs {SWING_PEAK}",
            g(u)
        );
    }

    #[test]
    fn the_weight_is_the_shape_magnitude_and_rest_is_none() {
        let settle = Duration::from_millis(900);
        let dir = DVec2::X;
        assert_eq!(ViewPose::at(10., settle, Duration::ZERO, dir, 6.), None);
        assert_eq!(ViewPose::at(10., settle, settle, dir, 6.), None);
        assert_eq!(
            ViewPose::at(10., settle, Duration::from_secs(5), dir, 6.),
            None
        );
        for ms in (1..900).step_by(7) {
            let pose = ViewPose::at(10., settle, Duration::from_millis(ms), dir, 6.).unwrap();
            assert!(
                (0. ..=1.).contains(&pose.weight),
                "{ms} ms: {}",
                pose.weight
            );
            let ratio = pose.theta.abs() / 10f64.to_radians();
            assert!((ratio - pose.weight).abs() < 1e-12, "{ms} ms");
        }
        let peak = ViewPose::at(
            10.,
            settle,
            Duration::from_secs_f64(0.9 * 0.190381),
            dir,
            6.,
        )
        .unwrap();
        assert!((peak.weight - 1.).abs() < 1e-6 && peak.theta > 0.);
        let back = ViewPose::at(10., settle, Duration::from_millis(572), dir, 6.).unwrap();
        assert!(back.theta < 0., "the overshoot turns the other way");
        let id = ViewPose::identity(dir, 6.);
        assert_eq!((id.theta, id.weight), (0., 0.));
    }

    #[test]
    fn the_direction_points_from_the_origin_and_coincident_centres_fall_back() {
        assert_eq!(
            direction_between([100., 50.], [400., 50.]),
            SwingFrom::Origin([1., 0.])
        );
        let SwingFrom::Origin(d) = direction_between([0., 0.], [3., -4.]) else {
            panic!("expected an origin direction");
        };
        assert!((d[0] - 0.6).abs() < 1e-12 && (d[1] + 0.8).abs() < 1e-12);
        assert_eq!(
            direction_between([10., 10.], [10.5, 10.5]),
            SwingFrom::Fallback
        );
    }

    #[test]
    fn seeded_directions_are_unit_and_differ_between_seeds() {
        let a = seeded_direction(1);
        let b = seeded_direction(2);
        for d in [a, b] {
            assert!((d[0].hypot(d[1]) - 1.).abs() < 1e-12);
        }
        assert_ne!(a, b);
        assert_eq!(seeded_direction(7), seeded_direction(7));
    }
}
