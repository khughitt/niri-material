//! The glass edge in f64: the two-boundary bevel coordinate, the height-field
//! profile, the softened outer gradient, the bevel normal, and the refracted
//! ray's path (docs/specs/2026-09-30-glass-edge-optics-design.md §3.1).
//! `prelude.frag` mirrors these functions, and a test pins the constants the
//! two share. Coordinates are logical px, y down, +z toward the viewer.

use glam::{DVec2, DVec3};

/// The bevel's slope cap, |grad h| <= 20: the structural normal keeps
/// n.z >= 1 / sqrt(401).
pub const BEVEL_SLOPE_CAP: f64 = 20.;
/// The outer gradient's softening length, logical px.
pub const BEVEL_OUTER_SOFTEN: f64 = 0.5;
/// Below this length the softened gradient sits on the ridge between parallel
/// sides, and the inner gradient stands in.
pub const BEVEL_RIDGE_EPS: f64 = 1e-4;
/// Keeps the profile slope's powers off pow(0, y <= 0), which GLSL leaves
/// undefined.
pub const BEVEL_PROFILE_EPS: f64 = 1e-6;
/// Where log(log(1 + x) / x) switches to its series 1 - x / 2.
pub const BEVEL_LSP_SMALL: f64 = 1e-3;
/// `tanhs` clamps its argument here, so exp never sees more than 20.
pub const BEVEL_TANH_CLAMP: f64 = 10.;
/// The ray path's floor on -t.z: it binds only for aberration taps whose
/// effective index passes 4.
pub const RAY_PATH_FLOOR: f64 = 0.25;
/// Tap normals are lifted to at least this z before normalizing.
pub const TAP_MIN_Z: f64 = 0.05;

/// Corner radii in `CornerRadius` order: top-left, top-right, bottom-right,
/// bottom-left.
pub type Radii = [f64; 4];

/// This quadrant's radius; mirrors `cornerRadius`.
pub fn corner_radius(p: DVec2, r: Radii) -> f64 {
    let top = if p.x < 0. { r[0] } else { r[1] };
    let bottom = if p.x < 0. { r[3] } else { r[2] };
    if p.y < 0. {
        top
    } else {
        bottom
    }
}

/// Signed distance to a rounded box of half-size `b`; mirrors `sdRoundedBox`.
pub fn sd_rounded_box(p: DVec2, b: DVec2, radii: Radii) -> f64 {
    let r = corner_radius(p, radii);
    let q = p.abs() - b + DVec2::splat(r);
    q.max_element().min(0.) + q.max(DVec2::ZERO).length() - r
}

/// The exact outward gradient; mirrors `sdRoundedBoxGrad`.
pub fn sd_rounded_box_grad(p: DVec2, b: DVec2, radii: Radii) -> DVec2 {
    let r = corner_radius(p, radii);
    let q = p.abs() - b + DVec2::splat(r);
    let g = if q.x > 0. && q.y > 0. {
        q / q.length()
    } else if q.x > q.y {
        DVec2::X
    } else {
        DVec2::Y
    };
    g * DVec2::new(
        if p.x >= 0. { 1. } else { -1. },
        if p.y >= 0. { 1. } else { -1. },
    )
}

/// log(softplus(t)), without overflow for large t or loss for very negative t.
pub fn lsp(t: f64) -> f64 {
    if t > 0. {
        return (t + (1. + (-t).exp()).ln()).ln();
    }
    let x = t.exp();
    let ratio = if x < BEVEL_LSP_SMALL {
        1. - 0.5 * x
    } else {
        (1. + x).ln() / x
    };
    t + ratio.ln()
}

/// tanh with a clamped argument, written with exp as GLSL ES 1.00 must.
pub fn tanhs(t: f64) -> f64 {
    let e = (2. * t.clamp(-BEVEL_TANH_CLAMP, BEVEL_TANH_CLAMP)).exp();
    (e - 1.) / (e + 1.)
}

