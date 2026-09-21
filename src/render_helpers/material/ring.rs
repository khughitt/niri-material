//! The ring beam's geometry and envelope
//! (docs/specs/2026-09-19-ring-beam-design.md §1–§2). Pure: the tile asks
//! this for the perimeter and the frame values, the shader mirrors `comet`.

use std::f64::consts::{FRAC_PI_2, PI};
use std::time::Duration;

/// Gaussian half-width of the head, px.
pub const BEAM_HEAD_SIGMA: f32 = 20.;
/// Tail brightness just behind the head.
pub const BEAM_TAIL_START: f32 = 0.6;
/// Tail length as a fraction of the perimeter, and its cap in px.
pub const BEAM_TAIL_FRACTION: f64 = 0.25;
pub const BEAM_TAIL_MAX: f64 = 1200.;
/// Resting glow once the beam has passed.
pub const BEAM_REST: f32 = 0.2;
/// Edge spill onto the chamfer.
pub const BEAM_SPILL: f32 = 0.25;
/// Focus light scale (the sweep's 0.7).
pub const BEAM_BASE: f32 = 0.7;
/// Head fade in and out.
pub const BEAM_FADE: Duration = Duration::from_millis(300);
/// The splash envelope's shorter fade-in.
pub const BEAM_SPLASH_FADE: Duration = Duration::from_millis(100);
/// Backstop for a beam whose tile is never rendered (so no frame ever
/// computes its perimeter): the run is bounded per focus change regardless.
pub const BEAM_MAX_RUN: Duration = Duration::from_secs(120);
/// The envelope shape that ships; the other is kept for the sheets.
pub const BEAM_ENVELOPE: Envelope = Envelope::Plateau;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Envelope {
    /// Full brightness from fade-in to fade-out.
    Plateau,
    /// Brightest at launch, decaying over the run like a ripple.
    Splash,
}

/// The face the shader draws, jelly included: half extents and fitted radii
/// (top-left, top-right, bottom-right, bottom-left), centred at the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    pub half: [f64; 2],
    pub radii: [f64; 4],
}

/// Mirrors `fitRadii` in prelude.frag: scale all four radii by one factor
/// so no adjacent pair exceeds its edge.
pub fn fit_radii(r: [f64; 4], half: [f64; 2]) -> [f64; 4] {
    let edge = [2. * half[0], 2. * half[1]];
    let k = (edge[0] / (r[0] + r[1]).max(1e-6))
        .min(edge[0] / (r[3] + r[2]).max(1e-6))
        .min(edge[1] / (r[0] + r[3]).max(1e-6))
        .min(edge[1] / (r[1] + r[2]).max(1e-6))
        .min(1.);
    r.map(|x| x * k)
}

/// Mirrors `slabSurface` on the slab the frame actually renders: the face is
/// the slab minus the chamfer, scaled by the jelly resize; radii are fitted
/// against whichever box binds harder. `slab_size` and `chamfer` come from
/// the `MaterialFrame` (pixel-aligned, from the animated window size) and
/// `corner_radius` is the fitted radius the element receives, so this is
/// the shader's face, not a reconstruction of it.
pub fn face(
    slab_size: [f64; 2],
    chamfer: f64,
    corner_radius: [f64; 4],
    jelly_resize: [f64; 2],
) -> Face {
    let half_ext = [slab_size[0] / 2., slab_size[1] / 2.];
    let inner = [half_ext[0] - chamfer, half_ext[1] - chamfer];
    let half = [
        inner[0] * (1. + jelly_resize[0] / (2. * half_ext[0]).max(1.)),
        inner[1] * (1. + jelly_resize[1] / (2. * half_ext[1]).max(1.)),
    ];
    let radius_half = [half[0].min(inner[0]), half[1].min(inner[1])];
    Face {
        half,
        radii: fit_radii(corner_radius, radius_half),
    }
}

