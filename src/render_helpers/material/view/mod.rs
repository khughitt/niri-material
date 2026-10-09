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

use glam::{DVec2, DVec3};

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

/// No point drawn during a swing is nearer the camera than this fraction of its distance: at
/// most 4× magnification (§4.1).
pub const VIEW_NEAR_CLIP: f64 = 0.75;

/// One frame's rotation about the pivot and its camera (§3.2, §4.1). R turns by θ in the
/// (dir, z) plane, so the side along +dir lifts while θ > 0. The camera sits on the pivot's
/// normal at D = κE / w.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// cos θ − 1, written −2 sin²(θ/2): exactly 0 at θ = 0, and accurate near it.
    pub cos_m1: f64,
    pub sin: f64,
    pub dir: DVec2,
    /// λ = w / (κE): the camera ray's slope per px from the pivot, and ζ per px of height.
    pub lambda: f64,
    /// The slab's centre on its front face, as `slabSurface` computes it.
    pub pivot: DVec2,
}

/// An axis-aligned box as (min, max).
pub type Bounds = (DVec2, DVec2);

impl Camera {
    pub fn new(pose: &ViewPose, slab_min: DVec2, slab_size: DVec2) -> Self {
        let extent = pose.dir.x.abs() * slab_size.x + pose.dir.y.abs() * slab_size.y;
        let half = 0.5 * pose.theta;
        Self {
            cos_m1: -2. * half.sin() * half.sin(),
            sin: pose.theta.sin(),
            dir: pose.dir,
            lambda: pose.weight / (pose.kappa * extent),
            pivot: slab_min + slab_size * 0.5,
        }
    }

    /// (R⁻¹ − I) v: the turn by −θ, less the identity. Mirrors `viewUnrotateDelta`.
    pub fn unrotate_delta(&self, v: DVec3) -> DVec3 {
        let a = v.truncate().dot(self.dir);
        let da = a * self.cos_m1 + v.z * self.sin;
        let dz = -a * self.sin + v.z * self.cos_m1;
        (self.dir * da).extend(dz)
    }

    /// (R − I) v. Mirrors `viewRotateDelta`.
    pub fn rotate_delta(&self, v: DVec3) -> DVec3 {
        let a = v.truncate().dot(self.dir);
        let da = a * self.cos_m1 - v.z * self.sin;
        let dz = a * self.sin + v.z * self.cos_m1;
        (self.dir * da).extend(dz)
    }

    /// The camera ray through screen point `s` on z = 0, slab-local: origin
    /// `o = s + (R⁻¹ − I)(s − P)` and direction `r = R⁻¹ ((s − P) λ, −1)`, unnormalised.
    pub fn ray(&self, s: DVec2) -> (DVec3, DVec3) {
        let rel = s - self.pivot;
        let o = s.extend(0.) + self.unrotate_delta(rel.extend(0.));
        let dir = (rel * self.lambda).extend(-1.);
        (o, dir + self.unrotate_delta(dir))
    }

    /// A slab-local point in the world: `q + (R − I)(q − P)`.
    pub fn to_world(&self, q: DVec3) -> DVec3 {
        q + self.rotate_delta(q - self.pivot.extend(0.))
    }

    /// ζ of a world point: its height toward the camera as a fraction of the camera distance.
    pub fn zeta(&self, e: DVec3) -> f64 {
        e.z * self.lambda
    }

    /// Where a world point projects on the screen along its camera ray:
    /// `e.xy + (e.xy − P)(f − 1)`, `f − 1 = ζ / (1 − ζ)`.
    pub fn project(&self, e: DVec3) -> DVec2 {
        let zeta = self.zeta(e);
        e.truncate() + (e.truncate() - self.pivot) * (zeta / (1. - zeta))
    }

