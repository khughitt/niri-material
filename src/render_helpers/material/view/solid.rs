//! The traced slab under a tilt (docs/specs/2026-10-09-transient-rigid-tilt-design.md §4.1
//! step 4, §4.5): the resting slab as a convex solid Ψ ≤ 0, the ray's segment split where it
//! crosses the silhouette's centre lines, golden-section and bisection bracketing per piece, and
//! the outline distance that gives coverage. `prelude.frag` mirrors it line for line, and a test
//! here pins the constants and lines the two share.
//!
//! The segment runs from the window plane z = 0 to the back plane z = −T and is parameterised by
//! depth, q(t) = h₀ + t·T·ρ with ρ = r / (−r.z), so z = −tT.

use glam::{DVec2, DVec3};

use super::Camera;
use crate::render_helpers::material::bevel::{
    corner_radius, sd_rounded_box, sd_rounded_box_grad, Radii, BEVEL_PROFILE_EPS,
};

/// The golden-section ratio, (√5 − 1) / 2.
pub const VIEW_GOLDEN: f64 = 0.618_033_988_749_895;
/// Golden-section steps per piece: the bracket shrinks to 0.618^28 ≈ 1.4e-6 of the piece.
pub const VIEW_GOLDEN_STEPS: usize = 28;
/// Bisection steps to the entry: the longest piece in the configured ranges (thickness 200,
/// |ρ| ≤ 1.2, scale 4) ends within 1/64 physical px after 16; 24 leaves a margin.
pub const VIEW_BISECT_STEPS: usize = 24;

/// The resting slab as `slabSurface` frames it, without jelly (§4.1 "Jelly").
#[derive(Debug, Clone, Copy)]
pub struct Solid {
    pub center: DVec2,
    pub half: DVec2,
    /// Outer radii, fitted, in `CornerRadius` order.
    pub outer_r: Radii,
    /// The chamfer actually drawn (the tiny-slab guard applied).
    pub chamfer: f64,
    pub thickness: f64,
    /// `bevel-profile`, k.
    pub profile: f64,
}

impl Solid {
    /// min(chamfer, thickness): the bevel's height.
    pub fn rise(&self) -> f64 {
        self.chamfer.min(self.thickness)
    }

    /// σ, the silhouette's signed distance, with the radius of the quadrant `sel` (relative to
    /// the centre) lies in. Mirrors `viewSigma`.
    pub fn sigma(&self, xy: DVec2, sel: DVec2) -> f64 {
        let r = corner_radius(sel, self.outer_r);
        sd_rounded_box(xy - self.center, self.half, [r; 4])
    }

    /// ∇σ in the same quadrant, as `sdRoundedBoxGrad`. Mirrors `viewSigmaGrad`.
    pub fn sigma_grad(&self, xy: DVec2, sel: DVec2) -> DVec2 {
        let r = corner_radius(sel, self.outer_r);
        sd_rounded_box_grad(xy - self.center, self.half, [r; 4])
    }

    /// Ψ (§4.1): ≤ 0 inside. `X` inverts today's `bevelProfile`, so Ψ ≤ 0 is the set
    /// z ≤ height − T. Without a rise (bevel 0 or thickness 0) Ψ is σ. Mirrors `viewPsi`.
    pub fn psi(&self, q: DVec3, sel: DVec2) -> f64 {
        let sigma = self.sigma(q.truncate(), sel);
        let rise = self.rise();
        if rise <= 0. {
            return sigma;
        }
        let y = -q.z / rise;
        let x = if y >= 1. {
            1.
        } else {
            (1. - (1. - y).max(0.).powf(self.profile))
                .max(0.)
                .powf(1. / self.profile)
        };
        sigma + self.chamfer * (1. - x)
    }

    /// ∂Ψ/∂z = c X'(y) / rise, X'(y) = (1 − (1 − y)^k)^(1/k − 1) (1 − y)^(k − 1), both bases
    /// kept off zero as `bevelProfileSlope` keeps them. Mirrors `viewPsiDz`.
    pub fn psi_dz(&self, q: DVec3) -> f64 {
        let rise = self.rise();
        if rise <= 0. {
            return 0.;
        }
        let y = -q.z / rise;
        if y >= 1. {
            return 0.;
        }
        let k = self.profile;
        let b = (1. - y).clamp(BEVEL_PROFILE_EPS, 1.);
        let a = (1. - b.powf(k)).clamp(BEVEL_PROFILE_EPS, 1.);
        self.chamfer * a.powf(1. / k - 1.) * b.powf(k - 1.) / rise
    }
}