fn beam_line(face: &Face, gap: f64) -> Option<([f64; 2], [f64; 4])> {
    let half = [face.half[0] - gap, face.half[1] - gap];
    if half[0] <= 0. || half[1] <= 0. {
        return None;
    }
    Some((half, face.radii.map(|r| (r - gap).max(0.))))
}

/// Perimeter of the beam line; 0 when the face is too small to hold one.
pub fn beam_perimeter(face: &Face, gap: f64) -> f64 {
    let Some((half, r)) = beam_line(face, gap) else {
        return 0.;
    };
    4. * (half[0] + half[1]) - (2. - FRAC_PI_2) * r.iter().sum::<f64>()
}

pub fn tail_length(perimeter: f64) -> f64 {
    (perimeter * BEAM_TAIL_FRACTION).min(BEAM_TAIL_MAX)
}

/// Distance along the beam line of the point nearest `q` (relative to the
/// face centre, y down), clockwise from the end of the top-left arc.
/// Mirrors `arcPosition` in prelude.frag; a projection, valid for any `q`.
pub fn arc_position(face: &Face, gap: f64, q: [f64; 2]) -> f64 {
    let Some((h, r)) = beam_line(face, gap) else {
        return 0.;
    };
    let [tl, tr, br, bl] = r;
    let top = 2. * h[0] - tl - tr;
    let right = 2. * h[1] - tr - br;
    let bottom = 2. * h[0] - br - bl;
    let left = 2. * h[1] - bl - tl;
    let after_top = top + tr * FRAC_PI_2;
    let after_right = after_top + right + br * FRAC_PI_2;
    let after_bottom = after_right + bottom + bl * FRAC_PI_2;
    let (x, y) = (q[0], q[1]);
    if x > h[0] - tr && y < -h[1] + tr {
        let a = (y + h[1] - tr).atan2(x - h[0] + tr).clamp(-FRAC_PI_2, 0.);
        return top + tr * (a + FRAC_PI_2);
    }
    if x > h[0] - br && y > h[1] - br {
        let a = (y - h[1] + br).atan2(x - h[0] + br).clamp(0., FRAC_PI_2);
        return after_top + right + br * a;
    }
    if x < -h[0] + bl && y > h[1] - bl {
        let a = (y - h[1] + bl).atan2(x + h[0] - bl).clamp(FRAC_PI_2, PI);
        return after_right + bottom + bl * (a - FRAC_PI_2);
    }
    if x < -h[0] + tl && y < -h[1] + tl {
        let mut a = (y + h[1] - tl).atan2(x + h[0] - tl);
        if a > 0. {
            a -= 2. * PI;
        }
        let a = a.clamp(-PI, -FRAC_PI_2);
        return after_bottom + left + tl * (a + PI);
    }
    let to_edge = [h[0] - x.abs(), h[1] - y.abs()];
    if to_edge[1] <= to_edge[0] {
        if y < 0. {
            return (x + h[0] - tl).clamp(0., top);
        }
        return after_right + (h[0] - br - x).clamp(0., bottom);
    }
    if x > 0. {
        return after_top + (y + h[1] - tr).clamp(0., right);
    }
    // The left edge starts where the bottom-left arc ends, at y = h − bl.
    after_bottom + (h[1] - bl - y).clamp(0., left)
}

/// What the shader needs for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamFrame {
    /// Head arc position, px; unbounded during a run.
    pub head: f32,
    /// Head cutoff, 0–1; exactly 0 once the head has completed its lap.
    pub env: f32,
    /// Shared brightness of head and tail, 0–1.
    pub decay: f32,
}