    /// `toScreen(q) − q.xy` for a slab-local point, exactly zero at the identity. Mirrors
    /// `viewScreenShift`.
    pub fn screen_shift(&self, q: DVec3) -> DVec2 {
        let dq = self.rotate_delta(q - self.pivot.extend(0.));
        let exy = q.truncate() + dq.truncate();
        let zeta = (q.z + dq.z) * self.lambda;
        dq.truncate() + (exy - self.pivot) * (zeta / (1. - zeta))
    }

    /// `toScreen` (§4.1 step 5).
    pub fn to_screen(&self, q: DVec3) -> DVec2 {
        q.truncate() + self.screen_shift(q)
    }
}

/// The element footprint during a swing (§6): the bounding box of the projections of the slab
/// box's eight corners and the texture rect's four, each set first clipped at ζ = 3/4. The kept
/// vertices are the corners at or below the plane plus each edge's crossing of it.
pub fn footprint(camera: &Camera, slab: Bounds, tex: Bounds, thickness: f64) -> Bounds {
    let corners = |(lo, hi): Bounds, z: f64| {
        [
            DVec3::new(lo.x, lo.y, z),
            DVec3::new(hi.x, lo.y, z),
            DVec3::new(hi.x, hi.y, z),
            DVec3::new(lo.x, hi.y, z),
        ]
        .map(|q| camera.to_world(q))
    };
    let front = corners(slab, 0.);
    let back = corners(slab, -thickness);
    let decal = corners(tex, 0.);
    let mut edges = Vec::with_capacity(16);
    for i in 0..4 {
        let j = (i + 1) % 4;
        edges.extend([
            (front[i], front[j]),
            (back[i], back[j]),
            (front[i], back[i]),
        ]);
        edges.push((decal[i], decal[j]));
    }
    let mut lo = DVec2::INFINITY;
    let mut hi = DVec2::NEG_INFINITY;
    let mut keep = |e: DVec3| {
        let s = camera.project(e);
        lo = lo.min(s);
        hi = hi.max(s);
    };
    for e in front.into_iter().chain(back).chain(decal) {
        if camera.zeta(e) <= VIEW_NEAR_CLIP {
            keep(e);
        }
    }
    for (a, b) in edges {
        let (za, zb) = (camera.zeta(a), camera.zeta(b));
        if (za - VIEW_NEAR_CLIP) * (zb - VIEW_NEAR_CLIP) < 0. {
            keep(a + (b - a) * ((VIEW_NEAR_CLIP - za) / (zb - za)));
        }
    }
    (lo, hi)
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

    /// A deterministic stream of uniforms in [0, 1).
    pub(super) fn uniforms(seed: u64) -> impl FnMut() -> f64 {
        let mut i = seed << 20;
        move || {
            i += 1;
            hash01(i)
        }
    }

    fn slab(min: (f64, f64), size: (f64, f64)) -> (DVec2, DVec2) {
        (DVec2::new(min.0, min.1), DVec2::new(size.0, size.1))
    }

    fn peak(peak_deg: f64, dir: DVec2, kappa: f64) -> ViewPose {
        ViewPose {
            theta: peak_deg.to_radians(),
            dir,
            weight: 1.,
            kappa,
        }
    }

    const DIRS: [DVec2; 4] = [
        DVec2::new(1., 0.),
        DVec2::new(0., 1.),
        DVec2::new(0.6, 0.8),
        DVec2::new(-0.8, 0.6),
    ];

    #[test]
    fn the_identity_reproduces_its_inputs() {
        let (min, size) = slab((10., 20.), (640., 360.));
        let mut rnd = uniforms(1);
        for dir in DIRS {
            let camera = Camera::new(&ViewPose::identity(dir, 6.), min, size);
            for _ in 0..2000 {
                let s = DVec2::new(rnd() * 900. - 100., rnd() * 600. - 100.);
                let (o, r) = camera.ray(s);
                assert_eq!(o, s.extend(0.));
                assert_eq!(r, DVec3::new(0., 0., -1.));
                let q = DVec3::new(s.x, s.y, -rnd() * 200.);
                assert_eq!(camera.to_world(q), q);
                assert_eq!(camera.screen_shift(q), DVec2::ZERO);
                assert_eq!(camera.to_screen(q), q.truncate());
                assert_eq!(camera.zeta(q), 0.);
            }
        }
    }

    #[test]
    fn the_positive_lobe_lifts_the_side_along_dir() {
        let (min, size) = slab((0., 0.), (400., 300.));
        let camera = Camera::new(&peak(10., DVec2::X, 6.), min, size);
        let right = camera.to_world(DVec3::new(400., 150., 0.));
        let left = camera.to_world(DVec3::new(0., 150., 0.));
        assert!(right.z > 0. && left.z < 0., "{right} {left}");
        // The projection is the pinhole's: a raised point moves away from the pivot.
        let shift = camera.to_screen(DVec3::new(400., 150., 0.)) - DVec2::new(400., 150.);
        assert!(shift.x < 0., "foreshortened toward the pivot: {shift}");
    }

    /// Points on the slab box's faces and on the texture rect, the drawn ones only.
    fn drawn_points(
        camera: &Camera,
        s: (DVec2, DVec2),
        t: (DVec2, DVec2),
        thickness: f64,
    ) -> Vec<DVec2> {
        let mut out = Vec::new();
        let n = 24;
        let mut push = |q: DVec3| {
            let e = camera.to_world(q);
            if camera.zeta(e) <= VIEW_NEAR_CLIP {
                out.push(camera.project(e));
            }
        };
        for i in 0..=n {
            for j in 0..=n {
                let (a, b) = (i as f64 / n as f64, j as f64 / n as f64);
                let slab = s.0 + (s.1 - s.0) * DVec2::new(a, b);
                for z in [0., -thickness, -thickness * a] {
                    push(slab.extend(z));
                }
                for z in [0., -thickness * b] {
                    push(DVec3::new(s.0.x, slab.y, z));
                    push(DVec3::new(s.1.x, slab.y, z));
                    push(DVec3::new(slab.x, s.0.y, z));
                    push(DVec3::new(slab.x, s.1.y, z));
                }
                push((t.0 + (t.1 - t.0) * DVec2::new(a, b)).extend(0.));
            }
        }
        out
    }

    fn assert_contains(fp: Bounds, points: &[DVec2], what: &str) {
        assert!(fp.0.is_finite() && fp.1.is_finite(), "{what}: {fp:?}");
        for p in points {
            assert!(
                p.x >= fp.0.x - 1e-9
                    && p.y >= fp.0.y - 1e-9
                    && p.x <= fp.1.x + 1e-9
                    && p.y <= fp.1.y + 1e-9,
                "{what}: {p} outside {fp:?}"
            );
        }
    }

    #[test]
    fn the_footprint_contains_every_drawn_point() {
        let thickness = 31.2;
        for (slab_min, slab_size, tex) in [
            // A texture 150 px past the slab on every side.
            ((0., 0.), (640., 360.), ((-150., -150.), (790., 510.))),
            // The (250, 250) texture corner of a 100 px slab (§6).
            ((-50., -50.), (100., 100.), ((-50., -50.), (250., 250.))),
        ] {
            let s = (
                DVec2::from(slab_min),
                DVec2::from(slab_min) + DVec2::from(slab_size),
            );
            let t = (DVec2::from(tex.0), DVec2::from(tex.1));
            for peak_deg in [2_f64, 10., 20.] {
                for kappa in [1.5, 6., 50.] {
                    for dir in DIRS {
                        for sign in [1., -1.] {
                            let pose = ViewPose {
                                theta: sign * peak_deg.to_radians(),
                                ..peak(peak_deg, dir, kappa)
                            };
                            let camera = Camera::new(&pose, s.0, slab_size.into());
                            let fp = footprint(&camera, s, t, thickness);
                            let what = format!("{peak_deg}° κ {kappa} {dir} {sign}");
                            assert_contains(fp, &drawn_points(&camera, s, t, thickness), &what);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_decal_past_the_near_clip_has_a_finite_footprint() {
        // A texture reaching x = 500 from the pivot of a 100 px slab at 20°, κ = 1.5 (§8.1).
        let s = (DVec2::new(-50., -50.), DVec2::new(50., 50.));
        let t = (DVec2::new(-50., -50.), DVec2::new(500., 50.));
        let camera = Camera::new(&peak(20., DVec2::X, 1.5), s.0, DVec2::splat(100.));
        let far = camera.to_world(DVec3::new(500., 0., 0.));
        assert!(
            camera.zeta(far) > VIEW_NEAR_CLIP,
            "the case must cross the clip"
        );
        let fp = footprint(&camera, s, t, 31.2);
        assert_contains(fp, &drawn_points(&camera, s, t, 31.2), "past the clip");
        // No point is magnified more than 4× about the pivot.
        assert!(fp.1.x < 4. * 500. && fp.1.x > 50., "{fp:?}");
    }

    #[test]
    fn the_footprint_is_finite_on_every_frame_of_a_run() {
        let s = (DVec2::new(-50., -50.), DVec2::new(50., 50.));
        let t = (DVec2::new(-400., -400.), DVec2::new(900., 900.));
        let settle = Duration::from_millis(900);
        for ms in 0..=900 {
            let Some(pose) = ViewPose::at(
                20.,
                settle,
                Duration::from_millis(ms),
                DVec2::new(0.6, 0.8),
                1.5,
            ) else {
                continue;
            };
            let camera = Camera::new(&pose, s.0, DVec2::splat(100.));
            let fp = footprint(&camera, s, t, 200.);
            assert_contains(fp, &drawn_points(&camera, s, t, 200.), &format!("{ms} ms"));
        }
    }

    #[test]
    fn the_slab_never_reaches_the_near_clip() {
        let mut rnd = uniforms(2);
        for _ in 0..20_000 {
            let peak_deg = rnd() * 20.;
            let kappa = 1.5 + rnd() * 48.5;
            let angle = rnd() * 2. * PI;
            let dir = DVec2::new(angle.cos(), angle.sin());
            let size = DVec2::new(2. + rnd() * 4000., 2. + rnd() * 3000.);
            let thickness = rnd() * 200.;
            let camera = Camera::new(&peak(peak_deg, dir, kappa), DVec2::ZERO, size);
            let bound = peak_deg.to_radians().sin() / (2. * kappa);
            for (x, y) in [(0., 0.), (1., 0.), (1., 1.), (0., 1.)] {
                for z in [0., -thickness] {
                    let q = DVec3::new(x * size.x, y * size.y, z);
                    let zeta = camera.zeta(camera.to_world(q));
                    assert!(zeta <= bound + 1e-12, "ζ {zeta} > {bound}");
                    assert!(bound <= 0.114, "{bound}");
                }
            }
        }
    }

    #[test]
    fn rays_through_the_slab_point_down() {
        let mut rnd = uniforms(3);
        for _ in 0..20_000 {
            let peak_deg = rnd() * 20.;
            let kappa = 1.5 + rnd() * 48.5;
            let angle = rnd() * 2. * PI;
            let size = DVec2::new(2. + rnd() * 4000., 2. + rnd() * 3000.);
            let camera = Camera::new(
                &peak(peak_deg, DVec2::new(angle.cos(), angle.sin()), kappa),
                DVec2::ZERO,
                size,
            );
            let fp = footprint(
                &camera,
                (DVec2::ZERO, size),
                (DVec2::ZERO, size),
                rnd() * 200.,
            );
            let s = fp.0 + (fp.1 - fp.0) * DVec2::new(rnd(), rnd());
            assert!(
                camera.ray(s).1.z < 0.,
                "r.z ≥ 0 at {s} in the slab's footprint"
            );
        }
    }
}
