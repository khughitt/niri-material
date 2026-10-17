//! Which project hue each workspace holds for the shell
//! (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md §4).
//!
//! Pure: the caller passes what each workspace shows at `now`. A window
//! counts once it has stayed its workspace's active window for
//! `HOLD_DWELL`; the workspace then holds that window's accent while
//! windows without one are active, and goes neutral when the held window
//! closes, leaves, or loses its accent. Focus timestamps play no part.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use niri_config::Color;

use crate::animation::{Animation, Clock};
use crate::render_helpers::surface_remap::{HueBlend, OutputHue};

/// How long a window must stay active before it counts (spec §8).
pub const HOLD_DWELL: Duration = Duration::from_millis(1000);

/// One workspace as the resolver sees it this frame.
#[derive(Debug, Clone)]
pub struct WorkspaceObs<Ws, K> {
    pub id: Ws,
    pub active: Option<K>,
    /// Every window on the workspace with its folded accent.
    pub windows: Vec<(K, Option<Color>)>,
}

#[derive(Debug)]
struct Fade {
    from: HueBlend,
    to: HueBlend,
    anim: Animation,
}

#[derive(Debug)]
struct WorkspaceHue<K> {
    /// The active window and when it became active.
    candidate: Option<(K, Duration)>,
    /// The settled window whose accent the workspace holds.
    record: Option<K>,
    target: HueBlend,
    fade: Option<Fade>,
}

impl<K> WorkspaceHue<K> {
    fn current(&self) -> HueBlend {
        match &self.fade {
            Some(fade) => HueBlend::lerp(fade.from, fade.to, fade.anim.clamped_value() as f32),
            None => self.target,
        }
    }
}

#[derive(Debug)]
pub struct ShellHue<Ws, K> {
    clock: Clock,
    workspaces: HashMap<Ws, WorkspaceHue<K>>,
    now: Duration,
}

impl<Ws: Copy + Eq + Hash, K: Copy + Eq> ShellHue<Ws, K> {
    pub fn new(clock: Clock) -> Self {
        Self {
            clock,
            workspaces: HashMap::new(),
            now: Duration::ZERO,
        }
    }

    pub fn update(
        &mut self,
        now: Duration,
        anim: niri_config::Animation,
        obs: &[WorkspaceObs<Ws, K>],
    ) {
        self.now = now;
        self.workspaces
            .retain(|id, _| obs.iter().any(|o| o.id == *id));

        for o in obs {
            let ws = self.workspaces.entry(o.id).or_insert_with(|| WorkspaceHue {
                candidate: None,
                record: None,
                target: HueBlend::NEUTRAL,
                fade: None,
            });
            // `None`: not on this workspace; `Some(None)`: here, no accent.
            let accent_of = |k: K| o.windows.iter().find(|(w, _)| *w == k).map(|(_, a)| *a);

            if ws.candidate.map(|(k, _)| k) != o.active {
                ws.candidate = o.active.map(|k| (k, now));
            }
            if let Some((k, since)) = ws.candidate {
                if now.saturating_sub(since) >= HOLD_DWELL && matches!(accent_of(k), Some(Some(_)))
                {
                    ws.record = Some(k);
                }
            }

            let accent = ws.record.and_then(|k| accent_of(k).flatten());
            if accent.is_none() {
                ws.record = None;
            }
            let target = accent.map_or(HueBlend::NEUTRAL, HueBlend::from_accent);

            if target != ws.target {
                let from = ws.current();
                ws.fade = Some(Fade {
                    from,
                    to: target,
                    anim: Animation::new(self.clock.clone(), 0., 1., 0., anim),
                });
                ws.target = target;
            }
            if ws.fade.as_ref().is_some_and(|fade| fade.anim.is_done()) {
                ws.fade = None;
            }
        }
    }

    pub fn current(&self, ws: Ws) -> HueBlend {
        self.workspaces
            .get(&ws)
            .map_or(HueBlend::NEUTRAL, WorkspaceHue::current)
    }

    pub fn is_fading(&self, ws: Ws) -> bool {
        self.workspaces.get(&ws).is_some_and(|w| w.fade.is_some())
    }