impl BeamFrame {
    /// The resting state (spec §1.3): no head, no tail, only the glow.
    pub const REST: Self = Self {
        head: 0.,
        env: 0.,
        decay: 0.,
    };
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    if e1 <= e0 {
        return if x >= e1 { 1. } else { 0. };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// The frame values at `elapsed` for a beam of `speed` on a `perimeter`.
/// `env` is exactly zero from the lap's end on, whatever the envelope does.
pub fn beam_frame(elapsed: Duration, speed: f64, perimeter: f64, envelope: Envelope) -> BeamFrame {
    let t = elapsed.as_secs_f64();
    let lap = perimeter / speed;
    let run = (perimeter + tail_length(perimeter)) / speed;
    let fade_in = match envelope {
        Envelope::Plateau => BEAM_FADE,
        Envelope::Splash => BEAM_SPLASH_FADE,
    }
    .as_secs_f64();
    let fade = BEAM_FADE.as_secs_f64();
    let env = if t >= lap {
        0.
    } else {
        smoothstep(0., fade_in, t) * (1. - smoothstep(lap - fade, lap, t))
    };
    let decay = match envelope {
        Envelope::Plateau => 1.,
        Envelope::Splash => {
            let x = (1. - t / run).clamp(0., 1.);
            x * x
        }
    };
    BeamFrame {
        head: (speed * t) as f32,
        env: env as f32,
        decay: decay as f32,
    }
}

/// The head at arc position `s`: a Gaussian of the circular distance to
/// `head`, gated by the cutoff `env`. Mirrors main.frag.
pub fn head_term(s: f32, head: f32, env: f32, perimeter: f32) -> f32 {
    let sigma = BEAM_HEAD_SIGMA;
    let d = (head - s + perimeter / 2.).rem_euclid(perimeter) - perimeter / 2.;
    env * (-d * d / (2. * sigma * sigma)).exp()
}

/// The tail `behind` px behind the head, of length `tail`: zero at the
/// head, zero with zero slope at `tail`. Every factor is evaluated inside
/// the branch so nothing overflows at rest. Mirrors main.frag.
pub fn tail_term(behind: f32, tail: f32) -> f32 {
    if behind > 0. && behind < tail {
        let taper = 1. - behind / tail;
        BEAM_TAIL_START
            * (-behind / tail).exp()
            * (1. - (-behind / BEAM_HEAD_SIGMA).exp())
            * taper
            * taper
    } else {
        0.
    }
}

/// The focus light at arc position `s`: the shader's formula in `f32`, so
/// its constants are tested here.
pub fn comet(s: f32, frame: BeamFrame, perimeter: f32, glow: f32) -> f32 {
    let tail = tail_length(f64::from(perimeter)) as f32;
    let moving = head_term(s, frame.head, frame.env, perimeter) + tail_term(frame.head - s, tail);
    BEAM_BASE * glow * (frame.decay * moving + BEAM_REST)
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};
    use std::time::Duration;

    use super::*;

    const HALF: f64 = 2. - FRAC_PI_2;

    fn plain(w: f64, h: f64, r: f64) -> Face {
        Face {
            half: [w / 2., h / 2.],
            radii: [r; 4],
        }
    }

    #[test]
    fn perimeter_of_a_rounded_box_and_its_degenerate_cases() {
        let f = plain(200., 100., 10.);
        assert!((beam_perimeter(&f, 0.) - (4. * 150. - HALF * 40.)).abs() < 1e-9);
        assert!((beam_perimeter(&plain(200., 100., 0.), 0.) - 600.).abs() < 1e-9);
        // a gap larger than the radius floors the radius at 0: the line is a plain box
        assert!((beam_perimeter(&plain(200., 100., 10.), 12.) - (4. * (88. + 38.))).abs() < 1e-9);
        // a face narrower than 2·gap has no beam line
        assert_eq!(beam_perimeter(&plain(200., 14., 8.), 8.), 0.);
        assert_eq!(beam_perimeter(&plain(200., 16., 8.), 8.), 0.);
    }

    #[test]
    fn the_face_mirrors_slab_surface_with_and_without_jelly() {
        // slab 112 (a 100 px window, bevel 12, offset 6 — prelude.frag's example),
        // chamfer 12: inner face 88
        let f = face([112., 112.], 12., [8.; 4], [0., 0.]);
        assert_eq!(f.half, [44., 44.]);
        assert_eq!(f.radii, [8.; 4]);
        // jelly resize +8 on x: inner_half.x · (1 + 8 / 112)
        let j = face([112., 112.], 12., [8.; 4], [8., 0.]);
        assert!((j.half[0] - 44. * (1. + 8. / 112.)).abs() < 1e-9);
        assert_eq!(j.half[1], 44.);
        // radii are fitted against min(inner_half, half_ext − chamfer), never past it
        let big = face([52., 52.], 12., [30.; 4], [0., 0.]);
        assert!(
            big.radii.iter().all(|r| (*r - 14.).abs() < 1e-9),
            "{:?}",
            big.radii
        );
        // no chamfer: the face is the slab
        let flat = face([200., 100.], 0., [10.; 4], [0., 0.]);
        assert_eq!(flat.half, [100., 50.]);
    }

    #[test]
    fn arc_position_walks_clockwise_from_the_top_left_arc() {
        let f = plain(200., 100., 10.);
        let gap = 0.;
        let p = beam_perimeter(&f, gap);
        let top = 200. - 20.;
        let right = 100. - 20.;
        // start of the top edge, its middle, its end
        assert!((arc_position(&f, gap, [-90., -50.]) - 0.).abs() < 1e-9);
        assert!((arc_position(&f, gap, [0., -50.]) - 90.).abs() < 1e-9);
        assert!((arc_position(&f, gap, [90., -50.]) - top).abs() < 1e-9);
        // top-right arc midpoint: 45° into a 10 px quarter arc
        let c = [90., -40.];
        let a = -FRAC_PI_2 / 2.;
        let q = [c[0] + 10. * a.cos(), c[1] + 10. * a.sin()];
        assert!((arc_position(&f, gap, q) - (top + 10. * FRAC_PI_2 / 2.)).abs() < 1e-6);
        // right edge middle, bottom edge middle (right→left), left edge middle (bottom→top)
        assert!(
            (arc_position(&f, gap, [100., 0.]) - (top + 10. * FRAC_PI_2 + right / 2.)).abs() < 1e-9
        );
        let after_right = top + 10. * FRAC_PI_2 + right + 10. * FRAC_PI_2;
        assert!((arc_position(&f, gap, [0., 50.]) - (after_right + top / 2.)).abs() < 1e-9);
        let after_bottom = after_right + top + 10. * FRAC_PI_2;
        assert!((arc_position(&f, gap, [-100., 0.]) - (after_bottom + right / 2.)).abs() < 1e-9);
        // the top-left arc ends at −π/2, where s reaches P (or wraps to 0)
        let c = [-90., -40.];
        let a = -FRAC_PI_2 + 1e-7;
        let q = [c[0] + 10. * a.cos(), c[1] + 10. * a.sin()];
        let s_end = arc_position(&f, gap, q);
        assert!(
            (p - s_end).abs() < 1e-3 || s_end.abs() < 1e-3,
            "{s_end} vs {p}"
        );
        // the top-left arc *starts* at −π, a quarter arc before the seam
        let a = -PI + 1e-7;
        let q = [c[0] + 10. * a.cos(), c[1] + 10. * a.sin()];
        assert!((arc_position(&f, gap, q) - (p - 10. * FRAC_PI_2)).abs() < 1e-3);
        // a point inside the face projects to its nearest edge
        assert!((arc_position(&f, gap, [0., -30.]) - 90.).abs() < 1e-9);
    }

    #[test]
    fn unequal_corners_keep_every_segment_boundary_continuous() {
        // TL 10, TR 20, BR 30, BL 40 on a 200×100 face
        let f = Face {
            half: [100., 50.],
            radii: [10., 20., 30., 40.],
        };
        let gap = 0.;
        let p = beam_perimeter(&f, gap);
        let step = 1e-4;
        // walk each boundary from both sides: the two samples must be a hair apart
        let boundaries: [([f64; 2], [f64; 2]); 8] = [
            // top edge end → TR arc start
            (
                [100. - 20. - step, -50.],
                [100. - 20. + step * 1e-3, -50. + 0.],
            ),
            // TR arc end → right edge start
            ([100., -50. + 20. - step], [100., -50. + 20. + step]),
            // right edge end → BR arc start
            ([100., 50. - 30. - step], [100., 50. - 30. + step * 1e-3]),
            // BR arc end → bottom edge start
            ([100. - 30. + step * 1e-3, 50.], [100. - 30. - step, 50.]),
            // bottom edge end → BL arc start
            ([-100. + 40. + step, 50.], [-100. + 40. - step * 1e-3, 50.]),
            // BL arc end → left edge start
            ([-100., 50. - 40. + step * 1e-3], [-100., 50. - 40. - step]),
            // left edge end → TL arc start
            (
                [-100., -50. + 10. + step],
                [-100., -50. + 10. - step * 1e-3],
            ),
            // TL arc end → seam
            (
                [-100. + 10. - step * 1e-3, -50.],
                [-100. + 10. + step, -50.],
            ),
        ];
        for (i, (a, b)) in boundaries.iter().enumerate() {
            let sa = arc_position(&f, gap, *a);
            let sb = arc_position(&f, gap, *b);
            let d = (sa - sb).abs().min((p - (sa - sb).abs()).abs());
            assert!(d < 1e-2, "boundary {i}: {sa} vs {sb} (P = {p})");
        }
        // the left edge is measured from the *bottom-left* arc's end: its
        // midpoint is P − tl·π/2 − left/2 with left = 100 − bl − tl
        let left = 100. - 40. - 10.;
        assert!(
            (arc_position(&f, gap, [-100., (10. - 40.) / 2.]) - (p - 10. * FRAC_PI_2 - left / 2.))
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn tail_length_is_a_quarter_capped() {
        assert_eq!(tail_length(2000.), 500.);
        assert_eq!(tail_length(6000.), 1200.);
    }

    #[test]
    fn plateau_frame_fades_the_head_in_and_out_and_runs_until_the_tail_clears() {
        let p = 3000.;
        let v = 300.; // lap 10 s, tail 750 px → 2.5 s
        let at = |ms: u64| beam_frame(Duration::from_millis(ms), v, p, Envelope::Plateau);
        assert_eq!(at(0).head, 0.);
        assert_eq!(at(0).env, 0.);
        assert_eq!(at(0).decay, 1.);
        assert!((at(150).env - 0.5).abs() < 1e-6, "smoothstep midpoint");
        assert_eq!(at(5000).env, 1.);
        assert!((at(5000).head - 1500.).abs() < 1e-3);
        assert!(at(9850).env > 0.49 && at(9850).env < 0.51);
        assert_eq!(at(10_000).env, 0., "exactly zero at the seam");
        assert_eq!(at(11_000).env, 0., "and for the whole drain");
        assert!(
            (at(11_000).head - 3300.).abs() < 1e-3,
            "the head keeps counting through the drain"
        );
        assert_eq!(at(11_000).decay, 1.);
    }

    #[test]
    fn splash_frame_decays_head_and_tail_together_but_keeps_the_head_cutoff() {
        let p = 3000.;
        let v = 300.;
        let run = (p + tail_length(p)) / v; // 12.5 s
        let at = |ms: u64| beam_frame(Duration::from_millis(ms), v, p, Envelope::Splash);
        assert!((at(50).env - 0.5).abs() < 1e-6, "100 ms fade-in");
        assert!(
            (at(6250).decay - 0.25).abs() < 1e-6,
            "(1 − t/run)² at half the run"
        );
        assert_eq!(
            at(10_000).env,
            0.,
            "the head still ends its lap at the seam"
        );
        assert!(at(10_000).decay > 0., "while the tail is still draining");
        let end = Duration::from_secs_f64(run);
        assert!(beam_frame(end, v, p, Envelope::Splash).decay.abs() < 1e-6);
    }

    #[test]
    fn the_head_is_symmetric_across_the_seam() {
        let p = 4000.;
        assert!((head_term(0., 20., 1., p) - 0.607).abs() < 1e-3);
        assert!(
            (head_term(p - 20., 20., 1., p) - 0.135).abs() < 1e-3,
            "across the seam at launch"
        );
        assert!(
            (head_term(20., p - 20., 1., p) - 0.135).abs() < 1e-3,
            "across the seam on the return"
        );
        assert_eq!(head_term(0., 20., 0., p), 0., "env gates it");
    }

    #[test]
    fn the_tail_grows_out_of_the_head_and_tapers_to_zero_at_l() {
        let l = tail_length(4000.) as f32; // 1000
        assert_eq!(tail_term(0., l), 0., "no tail at behind = 0");
        assert_eq!(tail_term(-10., l), 0., "nothing ahead of the head");
        assert!(tail_term(300., l) > 0.1);
        assert_eq!(tail_term(l, l), 0., "zero at L");
        assert!(tail_term(l - 1., l) < 1e-5, "and flat there");
        assert!(tail_term(l + 1., l) == 0.);
    }

    #[test]
    fn comet_composes_head_tail_rest_and_decay() {
        let p = 4000.;
        let l = tail_length(p as f64) as f32;
        let f = BeamFrame {
            head: 20.,
            env: 1.,
            decay: 0.5,
        };
        let expected =
            BEAM_BASE * (0.5 * (head_term(0., 20., 1., p) + tail_term(20., l)) + BEAM_REST);
        assert!((comet(0., f, p, 1.) - expected).abs() < 1e-6);
        // at launch s = 0 sees head *and* the tail's first 20 px: about 0.964 before BASE/decay
        assert!(((head_term(0., 20., 1., p) + tail_term(20., l)) - 0.964).abs() < 2e-3);
    }

    #[test]
    fn rest_is_finite_in_f32_everywhere() {
        let p = 4000.;
        for s in [0., 1., 2000., p - 1., p] {
            let v = comet(s, BeamFrame::REST, p, 1.);
            assert!(v.is_finite(), "s = {s}: {v}");
            assert!((v - BEAM_BASE * BEAM_REST).abs() < 1e-6, "s = {s}: {v}");
        }
        // a head past the seam with decay still up contributes only its tail
        let drain = BeamFrame {
            head: p + 100.,
            env: 0.,
            decay: 1.,
        };
        assert!(
            (comet(50., drain, p, 1.) - BEAM_BASE * BEAM_REST).abs() < 1e-6,
            "no head at s = 50"
        );
        assert!(
            comet(p - 50., drain, p, 1.) > BEAM_BASE * BEAM_REST,
            "tail at s = P − 50"
        );
    }

    #[test]
    fn the_shader_names_the_same_constants() {
        let frag = include_str!("../shaders/material/main.frag");
        for (name, value) in [
            ("BEAM_HEAD_SIGMA", format!("{:.1}", BEAM_HEAD_SIGMA)),
            ("BEAM_TAIL_START", format!("{:.1}", BEAM_TAIL_START)),
            ("BEAM_TAIL_FRACTION", format!("{:.2}", BEAM_TAIL_FRACTION)),
            ("BEAM_TAIL_MAX", format!("{:.1}", BEAM_TAIL_MAX)),
            ("BEAM_REST", format!("{:.1}", BEAM_REST)),
            ("BEAM_SPILL", format!("{:.2}", BEAM_SPILL)),
            ("BEAM_BASE", format!("{:.1}", BEAM_BASE)),
        ] {
            let line = format!("const float {name} = {value};");
            assert!(frag.contains(&line), "main.frag lacks `{line}`");
        }
    }

    #[test]
    fn glow_scales_the_whole_focus_light() {
        let f = BeamFrame {
            head: 100.,
            env: 1.,
            decay: 1.,
        };
        assert!((comet(100., f, 4000., 2.) - 2. * comet(100., f, 4000., 1.)).abs() < 1e-6);
        assert!(
            (comet(3000., BeamFrame::REST, 4000., 0.5) - 0.5 * BEAM_BASE * BEAM_REST).abs() < 1e-7
        );
    }
}