/// The outer box's softened outward gradient (spec §3.1): softplus for each
/// positive part, `tanhs` for each sign, normalized in the log domain. ZERO on
/// the ridge between parallel sides. Mirrors `softOuterGrad`.
pub fn soft_outer_grad(p: DVec2, b: DVec2, radii: Radii) -> DVec2 {
    let r = corner_radius(p, radii);
    let q = p.abs() - b + DVec2::splat(r);
    let l = DVec2::new(lsp(q.x / BEVEL_OUTER_SOFTEN), lsp(q.y / BEVEL_OUTER_SOFTEN));
    let m = l.max_element();
    let c = DVec2::new(
        (l.x - m).exp() * tanhs(p.x / BEVEL_OUTER_SOFTEN),
        (l.y - m).exp() * tanhs(p.y / BEVEL_OUTER_SOFTEN),
    );
    let len = c.length();
    if len < BEVEL_RIDGE_EPS {
        DVec2::ZERO
    } else {
        c / len
    }
}

/// f(u) = 1 - (1 - u^k)^(1/k): 0 at the face edge, 1 at the silhouette.
/// Mirrors `bevelProfile`.
pub fn profile(u: f64, k: f64) -> f64 {
    1. - (1. - u.powf(k)).max(0.).powf(1. / k)
}

/// f'(u) = u^(k-1) (1 - u^k)^(1/k - 1), both bases kept off zero. Mirrors
/// `bevelProfileSlope`.
pub fn profile_slope(u: f64, k: f64) -> f64 {
    let a = u.clamp(BEVEL_PROFILE_EPS, 1.);
    let b = (1. - u.powf(k)).clamp(BEVEL_PROFILE_EPS, 1.);
    a.powf(k - 1.) * b.powf(1. / k - 1.)
}

/// The slab's two boxes: the fixed silhouette and the face that trails the
/// jelly. Radii are taken as given (the shader fits them before this point).
#[derive(Debug, Clone, Copy)]
pub struct Slab {
    pub center: DVec2,
    pub half: DVec2,
    pub outer_r: Radii,
    pub face_center: DVec2,
    pub face_half: DVec2,
    pub face_r: Radii,
    pub chamfer: f64,
    pub thickness: f64,
    /// `bevel-profile`, k.
    pub profile: f64,
    /// One physical pixel in logical px, `1 / scale`.
    pub aa: f64,
}

/// What `slabSurface` returns for one fragment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeSample {
    pub outer_dist: f64,
    pub inner_dist: f64,
    /// Local height h: `thickness` on the face.
    pub height: f64,
    /// u: 0 at the face edge, 1 at the silhouette, 0 on the face.
    pub across: f64,
    /// normalize(grad u): outward across the bevel; ZERO on the face.
    pub across_dir: DVec2,
    /// The structural normal.
    pub normal: DVec3,
}

impl Slab {
    /// Concentric boxes at rest: `outer_r = face_r + chamfer`.
    pub fn at_rest(
        half: DVec2,
        chamfer: f64,
        face_radius: f64,
        thickness: f64,
        profile: f64,
    ) -> Self {
        Self {
            center: DVec2::ZERO,
            half,
            outer_r: [face_radius + chamfer; 4],
            face_center: DVec2::ZERO,
            face_half: half - DVec2::splat(chamfer),
            face_r: [face_radius; 4],
            chamfer,
            thickness,
            profile,
            aa: 1.,
        }
    }

    /// The face moved by `shift` and resized by `resize` px, as `slabSurface`
    /// applies `mat_jelly_move` and `mat_jelly_resize`.
    pub fn deformed(self, shift: DVec2, resize: DVec2) -> Self {
        Self {
            face_center: self.center + shift,
            face_half: (self.half - DVec2::splat(self.chamfer))
                * (DVec2::ONE + resize / (2. * self.half).max(DVec2::ONE)),
            ..self
        }
    }

    /// Mirrors `slabSurface` from the distances on.
    pub fn surface(&self, p: DVec2) -> EdgeSample {
        self.surface_with(p, soft_outer_grad)
    }

