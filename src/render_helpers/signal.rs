//! Effective signal and the pure envelope solver (design §3, §4).

use std::f32::consts::TAU;
use std::time::Duration;

use niri_config::{Color, ResolvedResponse, SignalMotionPolicy};
use niri_ipc::{SignalLevel, SignalMotion};
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::render_helpers::material::ring::BeamFrame;
use crate::window::signal::{Folded, IMPULSE_LIFETIME};

pub const BREATHE_PERIOD: Duration = Duration::from_millis(4000);
pub const PULSE_PERIOD: Duration = Duration::from_millis(1200);
pub const FLASH_PERIOD: Duration = Duration::from_millis(500);
pub const FLASH_EDGE: Duration = Duration::from_millis(50);
const BUCKETS_PER_PERIOD: u32 = 32;
const FLASH_EDGE_SAMPLES: u32 = 4;
const ATTACK: Duration = Duration::from_millis(80);
const DECAY_TAU_SECS: f32 = 0.35;

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveImpulse {
    pub selector: u8,
    pub accent: Option<Color>,
    pub at: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveSignal {
    pub accent: Option<Color>,
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub impulses: Vec<EffectiveImpulse>,
}

impl EffectiveSignal {
    pub fn is_sustained(&self) -> bool {
        self.motion != SignalMotion::Static
    }

    pub fn has_live_impulses(&self, now: Duration) -> bool {
        self.impulses.iter().any(|i| now < i.at + IMPULSE_LIFETIME)
    }
}

/// Stage 1: apply the global policy, the response block, and the
/// input-activity gate (design 2026-09-18 §3): idle input makes sustained
/// motion static and touches nothing else.
pub fn effective(
    folded: &Folded,
    policy: SignalMotionPolicy,
    response: &ResolvedResponse,
    input_active: bool,
) -> EffectiveSignal {
    let motion = match (policy, folded.motion) {
        (SignalMotionPolicy::Off, _) => SignalMotion::Static,
        (SignalMotionPolicy::Reduced, SignalMotion::Flash) => SignalMotion::Pulse,
        (SignalMotionPolicy::Reduced, SignalMotion::Pulse) => SignalMotion::Breathe,
        (_, m) => m,
    };
    let motion = if response.attention_is_none() {
        SignalMotion::Static
    } else {
        motion
    };
    let motion = if input_active {
        motion
    } else {
        SignalMotion::Static
    };
    let impulses = folded
        .impulses
        .iter()
        .filter_map(|i| {
            let selector = response.impulse_selector(i.kind, policy)?;
            Some(EffectiveImpulse {
                selector,
                accent: i.accent,
                at: i.at,
            })
        })
        .collect();
    EffectiveSignal {
        accent: folded.accent,
        level: folded.level,
        motion,
        impulses,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ImpulseFrame {
    pub selector: u8,
    pub envelope: f32,
    pub progress: f32,
    pub accent: Option<[f32; 3]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignalFrame {
    /// Straight (unpremultiplied) accent color; `presence` says how much of it shows.
    pub accent: Option<[f32; 3]>,
    pub level: f32,
    pub breath: f32,
    pub impulses: [ImpulseFrame; 4],
    /// Crossfaded accent presence, 0 to 1.
    pub presence: f32,
    /// Crossfaded focus, 0 to 1.
    pub focus: f32,
    /// Beam head, cutoff and decay for this frame; `BeamFrame::REST` at rest.
    pub beam: BeamFrame,
}

/// The tile's crossfaded values and beam frame for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInputs {
    pub level: f32,
    pub accent: Option<[f32; 3]>,
    pub presence: f32,
    pub focus: f32,
    /// Beam head, cutoff and decay for this frame; `BeamFrame::REST` at rest.
    pub beam: BeamFrame,
}

impl FrameInputs {
    pub fn quiet() -> Self {
        Self {
            level: 0.,
            accent: None,
            presence: 0.,
            focus: 0.,
            beam: BeamFrame::REST,
        }
    }
}

pub fn level_value(level: SignalLevel) -> f32 {
    match level {
        SignalLevel::Quiet => 0.,
        SignalLevel::Active => 1. / 3.,
        SignalLevel::Notice => 2. / 3.,
        SignalLevel::Demand => 1.,
    }
}

/// sRGB config color to linear RGB, matching the shader's `srgbToLinear`.
pub fn color_linear(c: Color) -> [f32; 3] {
    let lin = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    [lin(c.r), lin(c.g), lin(c.b)]
}

pub fn envelope(age: Duration) -> f32 {
    if age >= IMPULSE_LIFETIME {
        return 0.;
    }
    if age < ATTACK {
        return age.as_secs_f32() / ATTACK.as_secs_f32();
    }
    (-(age - ATTACK).as_secs_f32() / DECAY_TAU_SECS).exp()
}

pub fn progress(age: Duration) -> f32 {
    (age.as_secs_f32() / IMPULSE_LIFETIME.as_secs_f32()).min(1.)
}

fn period(motion: SignalMotion) -> Option<Duration> {
    match motion {
        SignalMotion::Static => None,
        SignalMotion::Breathe => Some(BREATHE_PERIOD),
        SignalMotion::Pulse => Some(PULSE_PERIOD),
        SignalMotion::Flash => Some(FLASH_PERIOD),
    }
}

/// Sub-period time as `f32`, taken with integer arithmetic first so the
/// value stays exact after days of uptime (a raw `as_secs_f32` loses the
/// millisecond digits past a few days).
fn in_period(now: Duration, period: Duration) -> f32 {
    let nanos = now.as_nanos() % period.as_nanos();
    Duration::from_nanos(nanos as u64).as_secs_f32()
}

/// Oscillator value in [0, 1]. Breathe and Pulse are quantized to their
/// bucket so the value is constant between boundaries; Flash is a square
/// with 50 ms soft edges and a global phase.
pub fn breath(motion: SignalMotion, now: Duration, seed: f32) -> f32 {
    let Some(period) = period(motion) else {
        return 0.;
    };
    let p = period.as_secs_f32();
    let t = in_period(now, period);
    if motion == SignalMotion::Flash {
        let half = p / 2.;
        let e = FLASH_EDGE.as_secs_f32();
        let rising = (t / e).clamp(0., 1.);
        let falling = 1. - ((t - half) / e).clamp(0., 1.);
        return if t < half { rising } else { falling };
    }
    let bucket = p / BUCKETS_PER_PERIOD as f32;
    let t = (t / bucket).floor() * bucket;
    let phase = (t / p + seed) * TAU;
    0.5 - 0.5 * phase.cos()
}

/// Next absolute-clock instant at which `breath` changes (design §4).
pub fn next_boundary(motion: SignalMotion, now: Duration) -> Option<Duration> {
    let period = period(motion)?;
    if motion == SignalMotion::Flash {
        // Exactly four sample instants per edge: edge + 12.5, 25, 37.5, 50 ms.
        // The edge start itself is not a boundary: the value there equals
        // the preceding plateau. Search this period and the next.
        let half = period / 2;
        let sample = FLASH_EDGE / FLASH_EDGE_SAMPLES;
        let t = Duration::from_nanos((now.as_nanos() % period.as_nanos()) as u64);
        let base = now - t;
        let mut candidates = Vec::new();
        for start in [base, base + period] {
            for edge in [Duration::ZERO, half] {
                for k in 1..=FLASH_EDGE_SAMPLES {
                    candidates.push(start + edge + sample * k);
                }
            }
        }
        return candidates.into_iter().filter(|c| *c > now).min();
    }
    let bucket = period / BUCKETS_PER_PERIOD;
    let n = now.as_nanos() / bucket.as_nanos() + 1;
    Some(Duration::from_nanos((n * bucket.as_nanos()) as u64))
}

/// Effective drift rate: the response's rate under the motion policy and
/// the global animation switch (design: Inputs and motion).
pub fn drift_rate(hz: f64, policy: SignalMotionPolicy, animations_off: bool) -> f64 {
    if animations_off {
        return 0.;
    }
    match policy {
        SignalMotionPolicy::Off => 0.,
        SignalMotionPolicy::Reduced => hz / 2.,
        SignalMotionPolicy::Full => hz,
    }
}

/// Buckets per period for a rate: `hz` steps per second over the period,
/// rounded, at least one bucket. Config guarantees `hz == 0 || hz >= 1`
/// before the reduced-motion halving, so the 600 s aurora field has at
/// least 300.
fn buckets_on(period: Duration, hz: f64) -> Option<u128> {
    if hz <= 0. {
        return None;
    }
    Some(((hz * period.as_secs_f64()).round() as u128).max(1))
}

/// Phase in radians of a bucketed clock, constant within each bucket.
/// Buckets divide `period` evenly and are anchored to period starts on the
/// absolute clock, in integer nanoseconds, so no rounding accumulates over
/// uptime and every window on an output shares the same boundaries. The
/// seed offsets the phase, not the boundary. Pinned to 0 when the rate is
/// 0 so a static value fingerprints to a constant.
pub fn phase_on(period: Duration, hz: f64, now: Duration, seed: f32) -> f32 {
    let Some(n) = buckets_on(period, hz) else {
        return 0.;
    };
    let period = period.as_nanos();
    let k = (now.as_nanos() % period) * n / period;
    let phase = (k as f32 / n as f32 + seed).fract();
    (phase * TAU).rem_euclid(TAU)
}

/// Next absolute-clock instant at which `phase_on` changes: the first
/// nanosecond of the next bucket, so it is strictly in the future and
/// `phase_on` evaluated there is already the next bucket's value (ceiling
/// division; a floor would land one nanosecond early and re-arm the same
/// deadline). The `as u64` cast is exact below 584 years of uptime.
pub fn next_boundary_on(period: Duration, hz: f64, now: Duration) -> Option<Duration> {
    let n = buckets_on(period, hz)?;
    let period = period.as_nanos();
    let start = now.as_nanos() - now.as_nanos() % period;
    let k = (now.as_nanos() - start) * n / period + 1;
    Some(Duration::from_nanos(
        (start + (period * k).div_ceil(n)) as u64,
    ))
}

/// Whether a tile's slab band is in view. The band is the tile rect inflated
/// by `bevel` on every side: a superset of the exact slab, so a visible
/// band never freezes when the tile rect alone leaves view.
pub fn slab_in_view(
    location: Point<f64, Logical>,
    size: Size<f64, Logical>,
    bevel: f64,
    view: Rectangle<f64, Logical>,
) -> bool {
    let slab = Rectangle::new(
        location - Point::from((bevel, bevel)),
        Size::from((size.w + 2. * bevel, size.h + 2. * bevel)),
    );
    slab.overlaps(view)
}

/// Next redraw deadline for a tile: sustained signal motion, only while the
/// slab band is in view. The focus beam runs on the animation loop and has
/// no arm here (design 2026-09-18 §2).
pub fn tick_deadline(eff: &EffectiveSignal, in_view: bool, now: Duration) -> Option<Duration> {
    if !in_view || !eff.is_sustained() {
        return None;
    }
    next_boundary(eff.motion, now)
}

/// Stage 2: pure solve (design §3). `inputs` are the crossfaded values the
/// tile already computed plus the beam frame.
pub fn solve(e: &EffectiveSignal, now: Duration, seed: f32, inputs: FrameInputs) -> SignalFrame {
    let mut impulses = [ImpulseFrame::default(); 4];
    let live = e.impulses.iter().filter(|i| now < i.at + IMPULSE_LIFETIME);
    for (slot, i) in impulses.iter_mut().zip(live) {
        let age = now.saturating_sub(i.at);
        *slot = ImpulseFrame {
            selector: i.selector,
            envelope: envelope(age),
            progress: progress(age),
            accent: i.accent.map(color_linear),
        };
    }
    SignalFrame {
        accent: inputs.accent,
        level: inputs.level,
        breath: breath(e.motion, now, seed),
        impulses,
        presence: inputs.presence,
        focus: inputs.focus,
        beam: inputs.beam,
    }
}

/// Quantized frame for damage tracking (design §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalFingerprint {
    level_q: i32,
    accent_q: [i32; 3],
    breath_q: i32,
    impulses_q: [(u8, i32, i32, [i32; 3]); 4],
    presence_q: i32,
    focus_q: i32,
    head_q: i32,
    env_q: i32,
    decay_q: i32,
}

impl Default for SignalFingerprint {
    /// At rest: no accent (-1 sentinel), zero level and breath, no impulses.
    fn default() -> Self {
        Self {
            level_q: 0,
            accent_q: [-1; 3],
            breath_q: 0,
            impulses_q: [(0, 0, 0, [-1; 3]); 4],
            presence_q: 0,
            focus_q: 0,
            head_q: 0,
            env_q: 0,
            decay_q: 0,
        }
    }
}

impl SignalFingerprint {
    pub fn quantize(f: &SignalFrame) -> Self {
        let q256 = |v: f32| (v * 256.).round() as i32;
        let q128 = |v: f32| (v * 128.).round() as i32;
        let color = |c: Option<[f32; 3]>| c.map_or([-1; 3], |c| c.map(q256));
        let mut impulses_q = [(0u8, 0i32, 0i32, [-1i32; 3]); 4];
        for (dst, i) in impulses_q.iter_mut().zip(f.impulses.iter()) {
            *dst = if i.selector == 0 {
                (0, 0, 0, [-1; 3])
            } else {
                (
                    i.selector,
                    q128(i.envelope),
                    q128(i.progress),
                    color(i.accent),
                )
            };
        }
        Self {
            level_q: q256(f.level),
            accent_q: color(f.accent),
            breath_q: (f.breath * 32.).round() as i32,
            impulses_q,
            presence_q: q256(f.presence),
            focus_q: q256(f.focus),
            head_q: (f.beam.head * 2.).round() as i32,
            env_q: q256(f.beam.env),
            decay_q: q256(f.beam.decay),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::signal::{Folded, Impulse};
    use niri_config::{ResolvedResponse, SignalMotionPolicy as P};
    use niri_ipc::{ImpulseKind, SignalLevel as L, SignalMotion as M};

    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }

    fn folded(motion: M, impulses: Vec<Impulse>) -> Folded {
        Folded {
            level: L::Demand,
            motion,
            accent: None,
            tag: None,
            sources: vec!["a".into()],
            impulses,
        }
    }

    fn impulse(kind: ImpulseKind, at: Duration) -> Impulse {
        Impulse {
            source: "a".into(),
            kind,
            accent: None,
            at,
        }
    }

    #[test]
    fn policy_off_makes_motion_static_and_drops_impulses() {
        let r = ResolvedResponse::default();
        let e = effective(
            &folded(M::Pulse, vec![impulse(ImpulseKind::Done, ms(0))]),
            P::Off,
            &r,
            true,
        );
        assert_eq!(e.motion, M::Static);
        assert!(e.impulses.is_empty());
    }

    #[test]
    fn reduced_maps_flash_to_pulse_and_pulse_to_breathe() {
        let r = ResolvedResponse::default();
        assert_eq!(
            effective(&folded(M::Flash, vec![]), P::Reduced, &r, true).motion,
            M::Pulse
        );
        assert_eq!(
            effective(&folded(M::Pulse, vec![]), P::Reduced, &r, true).motion,
            M::Breathe
        );
    }

    #[test]
    fn attention_none_makes_motion_static() {
        let mut r = ResolvedResponse::default();
        r.attention = niri_config::AttentionResponse::None;
        assert_eq!(
            effective(&folded(M::Pulse, vec![]), P::Full, &r, true).motion,
            M::Static
        );
    }

    #[test]
    fn none_response_drops_impulse() {
        let mut r = ResolvedResponse::default();
        r.done = niri_config::ImpulseResponse::None;
        let e = effective(
            &folded(
                M::Static,
                vec![
                    impulse(ImpulseKind::Done, ms(0)),
                    impulse(ImpulseKind::Error, ms(1)),
                ],
            ),
            P::Full,
            &r,
            true,
        );
        assert_eq!(e.impulses.len(), 1);
        assert_eq!(
            e.impulses[0].selector,
            niri_config::ImpulseResponse::Flash as u8
        );
    }

    #[test]
    fn envelope_and_progress_shapes() {
        assert_eq!(envelope(ms(0)), 0.);
        assert!((envelope(ms(80)) - 1.).abs() < 1e-6);
        assert!(envelope(ms(430)) < envelope(ms(80)));
        assert_eq!(envelope(ms(1500)), 0.);
        assert_eq!(progress(ms(0)), 0.);
        assert!((progress(ms(750)) - 0.5).abs() < 1e-6);
        assert!(progress(ms(1500)) >= 1.);
    }

    #[test]
    fn breath_is_zero_when_static_and_bounded_otherwise() {
        assert_eq!(breath(M::Static, ms(123), 0.3), 0.);
        for t in (0..4000).step_by(50) {
            let b = breath(M::Breathe, ms(t), 0.3);
            assert!((0. ..=1.).contains(&b));
        }
    }

    #[test]
    fn flash_ignores_seed_and_breathe_uses_it() {
        assert_eq!(
            breath(M::Flash, ms(120), 0.),
            breath(M::Flash, ms(120), 0.7)
        );
        assert_ne!(
            breath(M::Breathe, ms(500), 0.),
            breath(M::Breathe, ms(500), 0.7)
        );
    }

    #[test]
    fn next_boundary_is_clock_aligned() {
        // Breathe period 4000 ms / 32 = 125 ms buckets, aligned to the absolute clock.
        assert_eq!(next_boundary(M::Breathe, ms(0)), Some(ms(125)));
        assert_eq!(next_boundary(M::Breathe, ms(130)), Some(ms(250)));
        assert_eq!(
            next_boundary(M::Pulse, ms(0)),
            Some(Duration::from_micros(37_500))
        );
        assert_eq!(next_boundary(M::Static, ms(10)), None);
        // Flash: 500 ms period, edges at 0 and 250, exactly 4 sample instants
        // per 50 ms edge, nothing on the plateaus: 16 boundaries per second.
        assert_eq!(
            next_boundary(M::Flash, ms(0)),
            Some(Duration::from_micros(12_500))
        );
        assert_eq!(
            next_boundary(M::Flash, ms(60)),
            Some(Duration::from_micros(262_500))
        );
        assert_eq!(
            next_boundary(M::Flash, ms(300)),
            Some(Duration::from_micros(512_500))
        );
        let mut t = Duration::ZERO;
        let mut seq = Vec::new();
        while let Some(n) = next_boundary(M::Flash, t) {
            if n >= Duration::from_secs(1) {
                break;
            }
            seq.push(n);
            t = n;
        }
        let expected: Vec<Duration> = [0u64, 250, 500, 750]
            .iter()
            .flat_map(|edge| (1..=4).map(move |k| Duration::from_micros(edge * 1000 + 12_500 * k)))
            .collect();
        assert_eq!(seq, expected);
    }

    #[test]
    fn slab_in_view_uses_the_bevel_band() {
        let view = Rectangle::new(Point::from((0., 0.)), Size::from((1920., 1080.)));
        let size = Size::from((400., 300.));
        assert!(slab_in_view(Point::from((10., 10.)), size, 12., view));
        // Tile rect fully left of the view, band still overlapping.
        assert!(slab_in_view(
            Point::from((-400. + 6., 10.)),
            size,
            12.,
            view
        ));
        // Band entirely out of view.
        assert!(!slab_in_view(
            Point::from((-400. - 20., 10.)),
            size,
            12.,
            view
        ));
    }

    #[test]
    fn tick_deadline_requires_sustained_motion_in_view() {
        let sustained = EffectiveSignal {
            accent: None,
            level: L::Demand,
            motion: M::Pulse,
            impulses: vec![],
        };
        assert!(tick_deadline(&sustained, true, ms(100)).is_some());
        assert!(tick_deadline(&sustained, false, ms(100)).is_none());
        let quiet = EffectiveSignal {
            motion: M::Static,
            ..sustained
        };
        assert!(tick_deadline(&quiet, true, ms(100)).is_none());
    }

    #[test]
    fn oscillator_is_exact_after_days_of_uptime() {
        let days = Duration::from_secs(9 * 86_400);
        for t in [ms(0), ms(37), ms(613), ms(1199)] {
            let a = breath(M::Pulse, t, 0.3);
            let b = breath(M::Pulse, days + t, 0.3);
            assert!((a - b).abs() < 1e-6, "{t:?}: {a} vs {b}");
            assert_eq!(
                next_boundary(M::Pulse, days + t).map(|n| n - days),
                next_boundary(M::Pulse, t)
            );
        }
    }

    #[test]
    fn fingerprint_is_default_at_rest_and_tracks_progress_and_selector() {
        let e = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        let f = solve(&e, ms(5000), 0.1, FrameInputs::quiet());
        assert_eq!(
            SignalFingerprint::quantize(&f),
            SignalFingerprint::default()
        );

        let mut e2 = e.clone();
        e2.impulses.push(EffectiveImpulse {
            selector: 3,
            accent: None,
            at: ms(0),
        });
        // Two moments on the flat tail where the envelope bucket is the same but progress moved.
        let a = SignalFingerprint::quantize(&solve(&e2, ms(1400), 0.1, FrameInputs::quiet()));
        let b = SignalFingerprint::quantize(&solve(&e2, ms(1420), 0.1, FrameInputs::quiet()));
        assert_ne!(a, b);

        let mut e3 = e2.clone();
        e3.impulses[0].selector = 2;
        let c = SignalFingerprint::quantize(&solve(&e3, ms(1400), 0.1, FrameInputs::quiet()));
        assert_ne!(a, c);
    }

    #[test]
    fn effective_helpers_report_sustained_and_live_state() {
        let mut e = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![EffectiveImpulse {
                selector: 1,
                accent: None,
                at: ms(10),
            }],
        };
        assert!(!e.is_sustained());
        assert!(e.has_live_impulses(ms(1509)));
        assert!(!e.has_live_impulses(ms(1510)));
        e.motion = M::Breathe;
        assert!(e.is_sustained());
    }

    #[test]
    fn level_and_color_conversions_match_shader_values() {
        assert_eq!(level_value(L::Quiet), 0.);
        assert_eq!(level_value(L::Active), 1. / 3.);
        assert_eq!(level_value(L::Notice), 2. / 3.);
        assert_eq!(level_value(L::Demand), 1.);
        let [r, g, b] = color_linear(niri_config::Color::new_unpremul(0.04045, 0.5, 1., 1.));
        assert!((r - 0.003130805).abs() < 1e-7);
        assert!((g - 0.21404114).abs() < 1e-7);
        assert_eq!(b, 1.);
    }

    #[test]
    fn solve_limits_output_to_four_live_impulses() {
        let e = EffectiveSignal {
            accent: None,
            level: L::Demand,
            motion: M::Static,
            impulses: (1..=5)
                .map(|selector| EffectiveImpulse {
                    selector,
                    accent: None,
                    at: ms(0),
                })
                .collect(),
        };
        let f = solve(
            &e,
            ms(80),
            0.,
            FrameInputs {
                level: 0.75,
                accent: Some([0.1, 0.2, 0.3]),
                presence: 1.,
                ..FrameInputs::quiet()
            },
        );
        assert_eq!(f.level, 0.75);
        assert_eq!(f.accent, Some([0.1, 0.2, 0.3]));
        assert_eq!(f.impulses.map(|i| i.selector), [1, 2, 3, 4]);
        assert_eq!(f.impulses[0].envelope, 1.);
    }

    #[test]
    fn fingerprint_uses_the_documented_quantization_steps() {
        let frame = SignalFrame {
            accent: Some([1. / 512., 1. / 256., 0.]),
            level: 1. / 512.,
            breath: 1. / 64.,
            impulses: [
                ImpulseFrame {
                    selector: 3,
                    envelope: 1. / 256.,
                    progress: 3. / 256.,
                    accent: Some([1. / 512., 0., 1.]),
                },
                ImpulseFrame::default(),
                ImpulseFrame::default(),
                ImpulseFrame::default(),
            ],
            presence: 1. / 512.,
            focus: 1. / 256.,
            beam: BeamFrame {
                head: 0.25,
                env: 1. / 256.,
                decay: 1. / 256.,
            },
        };
        assert_eq!(
            SignalFingerprint::quantize(&frame),
            SignalFingerprint {
                level_q: 1,
                accent_q: [1, 1, 0],
                breath_q: 1,
                impulses_q: [
                    (3, 1, 2, [1, 0, 256]),
                    (0, 0, 0, [-1; 3]),
                    (0, 0, 0, [-1; 3]),
                    (0, 0, 0, [-1; 3]),
                ],
                presence_q: 1,
                focus_q: 1,
                head_q: 1,
                env_q: 1,
                decay_q: 1,
            }
        );
    }

    #[test]
    fn drift_rate_follows_policy_and_animations() {
        assert_eq!(drift_rate(15., P::Full, false), 15.);
        assert_eq!(drift_rate(15., P::Reduced, false), 7.5);
        assert_eq!(drift_rate(15., P::Off, false), 0.);
        assert_eq!(drift_rate(15., P::Full, true), 0.);
        assert_eq!(drift_rate(0., P::Full, false), 0.);
    }

    #[test]
    fn a_ten_minute_period_buckets_at_the_rate_per_second() {
        let period = Duration::from_secs(600);
        // 4 Hz over 600 s is 2400 buckets of 250 ms, anchored to the clock.
        assert_eq!(next_boundary_on(period, 4., ms(0)), Some(ms(250)));
        assert_eq!(next_boundary_on(period, 4., ms(250)), Some(ms(500)));
        assert_eq!(next_boundary_on(period, 4., ms(251)), Some(ms(500)));
        assert_eq!(next_boundary_on(period, 4., ms(599_990)), Some(ms(600_000)));
        assert_eq!(next_boundary_on(period, 0., ms(100)), None);
        // The phase advances one 2400th of a turn per bucket and is
        // constant inside a bucket.
        assert_eq!(phase_on(period, 4., ms(0), 0.), 0.);
        assert_eq!(phase_on(period, 4., ms(249), 0.), 0.);
        let one_bucket = TAU / 2400.;
        assert!((phase_on(period, 4., ms(250), 0.) - one_bucket).abs() < 1e-6);
        assert_eq!(phase_on(period, 0., ms(250), 0.3), 0.);
        // Boundaries are strictly future and each one changes the phase.
        for hz in [1., 4., 30.] {
            let mut now = Duration::ZERO;
            for _ in 0..100 {
                let b = next_boundary_on(period, hz, now).unwrap();
                assert!(b > now, "{hz} Hz at {now:?}");
                assert_ne!(
                    phase_on(period, hz, b, 0.1),
                    phase_on(period, hz, now, 0.1),
                    "{hz} Hz at {now:?}"
                );
                now = b;
            }
        }
    }

    #[test]
    fn solve_carries_focus_presence_and_the_beam() {
        let e = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        let quiet = solve(&e, ms(5000), 0.1, FrameInputs::quiet());
        assert_eq!((quiet.presence, quiet.focus), (0., 0.));
        assert_eq!(quiet.beam, BeamFrame::REST);
        assert_eq!(
            SignalFingerprint::quantize(&quiet),
            SignalFingerprint::default()
        );

        let focused = solve(
            &e,
            ms(5000),
            0.1,
            FrameInputs {
                focus: 1.,
                beam: BeamFrame {
                    head: 1234.5,
                    env: 0.5,
                    decay: 0.75,
                },
                ..FrameInputs::quiet()
            },
        );
        assert_eq!(focused.focus, 1.);
        assert_eq!(
            focused.beam,
            BeamFrame {
                head: 1234.5,
                env: 0.5,
                decay: 0.75,
            },
            "the beam frame is carried straight"
        );

        let half = solve(
            &e,
            ms(5000),
            0.1,
            FrameInputs {
                accent: Some([1., 0.5, 0.]),
                presence: 0.5,
                ..FrameInputs::quiet()
            },
        );
        assert_eq!(
            half.accent,
            Some([1., 0.5, 0.]),
            "accent is carried straight"
        );
        assert_eq!(half.presence, 0.5);
    }

    #[test]
    fn tick_deadline_has_no_focus_arm() {
        let quiet = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        assert_eq!(tick_deadline(&quiet, true, ms(123)), None);
        let breathing = EffectiveSignal {
            motion: M::Breathe,
            ..quiet
        };
        assert!(tick_deadline(&breathing, true, ms(123)).is_some());
        assert_eq!(tick_deadline(&breathing, false, ms(123)), None);
    }

    #[test]
    fn settled_focus_fingerprints_to_a_constant() {
        let quiet = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        let inputs = FrameInputs {
            focus: 1.,
            beam: BeamFrame::REST,
            ..FrameInputs::quiet()
        };
        let a = SignalFingerprint::quantize(&solve(&quiet, ms(1000), 0.3, inputs));
        let b = SignalFingerprint::quantize(&solve(&quiet, ms(2000), 0.3, inputs));
        assert_eq!(a, b);
        let mid = FrameInputs {
            beam: BeamFrame {
                head: 100.,
                env: 1.,
                decay: 1.,
            },
            ..inputs
        };
        let c = SignalFingerprint::quantize(&solve(&quiet, ms(1000), 0.3, mid));
        assert_ne!(a, c, "a running beam changes the fingerprint");
    }

    #[test]
    fn the_fingerprint_tracks_the_head_by_half_pixels_through_the_drain() {
        let quiet = EffectiveSignal {
            accent: None,
            level: L::Quiet,
            motion: M::Static,
            impulses: vec![],
        };
        let mut f = solve(&quiet, ms(1000), 0.3, FrameInputs::quiet());
        f.focus = 1.;
        f.beam = BeamFrame {
            head: 3000.,
            env: 0.,
            decay: 1.,
        };
        let a = SignalFingerprint::quantize(&f);
        f.beam.head = 3000.2;
        assert_eq!(
            SignalFingerprint::quantize(&f),
            a,
            "under half a pixel is the same frame"
        );
        f.beam.head = 3000.6;
        assert_ne!(
            SignalFingerprint::quantize(&f),
            a,
            "the drain keeps redrawing"
        );
        f.beam = BeamFrame::REST;
        let rest = SignalFingerprint::quantize(&f);
        assert_eq!(SignalFingerprint::quantize(&f), rest, "rest is constant");
    }

    #[test]
    fn idle_input_makes_sustained_motion_static_and_nothing_else() {
        let folded = Folded {
            level: L::Demand,
            motion: M::Pulse,
            accent: Some(Color::from_rgba8_unpremul(0xe5, 0xa3, 0x3c, 0xff)),
            tag: None,
            sources: vec![String::from("demo")],
            impulses: vec![impulse(ImpulseKind::Ping, ms(100))],
        };
        let response = ResolvedResponse::default();
        let active = effective(&folded, P::Full, &response, true);
        assert_eq!(active.motion, M::Pulse);
        let idle = effective(&folded, P::Full, &response, false);
        assert_eq!(idle.motion, M::Static);
        assert_eq!(idle.level, active.level);
        assert_eq!(idle.accent, active.accent);
        assert_eq!(idle.impulses, active.impulses);
    }

    #[test]
    fn finite_impulse_finishes_while_optic_time_is_held() {
        let clock = crate::animation::Clock::with_time(Duration::ZERO);
        let folded = folded(M::Static, vec![impulse(ImpulseKind::Ping, ms(100))]);
        let effective = effective(&folded, P::Full, &ResolvedResponse::default(), false);
        clock.set_optic_active(false, ms(100));
        let live = solve(&effective, ms(200), 0., FrameInputs::quiet());
        assert!(live.impulses[0].envelope > 0.);
        let rest = solve(&effective, ms(1600), 0., FrameInputs::quiet());
        assert_eq!(rest.impulses[0].envelope, 0.);
        assert_eq!(clock.optic_time(ms(1600)).logical_now, ms(100));
    }
}