/// The ray through one screen point, set up as `viewBegin` sets it up (§4.1 steps 1–3).
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    /// The slab-local direction, unnormalised.
    pub r: DVec3,
    /// The window-plane hit, z exactly 0: the screen point plus its shift.
    pub h0: DVec3,
    /// r / (−r.z): the step per px of depth.
    pub rho: DVec3,
    /// The ray parameter at h₀, from the origin along r.
    pub tau0: f64,
}

impl Segment {
    /// `None` when the ray does not point down (r.z ≥ 0): no plane hit, no glass.
    pub fn new(camera: &Camera, s: DVec2) -> Option<Self> {
        let rel = s - camera.pivot;
        let o_shift = camera.unrotate_delta(rel.extend(0.));
        let dir = (rel * camera.lambda).extend(-1.);
        let r = dir + camera.unrotate_delta(dir);
        if r.z >= 0. {
            return None;
        }
        let tau0 = -o_shift.z / r.z;
        let h0_shift = o_shift.truncate() + r.truncate() * tau0;
        Some(Self {
            r,
            h0: (s + h0_shift).extend(0.),
            rho: r / -r.z,
            tau0,
        })
    }

    /// The point `t` of the way down to the back plane.
    pub fn point(&self, t: f64, thickness: f64) -> DVec3 {
        self.h0 + self.rho * (t * thickness)
    }
}

/// One piece of the segment between centre-line crossings (§4.1 "Quadrant seams"): its ends,
/// each end's plane normal, and the quadrant its midpoint lies in, whose radius the whole piece
/// takes, so the piece is one convex function up to its ends.
#[derive(Debug, Clone, Copy)]
struct Piece {
    a: f64,
    b: f64,
    na: DVec3,
    nb: DVec3,
    sel: DVec2,
}