    /// The output's point: the two workspaces it shows, blended by the
    /// fractional render index (`Monitor::workspace_blend`).
    pub fn output_hue(&self, (a, b, frac): (Ws, Ws, f64)) -> OutputHue {
        OutputHue {
            blend: HueBlend::lerp(self.current(a), self.current(b), frac as f32),
            animating: self.is_fading(a) || self.is_fading(b),
        }
    }

    /// The earliest moment a pending window settles, so a redraw can be
    /// scheduled for it.
    pub fn next_deadline(&self) -> Option<Duration> {
        self.workspaces
            .values()
            .filter_map(|w| {
                let (k, since) = w.candidate?;
                (w.record != Some(k)).then_some(since + HOLD_DWELL)
            })
            .filter(|deadline| *deadline > self.now)
            .min()
    }
}

#[cfg(test)]
mod tests {
    use niri_config::animations::MaterialSignalAnim;

    use super::*;

    const RED: &str = "#e5533c";
    const CYAN: &str = "#3cc6e5";

    fn color(s: &str) -> Color {
        s.parse().unwrap()
    }

    fn hued(s: &str) -> HueBlend {
        HueBlend::from_accent(color(s))
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn off() -> niri_config::Animation {
        niri_config::Animation {
            off: true,
            ..MaterialSignalAnim::default().0
        }
    }

    fn on() -> niri_config::Animation {
        MaterialSignalAnim::default().0
    }

    fn ws(id: u64, active: Option<u64>, windows: &[(u64, Option<&str>)]) -> WorkspaceObs<u64, u64> {
        WorkspaceObs {
            id,
            active,
            windows: windows.iter().map(|(k, a)| (*k, a.map(color))).collect(),
        }
    }

    fn resolver() -> ShellHue<u64, u64> {
        ShellHue::new(Clock::with_time(Duration::ZERO))
    }

    /// Agent 1 (red) settled on workspace 1 at 1000 ms.
    fn settled_on_red() -> ShellHue<u64, u64> {
        let mut r = resolver();
        let w = [ws(1, Some(1), &[(1, Some(RED)), (2, None)])];
        r.update(ms(0), off(), &w);
        r.update(ms(1000), off(), &w);
        assert_eq!(r.current(1), hued(RED));
        r
    }

    #[test]
    fn settles_after_the_dwell() {
        let mut r = resolver();
        let w = [ws(1, Some(1), &[(1, Some(RED))])];
        r.update(ms(0), off(), &w);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        r.update(ms(999), off(), &w);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        r.update(ms(1000), off(), &w);
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn a_brief_visit_does_not_count() {
        let mut r = settled_on_red();
        let windows = [(1, Some(RED)), (2, None), (3, Some(CYAN))];
        r.update(ms(2000), off(), &[ws(1, Some(3), &windows)]);
        r.update(ms(2300), off(), &[ws(1, Some(2), &windows)]);
        r.update(ms(8000), off(), &[ws(1, Some(2), &windows)]);
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn hold_across_a_window_without_accent() {
        let mut r = settled_on_red();
        let windows = [(1, Some(RED)), (2, None)];
        r.update(ms(2000), off(), &[ws(1, Some(2), &windows)]);
        r.update(ms(60_000), off(), &[ws(1, Some(2), &windows)]);
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn a_settled_second_agent_takes_over() {
        let mut r = settled_on_red();
        let windows = [(1, Some(RED)), (3, Some(CYAN))];
        r.update(ms(2000), off(), &[ws(1, Some(3), &windows)]);
        r.update(ms(3000), off(), &[ws(1, Some(3), &windows)]);
        assert_eq!(r.current(1), hued(CYAN));
    }

    #[test]
    fn closing_the_recorded_window_goes_neutral() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(1, Some(2), &[(2, None)])]);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
    }

    #[test]
    fn lost_accent_goes_neutral() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(1, Some(1), &[(1, None)])]);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
    }

    #[test]
    fn recorded_accent_change_is_followed() {
        let mut r = settled_on_red();
        r.update(
            ms(2000),
            off(),
            &[ws(1, Some(2), &[(1, Some(CYAN)), (2, None)])],
        );
        assert_eq!(r.current(1), hued(CYAN));
    }

    #[test]
    fn moved_window_clears_its_origin_record() {
        let mut r = settled_on_red();
        let both = |t: u64, r: &mut ShellHue<u64, u64>| {
            r.update(
                ms(t),
                off(),
                &[
                    ws(1, Some(2), &[(2, None)]),
                    ws(2, Some(1), &[(1, Some(RED))]),
                ],
            );
        };
        both(2000, &mut r);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        assert_eq!(r.current(2), HueBlend::NEUTRAL);
        both(2999, &mut r);
        assert_eq!(r.current(2), HueBlend::NEUTRAL);
        both(3000, &mut r);
        assert_eq!(r.current(2), hued(RED));
    }

    #[test]
    fn switch_blend_mixes_two_workspaces() {
        let mut r = resolver();
        let w = [
            ws(1, Some(1), &[(1, Some(RED))]),
            ws(2, Some(2), &[(2, Some(CYAN))]),
        ];
        r.update(ms(0), off(), &w);
        r.update(ms(1000), off(), &w);
        let point = r.output_hue((1, 2, 0.25));
        assert_eq!(point.blend, HueBlend::lerp(hued(RED), hued(CYAN), 0.25));
        assert!(!point.animating);
    }

    #[test]
    fn two_outputs_hold_their_own_hues() {
        let mut r = resolver();
        let w = [
            ws(1, Some(1), &[(1, Some(RED))]),
            ws(2, Some(2), &[(2, Some(CYAN))]),
        ];
        r.update(ms(0), off(), &w);
        r.update(ms(1000), off(), &w);
        assert_eq!(r.output_hue((1, 1, 0.)).blend, hued(RED));
        assert_eq!(r.output_hue((2, 2, 0.)).blend, hued(CYAN));
    }

    #[test]
    fn fade_reports_animating_until_it_ends() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut r: ShellHue<u64, u64> = ShellHue::new(clock.clone());
        let w = [ws(1, Some(1), &[(1, Some(RED))])];
        r.update(ms(0), on(), &w);
        clock.set_unadjusted(ms(1000));
        r.update(ms(1000), on(), &w);
        assert!(r.is_fading(1));
        assert!(r.output_hue((1, 1, 0.)).animating);
        clock.set_unadjusted(ms(1500));
        r.update(ms(1500), on(), &w);
        assert!(!r.is_fading(1));
        assert_eq!(r.current(1), hued(RED));
    }

    #[test]
    fn interrupted_fade_continues_from_its_point() {
        let mut clock = Clock::with_time(Duration::ZERO);
        let mut r: ShellHue<u64, u64> = ShellHue::new(clock.clone());
        let red = [ws(1, Some(1), &[(1, Some(RED)), (2, Some(CYAN))])];
        let cyan = [ws(1, Some(2), &[(1, Some(RED)), (2, Some(CYAN))])];
        r.update(ms(0), on(), &red);
        clock.set_unadjusted(ms(1000));
        r.update(ms(1000), on(), &red); // fade neutral -> red starts
        clock.set_unadjusted(ms(1500));
        r.update(ms(1500), on(), &red); // red settled
        r.update(ms(1500), on(), &cyan);
        clock.set_unadjusted(ms(2500));
        r.update(ms(2500), on(), &cyan); // fade red -> cyan starts
        clock.set_unadjusted(ms(2600));
        r.update(ms(2600), on(), &cyan);
        let before = r.current(1);
        // Agent 2, the recorded window, loses its accent mid-fade, so the
        // target flips to neutral; the new fade must start from the point
        // the red -> cyan fade had reached.
        let lost = [ws(1, Some(2), &[(1, Some(RED)), (2, None)])];
        r.update(ms(2600), on(), &lost);
        let after = r.current(1);
        assert_eq!(before, after, "a new fade starts where the old one was");
        assert!(r.is_fading(1));
        clock.set_unadjusted(ms(3200));
        r.update(ms(3200), on(), &lost);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
    }

    #[test]
    fn removed_workspaces_are_forgotten() {
        let mut r = settled_on_red();
        r.update(ms(2000), off(), &[ws(2, None, &[])]);
        assert_eq!(r.current(1), HueBlend::NEUTRAL);
        assert!(!r.workspaces.contains_key(&1));
    }

    #[test]
    fn next_deadline_is_the_pending_settle() {
        let mut r = resolver();
        let w = [ws(1, Some(1), &[(1, Some(RED))])];
        r.update(ms(400), off(), &w);
        assert_eq!(r.next_deadline(), Some(ms(1400)));
        r.update(ms(1400), off(), &w);
        assert_eq!(r.next_deadline(), None);
    }
}