    /// `surface` with the outer gradient supplied: the exact one is the
    /// tests' negative control.
    pub fn surface_with(
        &self,
        p: DVec2,
        outer_grad: fn(DVec2, DVec2, Radii) -> DVec2,
    ) -> EdgeSample {
        let d = sd_rounded_box(p - self.center, self.half, self.outer_r);
        let di = sd_rounded_box(p - self.face_center, self.face_half, self.face_r);
        let mut s = EdgeSample {
            outer_dist: d,
            inner_dist: di,
            height: self.thickness,
            across: 0.,
            across_dir: DVec2::ZERO,
            normal: DVec3::Z,
        };
        if !(self.chamfer > 0. && di >= 0.) {
            return s;
        }
        let g_in = sd_rounded_box_grad(p - self.face_center, self.face_half, self.face_r);
        let mut g_out = outer_grad(p - self.center, self.half, self.outer_r);
        if g_out == DVec2::ZERO {
            g_out = g_in;
        }
        let w = di - d;
        let grad_u = if w < self.aa {
            s.across = 1.;
            g_in / w.max(self.aa)
        } else {
            s.across = (di / w).clamp(0., 1.);
            (-d * g_in + di * g_out) / (w * w)
        };
        let rise = self.chamfer.min(self.thickness);
        s.height = self.thickness - rise * profile(s.across, self.profile);
        let mut slope = rise * profile_slope(s.across, self.profile) * grad_u;
        let len = slope.length();
        if len > BEVEL_SLOPE_CAP {
            slope *= BEVEL_SLOPE_CAP / len;
        }
        s.normal = slope.extend(1.).normalize();
        s.across_dir = if grad_u.length() > 0. {
            grad_u.normalize()
        } else {
            g_in
        };
        s
    }
}

/// GLSL's `refract`.
pub fn refract(i: DVec3, n: DVec3, eta: f64) -> DVec3 {
    let d = n.dot(i);
    let k = 1. - eta * eta * (1. - d * d);
    if k < 0. {
        DVec3::ZERO
    } else {
        eta * i - (eta * d + k.sqrt()) * n
    }
}

/// The orthographic view ray refracted into glass of index `ior`.
pub fn ray(n: DVec3, ior: f64) -> DVec3 {
    refract(DVec3::NEG_Z, n, 1. / ior)
}

/// The refracted ray's length to the backdrop plane under local height `h`;
/// mirrors `rayPath`.
pub fn ray_path(t: DVec3, h: f64) -> f64 {
    h / (-t.z).max(RAY_PATH_FLOOR)
}

/// A tap's normal lifted into the structural cap; mirrors `liftTapNormal`.
pub fn lift_tap_normal(n: DVec3) -> DVec3 {
    DVec3::new(n.x, n.y, n.z.max(TAP_MIN_Z)).normalize()
}