fn pieces(solid: &Solid, seg: &Segment) -> Vec<Piece> {
    let thickness = solid.thickness;
    let mut cuts = vec![(0., DVec3::Z)];
    if thickness > 0. {
        for (axis, normal) in [(0, DVec3::X), (1, DVec3::Y)] {
            let step = thickness * seg.rho[axis];
            if step != 0. {
                let at = (solid.center[axis] - seg.h0[axis]) / step;
                if at > 0. && at < 1. {
                    cuts.push((at, normal));
                }
            }
        }
    }
    cuts.sort_by(|x, y| x.0.total_cmp(&y.0));
    cuts.push((1., DVec3::Z));
    // Two crossings at one depth leave an empty piece between them; `viewTrace` skips it too.
    cuts.windows(2)
        .filter(|w| w[0].0 < w[1].0)
        .map(|w| {
            let ((a, na), (b, nb)) = (w[0], w[1]);
            let mid = seg.point(0.5 * (a + b), thickness);
            Piece {
                a,
                b,
                na,
                nb,
                sel: mid.truncate() - solid.center,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    A,
    B,
    Interior,
}

/// The minimum of `f` on [a, b], its ends included; a tie goes to an end, the start first
/// (§4.5 "Ties"). Mirrors `viewMinimise`.
fn minimise(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> (f64, f64, End) {
    let (mut lo, mut hi) = (a, b);
    let mut c = hi - VIEW_GOLDEN * (hi - lo);
    let mut d = lo + VIEW_GOLDEN * (hi - lo);
    let (mut fc, mut fd) = (f(c), f(d));
    for _ in 0..VIEW_GOLDEN_STEPS {
        if fc < fd {
            hi = d;
            d = c;
            fd = fc;
            c = hi - VIEW_GOLDEN * (hi - lo);
            fc = f(c);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + VIEW_GOLDEN * (hi - lo);
            fd = f(d);
        }
    }
    let (mut t, mut psi, mut end) = if fc < fd {
        (c, fc, End::Interior)
    } else {
        (d, fd, End::Interior)
    };
    let (fa, fb) = (f(a), f(b));
    if fb <= psi {
        (t, psi, end) = (b, fb, End::B);
    }
    if fa <= psi {
        (t, psi, end) = (a, fa, End::A);
    }
    (t, psi, end)
}

/// The entry on [lo, hi], where f(lo) > 0 ≥ f(hi). Mirrors the bisection in `viewTrace`.
fn bisect(f: &impl Fn(f64) -> f64, lo: f64, hi: f64) -> f64 {
    let (mut lo, mut hi) = (lo, hi);
    for _ in 0..VIEW_BISECT_STEPS {
        let mid = 0.5 * (lo + hi);
        if f(mid) <= 0. {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// What one ray finds (§4.1 "Kinds of hit", §4.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trace {
    pub hit: bool,
    /// Shade the point as the wall: the entry (or, on a miss, the minimiser) lies below the rim
    /// and the ray is outside at the rim depth.
    pub wall: bool,
    /// The shading point's depth parameter: the entry on a hit, the minimiser on a miss.
    pub t: f64,
    /// Ψmin over every piece, its depth parameter, its end plane's normal (z for the face and
    /// back planes and for an interior minimiser), and the quadrant it was evaluated in.
    pub psi_min: f64,
    pub min_t: f64,
    pub min_plane: DVec3,
    pub min_sel: DVec2,
}

/// Brackets the ray's entry into the slab (§4.1 "Bracketing"). Mirrors `viewTrace`.
pub fn trace(solid: &Solid, seg: &Segment) -> Trace {
    let thickness = solid.thickness;
    let mut best = Trace {
        hit: false,
        wall: false,
        t: 0.,
        psi_min: f64::INFINITY,
        min_t: 0.,
        min_plane: DVec3::Z,
        min_sel: DVec2::ZERO,
    };
    let mut entry = None;
    for piece in pieces(solid, seg) {
        let f = |t: f64| solid.psi(seg.point(t, thickness), piece.sel);
        let (t, psi, end) = minimise(&f, piece.a, piece.b);
        if psi < best.psi_min {
            best.psi_min = psi;
            best.min_t = t;
            best.min_sel = piece.sel;
            best.min_plane = match end {
                End::A => piece.na,
                End::B => piece.nb,
                End::Interior => DVec3::Z,
            };
        }
        if entry.is_none() && psi <= 0. {
            let at = if f(piece.a) <= 0. {
                piece.a
            } else {
                bisect(&f, piece.a, t)
            };
            entry = Some((at, piece.sel));
        }
    }
    best.hit = entry.is_some();
    let (t, sel) = entry.unwrap_or((best.min_t, best.min_sel));
    let rim = if thickness > 0. {
        (solid.rise() / thickness).min(1.)
    } else {
        0.
    };
    best.wall = t > rim && solid.psi(seg.point(rim, thickness), sel) > 0.;
    best.t = t;
    best
}

#[cfg(test)]
pub(super) mod tests {
    use std::f64::consts::PI;
    use std::time::Duration;

    use super::*;
    use crate::render_helpers::material::bevel::Slab;
    use crate::render_helpers::material::ring::fit_radii;
    use crate::render_helpers::material::view::tests::uniforms;
    use crate::render_helpers::material::view::{wall_faces_camera, ViewPose};

    /// The entry depth's tolerance, logical px: §4.1's 1/64 physical px at scale 16, so it holds
    /// at every scale up to 16.
    const ENTRY_DEPTH_TOL: f64 = 1. / 1024.;

    /// A slab at the origin of `half` with face radii `face_r`, fitted as `slabSurface` fits
    /// them, and its camera for `pose`.
    pub(in crate::render_helpers::material::view) fn setup(
        half: DVec2,
        chamfer: f64,
        face_r: Radii,
        thickness: f64,
        profile: f64,
        pose: &ViewPose,
    ) -> (Solid, Camera) {
        let chamfer = chamfer.min((half.x.min(half.y) - 1.).max(0.));
        let face_half = half - DVec2::splat(chamfer);
        let fitted = fit_radii(face_r, [face_half.x, face_half.y]);
        let solid = Solid {
            center: DVec2::ZERO,
            half,
            outer_r: fitted.map(|r| r + chamfer),
            chamfer,
            thickness,
            profile,
        };
        (solid, Camera::new(pose, -half, 2. * half))
    }

    fn peak(peak_deg: f64, dir: DVec2, kappa: f64) -> ViewPose {
        ViewPose {
            theta: peak_deg.to_radians(),
            dir,
            weight: 1.,
            kappa,
        }
    }

    /// The silhouette point along `angle` from the centre, at rest.
    fn silhouette(solid: &Solid, angle: f64) -> DVec2 {
        let u = DVec2::new(angle.cos(), angle.sin());
        let (mut lo, mut hi) = (0., solid.half.length() + 1.);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if solid.sigma(u * mid, u * mid) <= 0. {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        u * lo
    }

    /// What the certified reference says of one ray.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(in crate::render_helpers::material::view) enum Certified {
        /// The entry's depth parameter, bisected to 1e-15 of the piece.
        Hit(f64),
        Miss,
        /// Neither proved within the refinement budget: a ray tangent to the solid.
        Unresolved,
    }

    /// The first entry, certified by convexity rather than by sampling alone. Ψ is convex on
    /// each piece (`bracketing_matches_the_certified_intersection` asserts it), so:
    /// - a sample at or below 0 proves a hit, and the entry lies between it and the last
    ///   positive sample before it (the sublevel set of a convex function is an interval);
    /// - on an interval between samples, the chords through the two samples on either side,
    ///   extended across it, bound a convex function from below; where both stay above 0
    ///   the interval holds no hit.
    ///
    /// Intervals neither cleared nor hit are halved, up to 48 times; what is left is
    /// unresolved. A thin dip between two samples is therefore found or reported, never
    /// passed over.
    pub(in crate::render_helpers::material::view) fn certified_entry(
        solid: &Solid,
        seg: &Segment,
    ) -> Certified {
        for piece in pieces(solid, seg) {
            let f = |t: f64| solid.psi(seg.point(t, solid.thickness), piece.sel);
            match certify(&f, piece.a, piece.b) {
                Certified::Miss => continue,
                other => return other,
            }
        }
        Certified::Miss
    }

    fn certify(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> Certified {
        let mut pts: Vec<(f64, f64)> = (0..=64)
            .map(|i| a + (b - a) * f64::from(i) / 64.)
            .map(|t| (t, f(t)))
            .collect();
        for _ in 0..48 {
            if let Some(i) = pts.iter().position(|&(_, v)| v <= 0.) {
                if i == 0 {
                    return Certified::Hit(a);
                }
                let (mut lo, mut hi) = (pts[i - 1].0, pts[i].0);
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    if f(mid) <= 0. {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                return Certified::Hit(hi);
            }
            let chord = |i: usize, j: usize, t: f64| {
                let ((ti, fi), (tj, fj)) = (pts[i], pts[j]);
                fi + (fj - fi) * (t - ti) / (tj - ti)
            };
            let mut next = vec![pts[0]];
            let mut open = false;
            for i in 0..pts.len() - 1 {
                let (l, r) = (pts[i].0, pts[i + 1].0);
                // Lazy: `then_some` would evaluate `i - 1` at i = 0 and underflow.
                let left = (i >= 1).then(|| (i - 1, i));
                let right = (i + 2 < pts.len()).then(|| (i + 1, i + 2));
                let bound = |t: f64| {
                    let mut v = f64::NEG_INFINITY;
                    if let Some((p, q)) = left {
                        v = v.max(chord(p, q, t));
                    }
                    if let Some((p, q)) = right {
                        v = v.max(chord(p, q, t));
                    }
                    v
                };
                // The max of two lines is lowest at an end or where they cross.
                let mut low = bound(l).min(bound(r));
                if let (Some((p, q)), Some((u, w))) = (left, right) {
                    let (s1, s2) = (
                        (pts[q].1 - pts[p].1) / (pts[q].0 - pts[p].0),
                        (pts[w].1 - pts[u].1) / (pts[w].0 - pts[u].0),
                    );
                    if s1 != s2 {
                        let t = (pts[u].1 - s2 * pts[u].0 - pts[p].1 + s1 * pts[p].0) / (s1 - s2);
                        if t > l && t < r {
                            low = low.min(bound(t));
                        }
                    }
                }
                if low <= 0. {
                    open = true;
                    let mid = 0.5 * (l + r);
                    next.push((mid, f(mid)));
                }
                next.push(pts[i + 1]);
            }
            if !open {
                return Certified::Miss;
            }
            pts = next;
        }
        Certified::Unresolved
    }

    #[test]
    fn the_certifier_proves_misses_and_hits_on_known_functions() {
        // Plan review round 2: a function positive everywhere reaches the chord bounds on the
        // first pass, so a miss must come back certified, not panic.
        assert_eq!(certify(&|t: f64| 1. + t * t, 0., 1.), Certified::Miss);
        assert_eq!(
            certify(&|t: f64| (t - 0.5).powi(2) + 1e-3, 0., 1.),
            Certified::Miss
        );
        // A dip below 0 narrower than the first sample spacing (1/64): found, and bisected.
        let dip = |t: f64| (t - 0.5004).abs() - 1e-4;
        match certify(&dip, 0., 1.) {
            Certified::Hit(t) => assert!((t - 0.5003).abs() < 1e-12, "entry {t}"),
            other => panic!("{other:?}"),
        }
        assert_eq!(
            certify(&|t: f64| t - 0.25, 0., 1.),
            Certified::Hit(0.),
            "inside at the start"
        );
        // Tangent to 0: neither proved.
        assert_eq!(
            certify(&|t: f64| (t - 0.3).powi(2), 0., 1.),
            Certified::Unresolved
        );
    }

    /// A random slab, look and frame from the configured ranges.
    fn random_case(rnd: &mut impl FnMut() -> f64) -> (Solid, Camera) {
        let half = DVec2::new(20. + rnd() * 780., 20. + rnd() * 580.);
        let chamfer = if rnd() < 0.15 { 0. } else { rnd() * 20. };
        let thickness = if rnd() < 0.15 { 0. } else { rnd() * 200. };
        let face_r: Radii = if rnd() < 0.3 {
            [88., 12., 12., 12.]
        } else {
            std::array::from_fn(|_| rnd() * 30.)
        };
        let profile = 1. + rnd() * 7.;
        let angle = rnd() * 2. * PI;
        let pose = ViewPose::at(
            1. + rnd() * 19.,
            Duration::from_millis(900),
            Duration::from_secs_f64(0.9 * (0.02 + rnd() * 0.96)),
            DVec2::new(angle.cos(), angle.sin()),
            1.5 + rnd() * 48.5,
        )
        .unwrap();
        setup(half, chamfer, face_r, thickness, profile, &pose)
    }

    /// Screen points within about a pixel of the projected outline.
    fn near_outline(solid: &Solid, camera: &Camera, rnd: &mut impl FnMut() -> f64) -> DVec2 {
        let edge = silhouette(solid, rnd() * 2. * PI);
        let z = -rnd() * solid.thickness;
        camera.to_screen(edge.extend(z)) + DVec2::new(rnd() * 3. - 1.5, rnd() * 3. - 1.5)
    }

    #[test]
    fn psi_is_the_resting_height_field() {
        let mut rnd = uniforms(10);
        for _ in 0..200 {
            let k = 1. + rnd() * 7.;
            let chamfer = 2. + rnd() * 18.;
            let thickness = rnd() * 200.;
            let half = DVec2::new(100., 80.);
            let face_r = rnd() * 30.;
            let (solid, _) = setup(
                half,
                chamfer,
                [face_r; 4],
                thickness,
                k,
                &peak(0., DVec2::X, 6.),
            );
            let slab = Slab::at_rest(half, chamfer, solid.outer_r[0] - chamfer, thickness, k);
            for _ in 0..200 {
                let p = DVec2::new(rnd() * 240. - 120., rnd() * 200. - 100.);
                let z = -rnd() * thickness;
                let surface = slab.surface(p);
                if surface.outer_dist > 0. {
                    assert!(
                        solid.psi(p.extend(z), p) > 0.,
                        "outside the silhouette at {p}"
                    );
                    continue;
                }
                let top = surface.height - thickness;
                if (z - top).abs() < 1e-6 {
                    continue;
                }
                assert_eq!(
                    solid.psi(p.extend(z), p) <= 0.,
                    z <= top,
                    "{p} z {z} top {top}"
                );
            }
        }
    }

    #[test]
    fn bracketing_matches_the_certified_intersection() {
        let mut rnd = uniforms(11);
        let (mut rays, mut later, mut skipped) = (0, 0, 0);
        for _ in 0..120 {
            let (solid, camera) = random_case(&mut rnd);
            for _ in 0..40 {
                let s = near_outline(&solid, &camera, &mut rnd);
                let Some(seg) = Segment::new(&camera, s) else {
                    panic!("a ray near the slab points up at {s}");
                };
                // Convexity of Ψ along every piece.
                for piece in pieces(&solid, &seg) {
                    let f = |t: f64| solid.psi(seg.point(t, solid.thickness), piece.sel);
                    for i in 1..63 {
                        let t = |j: usize| piece.a + (piece.b - piece.a) * j as f64 / 63.;
                        let second = f(t(i - 1)) + f(t(i + 1)) - 2. * f(t(i));
                        assert!(
                            second >= -1e-7 * (1. + f(t(i)).abs()),
                            "not convex: {second}"
                        );
                    }
                }
                let got = trace(&solid, &seg);
                assert!(got.psi_min.is_finite() && got.t.is_finite(), "NaN at {s}");
                let exact = match certified_entry(&solid, &seg) {
                    Certified::Unresolved => {
                        skipped += 1;
                        continue;
                    }
                    Certified::Hit(t) => Some(t),
                    Certified::Miss => None,
                };
                rays += 1;
                assert_eq!(got.hit, exact.is_some(), "hit or miss at {s}: {got:?}");
                if let Some(t) = exact {
                    let depth = (got.t - t).abs() * solid.thickness;
                    assert!(
                        depth <= ENTRY_DEPTH_TOL,
                        "entry depth off by {depth} px at {s}"
                    );
                    if pieces(&solid, &seg).len() > 1 && t > pieces(&solid, &seg)[0].b {
                        later += 1;
                    }
                }
            }
        }
        assert!(
            skipped * 100 < rays,
            "{skipped} unresolved (tangent) rays of {rays}"
        );
        assert!(later > 0, "no ray's first hit lay in a later piece");
    }

    #[test]
    fn asymmetric_radii_find_the_hit_in_a_later_piece() {
        // (88, 12, 12, 12) on a 100 px slab (§4.1 "Quadrant seams"): at y = −49 the big
        // top-left corner leaves x < 0 outside (σ ≈ +6.9) while x > 0 is inside (σ ≈ −1). In the
        // overshoot the rays drift toward +x as they descend, so a ray that starts at x < 0 can
        // enter only after it crosses x = 0, in its second piece.
        let pose = ViewPose {
            theta: -10f64.to_radians(),
            ..peak(10., DVec2::X, 6.)
        };
        let (solid, camera) = setup(DVec2::splat(50.), 0., [88., 12., 12., 12.], 31.2, 2., &pose);
        let mut found = 0;
        for i in 0..4000 {
            let s = DVec2::new(-8. + i as f64 * 0.002, -49.);
            let seg = Segment::new(&camera, s).unwrap();
            let ps = pieces(&solid, &seg);
            let got = trace(&solid, &seg);
            let exact = match certified_entry(&solid, &seg) {
                Certified::Unresolved => continue,
                Certified::Hit(t) => Some(t),
                Certified::Miss => None,
            };
            assert_eq!(got.hit, exact.is_some(), "at {s}");
            if let Some(t) = exact {
                assert!(
                    (got.t - t).abs() * solid.thickness <= ENTRY_DEPTH_TOL,
                    "at {s}"
                );
                if ps.len() > 1 && t >= ps[1].a {
                    found += 1;
                }
            }
        }
        assert!(found > 0, "no ray entered through the second piece");
    }

    #[test]
    fn zero_rise_is_the_box_or_the_plane() {
        let mut rnd = uniforms(12);
        for (chamfer, thickness) in [(0., 31.2), (12., 0.), (0., 0.)] {
            let (solid, camera) = setup(
                DVec2::new(200., 150.),
                chamfer,
                [8.; 4],
                thickness,
                2.,
                &peak(14., DVec2::new(0.6, 0.8), 2.5),
            );
            for _ in 0..2000 {
                let s = near_outline(&solid, &camera, &mut rnd);
                let seg = Segment::new(&camera, s).unwrap();
                let got = trace(&solid, &seg);
                assert!(got.psi_min.is_finite() && got.t.is_finite(), "NaN at {s}");
                if thickness == 0. {
                    let sigma = solid.sigma(seg.h0.truncate(), seg.h0.truncate());
                    assert_eq!(got.hit, sigma <= 0.);
                    assert_eq!((got.t, got.wall), (0., false), "a plane has no wall");
                } else {
                    match certified_entry(&solid, &seg) {
                        Certified::Hit(_) => assert!(got.hit, "box at {s}"),
                        Certified::Miss => assert!(!got.hit, "box at {s}"),
                        Certified::Unresolved => {}
                    }
                }
            }
        }
    }

    /// The entry of an affine Ψ(t) = a + b t on [lo, hi], if any.
    fn affine_entry(a: f64, b: f64, lo: f64, hi: f64) -> Option<f64> {
        let at = |t: f64| a + b * t;
        if at(lo) <= 0. {
            Some(lo)
        } else if b < 0. && at(hi) <= 0. {
            Some(-a / b)
        } else {
            None
        }
    }

    #[test]
    fn analytic_entries_match_closed_form() {
        // Profile 1 is a planar chamfer, X(y) = y: along the row through a straight side's
        // middle, σ = x − half.x and the bevel term are both affine in t, with one kink at the
        // rim depth, so the entry has a closed form. Bevel 0 is a box. Both are analytic
        // controls for the mirror and for the certified reference.
        let mut checked = 0;
        for (chamfer, thickness, deg, kappa) in [
            (12., 31.2, 10., 6.),
            (0., 31.2, 14., 2.5),
            (20., 10., 20., 1.5),
            (12., 31.2, -10., 6.),
        ] {
            let pose = ViewPose {
                theta: f64::to_radians(deg),
                ..peak(10., DVec2::X, kappa)
            };
            let (solid, camera) = setup(
                DVec2::new(400., 300.),
                chamfer,
                [0.; 4],
                thickness,
                1.,
                &pose,
            );
            let rise = solid.rise();
            for i in 0..4000 {
                let s = DVec2::new(370. + f64::from(i) * 0.015, 0.);
                let seg = Segment::new(&camera, s).unwrap();
                // x(t) = h0.x + ρ.x T t; above the rim the bevel adds c (1 − T t / rise).
                let (x0, dx) = (seg.h0.x - solid.half.x, seg.rho.x * thickness);
                let rim = if rise > 0. {
                    (rise / thickness).min(1.)
                } else {
                    0.
                };
                let exact = if rise > 0. {
                    affine_entry(x0 + chamfer, dx - chamfer * thickness / rise, 0., rim)
                } else {
                    None
                }
                .or_else(|| affine_entry(x0, dx, rim, 1.));
                let got = trace(&solid, &seg);
                assert_eq!(
                    got.hit,
                    exact.is_some(),
                    "{chamfer} {thickness} {deg}° at {s}: {got:?}"
                );
                if let Some(t) = exact {
                    assert!(
                        (got.t - t).abs() * thickness <= ENTRY_DEPTH_TOL,
                        "{deg}° at {s}: {} vs {t}",
                        got.t
                    );
                    match certified_entry(&solid, &seg) {
                        Certified::Hit(c) => {
                            assert!((c - t).abs() * thickness <= 1e-9, "the reference at {s}")
                        }
                        other => panic!("the reference says {other:?} at {s}"),
                    }
                    checked += 1;
                } else {
                    assert!(
                        !matches!(certified_entry(&solid, &seg), Certified::Hit(_)),
                        "the reference hits at {s}"
                    );
                }
            }
        }
        assert!(checked > 1000, "{checked} analytic hits");
    }

    #[test]
    fn the_rim_graze_ray_hits_where_a_fixed_march_misses() {
        // §4.1: a peak ray of an 800 px slab at 10°, κ 6, profile 2, bevel 12, thickness 31.2 is
        // inside only between depths −8.93 and −9.02 px on the receding edge.
        let (solid, camera) = setup(
            DVec2::new(400., 300.),
            12.,
            [0.; 4],
            31.2,
            2.,
            &peak(10., DVec2::X, 6.),
        );
        let s = DVec2::new(-385.69676, 0.);
        let seg = Segment::new(&camera, s).unwrap();
        let got = trace(&solid, &seg);
        assert!(got.hit, "{got:?}");
        let depth = got.t * solid.thickness;
        assert!((depth - 8.93).abs() < 0.01, "entry depth {depth}");
        let marched = (0..=64).any(|i| solid.psi(seg.point(i as f64 / 64., 31.2), s) <= 0.);
        assert!(!marched, "the negative control: a 64-step march must miss");
    }

    /// Wall hits along a screen line across the edge whose outward normal is `n0`, between the
    /// projections of its rim and back edges, at `step` px.
    fn wall_width(solid: &Solid, camera: &Camera, edge: DVec2, n0: DVec2, step: f64) -> f64 {
        let a = camera.to_screen(edge.extend(-solid.rise())) - n0 * 2.;
        let b = camera.to_screen(edge.extend(-solid.thickness)) + n0 * 2.;
        let len = (b - a).dot(n0).abs().max(4.);
        let n = (len / step) as usize;
        let walls = (0..n)
            .filter(|&i| {
                let s = a + n0 * (i as f64 * step) * (b - a).dot(n0).signum();
                Segment::new(camera, s).is_some_and(|seg| {
                    let t = trace(solid, &seg);
                    t.hit && t.wall
                })
            })
            .count();
        walls as f64 * step
    }

    #[test]
    // The table's 3.14 is a width in px, not π.
    #[allow(clippy::approx_constant)]
    fn peak_wall_widths_match_the_spec_table() {
        // §2.1: the active slab, thickness 31.2 and bevel 12, so a wall 19.2 px deep below the
        // rim, at the swing's peak.
        for (peak_deg, widths) in [(10., [0.96, 1.77, 2.40]), (14., [2.36, 3.14, 3.76])] {
            for (kappa, want) in [4., 6., 10.].into_iter().zip(widths) {
                let (solid, camera) = setup(
                    DVec2::new(400., 300.),
                    12.,
                    [0.; 4],
                    31.2,
                    2.,
                    &peak(peak_deg, DVec2::X, kappa),
                );
                let got = wall_width(&solid, &camera, DVec2::new(400., 0.), DVec2::X, 0.002);
                assert!(
                    (got - want).abs() <= 0.1,
                    "{peak_deg}° κ {kappa}: {got} px, want {want}"
                );
            }
        }
    }

    #[test]
    fn the_wall_rule_agrees_with_the_ray_cast() {
        let settle = Duration::from_millis(900);
        let half = DVec2::new(400., 300.);
        // (peak, κ, dir, rounded corner radius): every frame, no frame, and the corner cases.
        for (peak_deg, kappa, dir, radius) in [
            (10., 6., DVec2::X, 0.),
            (12., 2.5, DVec2::X, 0.),
            (11., 2.5, DVec2::X, 0.),
            (10., 6., DVec2::Y, 0.),
            (14., 6., DVec2::new(0.6, 0.8), 60.),
        ] {
            let mut shown = 0;
            for frame in 1..54 {
                let elapsed = Duration::from_secs_f64(frame as f64 / 60.);
                let Some(pose) = ViewPose::at(peak_deg, settle, elapsed, dir, kappa) else {
                    continue;
                };
                let (solid, camera) = setup(half, 12., [radius; 4], 31.2, 2., &pose);
                let s_hat = dir * pose.theta.signum();
                // The wall point whose outward normal is ŝ, and its distance from the pivot.
                let r = solid.outer_r[0];
                let core = (half - DVec2::splat(r)) * s_hat.signum();
                let edge = if radius > 0. {
                    core + s_hat * r
                } else {
                    half * s_hat.signum() * s_hat.abs().ceil()
                };
                let h = edge.dot(s_hat);
                let extent = dir.x.abs() * 2. * half.x + dir.y.abs() * 2. * half.y;
                let rule = wall_faces_camera(&pose, extent, s_hat, h);
                let width = wall_width(&solid, &camera, edge, s_hat, 1. / 256.);
                if pose.theta.abs() < 0.5f64.to_radians() {
                    continue;
                }
                assert_eq!(
                    width > 0.,
                    rule,
                    "{peak_deg}° κ {kappa} {dir} frame {frame}: width {width}"
                );
                shown += usize::from(rule);
            }
            let all = peak_deg.to_radians().sin() > 1. / (2. * kappa);
            let none = peak_deg.to_radians() <= 1. / (2. * kappa);
            if dir == DVec2::X && none {
                assert_eq!(shown, 0, "{peak_deg}° κ {kappa}: no frame may show a wall");
            }
            if dir == DVec2::X && all {
                assert!(
                    shown > 30,
                    "{peak_deg}° κ {kappa}: every moving frame shows a wall"
                );
            }
        }
    }
}