/// One tap's in-plane displacement, as `tap` applies it.
pub fn displacement(n: DVec3, ior: f64, h: f64) -> DVec2 {
    let t = ray(lift_tap_normal(n), ior);
    t.truncate() * ray_path(t, h)
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use glam::Vec2;

    use super::*;

    const IOR: f64 = 1.28;

    /// The point `u` across the right side of a resting 112 px slab, chamfer 12.
    fn right_side(thickness: f64, k: f64, u: f64) -> EdgeSample {
        Slab::at_rest(DVec2::splat(56.), 12., 0., thickness, k)
            .surface(DVec2::new(44. + 12. * u, 0.))
    }

    #[test]
    fn the_reference_table_holds() {
        // Spec §3.1: (thickness, k, u, h, n.z, L, displacement) at chamfer 12, ior 1.28.
        let rows = [
            (6., 1., 0., 6.00, 0.894, 6.03, 0.64),
            (6., 1., 0.9, 0.60, 0.894, 0.60, 0.06),
            (6., 2., 0.9, 2.62, 0.696, 2.67, 0.55),
            (12., 1., 0.5, 6.00, 0.707, 6.12, 1.22),
            (12., 2., 0.9, 5.23, 0.436, 5.55, 1.85),
            (31.2, 1., 1., 19.20, 0.707, 19.59, 3.89),
            (31.2, 2., 0.9, 24.43, 0.436, 25.91, 8.64),
            (31.2, 2., 1., 19.20, 0.050, 23.69, 13.87),
            (6., 2., 1., 0.00, 0.050, 0.00, 0.00),
            (12., 2., 1., 0.00, 0.050, 0.00, 0.00),
            (75.3, 2., 0.9, 68.53, 0.436, 72.69, 24.24),
            (75.3, 2., 1., 63.30, 0.050, 78.10, 45.74),
            (12.1, 2., 0.9, 5.33, 0.436, 5.65, 1.89),
            (12.1, 2., 1., 0.10, 0.050, 0.12, 0.07),
        ];
        for (thickness, k, u, h, nz, path, shift) in rows {
            let s = right_side(thickness, k, u);
            let t = ray(s.normal, IOR);
            for (name, got, want) in [
                ("h", s.height, h),
                ("n.z", s.normal.z, nz),
                ("L", ray_path(t, s.height), path),
                (
                    "shift",
                    displacement(s.normal, IOR, s.height).length(),
                    shift,
                ),
            ] {
                assert!(
                    (got - want).abs() < 0.01,
                    "{thickness}/{k}/{u}: {name} {got} vs {want}"
                );
            }
        }
    }

    #[test]
    fn the_face_is_flat_and_its_path_is_the_thickness() {
        let slab = Slab::at_rest(DVec2::splat(56.), 12., 0., 31.2, 2.);
        let face = slab.surface(DVec2::new(10., -5.));
        assert_eq!(face.normal, DVec3::Z);
        assert_eq!((face.height, face.across), (31.2, 0.));
        assert!((ray_path(ray(face.normal, IOR), face.height) - 31.2).abs() < 1e-12);
        assert!(displacement(face.normal, IOR, face.height).length() < 1e-12);
    }

    #[test]
    fn upward_normals_bound_the_ray_and_the_floor_binds_only_past_index_four() {
        let cap_z = 1. / 401f64.sqrt();
        for ior in [1., 1.28, 3.] {
            for i in 0..=1000 {
                let z = cap_z + (1. - cap_z) * f64::from(i) / 1000.;
                let t = ray(DVec3::new((1. - z * z).sqrt(), 0., z), ior);
                assert!(
                    -t.z >= 1. / ior - 1e-12,
                    "ior {ior}, n.z {z}: -t.z {}",
                    -t.z
                );
                assert!(-t.z > RAY_PATH_FLOOR);
            }
        }
        let rim = DVec3::new((1f64 - 1. / 401.).sqrt(), 0., cap_z);
        assert!(-ray(rim, 4.).z >= RAY_PATH_FLOOR);
        assert!(
            -ray(rim, 9.).z < RAY_PATH_FLOOR,
            "the floor caps aberration taps at index 9"
        );
    }

    /// Today's chamfer normal: one slope across the whole band.
    fn today(slab: &Slab, p: DVec2) -> DVec3 {
        let g = sd_rounded_box_grad(p - slab.face_center, slab.face_half, slab.face_r);
        let rise = slab.chamfer.min(slab.thickness);
        let slope = rise.hypot(slab.chamfer);
        (g * (rise / slope)).extend(slab.chamfer / slope)
    }

    /// Every sample strictly inside the bevel on a `step` grid.
    fn bevel_samples(slab: &Slab, step: f64) -> Vec<(DVec2, EdgeSample)> {
        let reach = slab.half + DVec2::splat(2.);
        let n = (2. * reach / step).ceil();
        let mut out = Vec::new();
        for i in 0..=n.x as usize {
            for j in 0..=n.y as usize {
                let p = slab.center - reach + DVec2::new(i as f64, j as f64) * step;
                let s = slab.surface(p);
                if s.inner_dist > 0. && s.outer_dist < 0. {
                    out.push((p, s));
                }
            }
        }
        out
    }

    #[test]
    fn profile_one_at_rest_is_todays_bevel_away_from_the_corner_junctions() {
        for radius in [0., 8.] {
            let slab = Slab::at_rest(DVec2::splat(56.), 12., radius, 20., 1.);
            let samples = bevel_samples(&slab, 0.1);
            let mut over = 0;
            for (p, s) in &samples {
                let dev = (s.normal - today(&slab, *p)).length();
                assert!(dev <= 0.03, "r {radius} at {p}: {dev}");
                if dev > 0.01 {
                    over += 1;
                }
                // q of the outer box: its zero lines are the arc-to-side junctions.
                let q = p.abs() - slab.half + DVec2::splat(radius + slab.chamfer);
                if q.x.abs().min(q.y.abs()) > 1.5 {
                    assert!(
                        dev <= 0.002,
                        "r {radius} at {p}: {dev} away from a junction"
                    );
                }
                if q.x.min(q.y) <= -8. {
                    assert!(dev <= 1e-6, "r {radius} at {p}: {dev} on a straight side");
                }
            }
            assert!(
                over * 100 <= samples.len() * 3,
                "r {radius}: {over} of {}",
                samples.len()
            );
        }
    }

    /// The largest normal change between neighbouring samples along `lines`,
    /// strictly inside the bevel.
    fn max_jump(slab: &Slab, lines: &[(DVec2, DVec2)], step: f64, exact: bool) -> f64 {
        let grad: fn(DVec2, DVec2, Radii) -> DVec2 = if exact {
            sd_rounded_box_grad
        } else {
            soft_outer_grad
        };
        let mut worst = 0f64;
        for &(a, b) in lines {
            let n = ((b - a).length() / step).ceil() as usize;
            let mut prev: Option<DVec3> = None;
            for i in 0..=n {
                let s = slab.surface_with(a + (b - a) * (i as f64 / n as f64), grad);
                if s.inner_dist > 0. && s.outer_dist < 0. {
                    if let Some(prev) = prev {
                        worst = worst.max((s.normal - prev).length());
                    }
                    prev = Some(s.normal);
                } else {
                    prev = None;
                }
            }
        }
        worst
    }

    fn lines(points: &[((f64, f64), (f64, f64))]) -> Vec<(DVec2, DVec2)> {
        points
            .iter()
            .map(|&(a, b)| (DVec2::from(a), DVec2::from(b)))
            .collect()
    }

    /// Lines through the bottom-right corner of a 112 px slab.
    fn corner_lines() -> Vec<(DVec2, DVec2)> {
        lines(&[
            ((36., 52.), (52., 36.)),
            ((40., 56.), (56., 40.)),
            ((38., 50.), (50., 38.)),
            ((30., 47.), (58., 47.)),
            ((47., 30.), (47., 58.)),
            ((44., 44.), (56., 56.)),
            ((35., 48.5), (50., 48.5)),
            ((30., 41.), (58., 41.)),
            ((41., 30.), (41., 58.)),
        ])
    }

    #[test]
    fn the_normal_is_continuous_through_deformed_corners() {
        // Continuity: the largest neighbour jump falls with the step (0.25 is
        // exactly proportional). The face join at k = 1 is a crease by design
        // and lies at inner_dist = 0, which the samples exclude.
        for radius in [0., 8.] {
            for k in [1., 2.] {
                for shift in [(3., 3.), (3., 0.), (-3., -3.), (-3., 0.), (0., -3.)] {
                    let slab = Slab::at_rest(DVec2::splat(56.), 12., radius, 20., k)
                        .deformed(DVec2::from(shift), DVec2::ZERO);
                    let coarse = max_jump(&slab, &corner_lines(), 0.02, false);
                    let fine = max_jump(&slab, &corner_lines(), 0.005, false);
                    assert!(
                        fine <= 0.4 * coarse,
                        "r {radius} k {k} {shift:?}: {coarse} -> {fine}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_exact_outer_gradient_creases_where_the_softened_one_does_not() {
        for k in [1., 2.] {
            let slab = Slab::at_rest(DVec2::splat(56.), 12., 0., 20., k)
                .deformed(DVec2::new(-3., -3.), DVec2::ZERO);
            let coarse = max_jump(&slab, &corner_lines(), 0.02, true);
            let fine = max_jump(&slab, &corner_lines(), 0.005, true);
            assert!(
                fine >= 0.9 * coarse && coarse > 0.04,
                "k {k}: {coarse} -> {fine}"
            );
        }
    }

    #[test]
    fn the_normal_is_continuous_across_a_stadiums_centerlines() {
        let crossings = lines(&[
            ((-5., -97.), (5., -97.)),
            ((-5., 97.), (5., 97.)),
            ((7., -5.), (7., 5.)),
            ((-7., -5.), (-7., 5.)),
            ((-5., -95.5), (5., -95.5)),
            ((-5., -98.5), (5., -98.5)),
        ]);
        for shift in [(0., 0.), (2., 2.), (2., 0.), (0., 2.)] {
            for k in [1., 2.] {
                let slab = Slab::at_rest(DVec2::new(10., 100.), 6., 4., 20., k)
                    .deformed(DVec2::from(shift), DVec2::ZERO);
                let coarse = max_jump(&slab, &crossings, 0.02, false);
                let fine = max_jump(&slab, &crossings, 0.005, false);
                assert!(fine <= 0.3 * coarse, "k {k} {shift:?}: {coarse} -> {fine}");
            }
        }
    }

    /// `soft_outer_grad` in f32, as the shader evaluates it; `None` on the ridge.
    fn soft_outer_grad_f32(p: Vec2, b: Vec2, r: f32) -> Option<Vec2> {
        fn lsp(t: f32) -> f32 {
            if t > 0. {
                return (t + (1. + (-t).exp()).ln()).ln();
            }
            let x = t.exp();
            let ratio = if x < BEVEL_LSP_SMALL as f32 {
                1. - 0.5 * x
            } else {
                (1. + x).ln() / x
            };
            t + ratio.ln()
        }
        fn tanhs(t: f32) -> f32 {
            let c = BEVEL_TANH_CLAMP as f32;
            let e = (2. * t.clamp(-c, c)).exp();
            (e - 1.) / (e + 1.)
        }
        let s = BEVEL_OUTER_SOFTEN as f32;
        let q = p.abs() - b + Vec2::splat(r);
        let l = Vec2::new(lsp(q.x / s), lsp(q.y / s));
        let m = l.max_element();
        let c = Vec2::new(
            (l.x - m).exp() * tanhs(p.x / s),
            (l.y - m).exp() * tanhs(p.y / s),
        );
        assert!(c.is_finite(), "non-finite at p {p}, b {b}, r {r}");
        let len = c.length();
        (len >= BEVEL_RIDGE_EPS as f32).then(|| c / len)
    }

    #[test]
    fn the_softened_gradient_holds_in_f32() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        let (lo, hi) = (2f64.ln(), 4000f64.ln());
        for _ in 0..200_000 {
            let b = DVec2::new(
                (lo + (hi - lo) * next()).exp(),
                (lo + (hi - lo) * next()).exp(),
            );
            let r = next() * b.min_element();
            let p = DVec2::new((next() * 2.4 - 1.2) * b.x, (next() * 2.4 - 1.2) * b.y);
            let wide = soft_outer_grad(p, b, [r; 4]);
            match soft_outer_grad_f32(p.as_vec2(), b.as_vec2(), r as f32) {
                Some(narrow) => {
                    let narrow = narrow.as_dvec2();
                    let angle = narrow.perp_dot(wide).atan2(narrow.dot(wide)).abs();
                    assert!(angle < 0.001, "p {p}, b {b}, r {r}: {angle} rad");
                }
                None => {
                    // Only on the ridge between the long parallel sides.
                    let (short, long) = if b.x < b.y {
                        (p.x, p.y.abs() - (b.y - b.x))
                    } else {
                        (p.y, p.x.abs() - (b.x - b.y))
                    };
                    assert!(
                        short.abs() < 1. && long < 1.,
                        "guard off the ridge: p {p}, b {b}, r {r}"
                    );
                }
            }
        }
    }

    #[test]
    fn lifted_tap_normals_keep_every_ray_downward() {
        let at_cap = DVec3::new((1f64 - 0.05 * 0.05).sqrt(), 0., 0.05);
        assert!(
            (lift_tap_normal(at_cap) - at_cap).length() < 1e-12,
            "continuous at n.z = 0.05"
        );
        for z in [0.06f64, 0.3, 0.9, 1.] {
            let n = DVec3::new((1. - z * z).sqrt() * 0.6, (1. - z * z).sqrt() * 0.8, z);
            assert!(
                (lift_tap_normal(n) - n).length() < 1e-12,
                "n.z {z} is above the cap"
            );
        }
        // Normals swung from straight up, through horizontal, to straight down.
        for ior in [1., 1.28, 3.] {
            for i in 0..=200 {
                let a = PI * f64::from(i) / 200.;
                let n = DVec3::new(a.sin() * 0.6, a.sin() * 0.8, a.cos());
                let lifted = lift_tap_normal(n);
                assert!(lifted.z >= 1. / 401f64.sqrt() - 1e-12);
                let t = ray(lifted, ior);
                assert!(
                    -t.z >= 1. / ior - 1e-12,
                    "ior {ior}, angle {a}: -t.z {}",
                    -t.z
                );
            }
        }
    }

    #[test]
    fn u_spans_the_bevel_on_both_sides_under_motion() {
        for (shift, resize) in [
            ((3., 0.), (0., 0.)),
            ((0., 0.), (6., 0.)),
            ((3., 0.), (-6., 0.)),
        ] {
            let slab = Slab::at_rest(DVec2::splat(56.), 12., 0., 20., 2.)
                .deformed(DVec2::from(shift), DVec2::from(resize));
            for side in [1., -1.] {
                let face = slab.face_center.x + side * slab.face_half.x;
                let rim = side * slab.half.x;
                assert_eq!(
                    slab.surface(DVec2::new(face, 0.)).across,
                    0.,
                    "{shift:?} {resize:?} {side}"
                );
                assert_eq!(
                    slab.surface(DVec2::new(rim, 0.)).across,
                    1.,
                    "{shift:?} {resize:?} {side}"
                );
                let mut last = -1.;
                for i in 0..=1000 {
                    let x = face + (rim - face) * f64::from(i) / 1000.;
                    let u = slab.surface(DVec2::new(x, 0.)).across;
                    assert!(
                        u > last,
                        "{shift:?} {resize:?} {side}: u {u} after {last} at x {x}"
                    );
                    last = u;
                }
            }
        }
    }

    #[test]
    fn a_band_narrower_than_a_pixel_is_all_rim() {
        let mut slab = Slab::at_rest(DVec2::splat(56.), 0.75, 0., 20., 2.);
        slab.aa = 1.;
        for x in [55.3, 55.6, 55.9] {
            let s = slab.surface(DVec2::new(x, 0.));
            assert_eq!(s.across, 1.);
            assert!(s.normal.is_finite() && s.normal.z >= 1. / 401f64.sqrt() - 1e-12);
            assert!(s.height.is_finite());
        }
    }

    #[test]
    fn no_chamfer_or_no_thickness_is_a_flat_face() {
        let flat = Slab::at_rest(DVec2::splat(56.), 0., 0., 20., 2.);
        for x in [0., 50., 55.9] {
            let s = flat.surface(DVec2::new(x, 0.));
            assert_eq!((s.normal, s.height, s.across), (DVec3::Z, 20., 0.));
        }
        let thin = Slab::at_rest(DVec2::splat(56.), 12., 0., 0., 2.);
        for x in [0., 50., 55.9] {
            let s = thin.surface(DVec2::new(x, 0.));
            assert_eq!(s.normal, DVec3::Z, "x {x}");
            assert_eq!(s.height, 0.);
            assert_eq!(ray_path(ray(s.normal, IOR), s.height), 0.);
        }
    }
}
