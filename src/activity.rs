//! Compositor-wide input activity for the attention gate
//! (docs/specs/2026-09-18-ring-focus-motion-design.md §3). Pure: the
//! event-loop timer in `Niri` asks this what to do and when.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpticTime {
    pub real_now: Duration,
    pub logical_now: Duration,
    pub running: bool,
    pub real_anchor: Duration,
    pub logical_anchor: Duration,
}

#[derive(Debug)]
pub(crate) struct OpticTimeline {
    real_anchor: Duration,
    logical_anchor: Duration,
    running: bool,
    rendered: Option<Duration>,
}

impl OpticTimeline {
    pub fn new(now: Duration) -> Self {
        Self {
            real_anchor: now,
            logical_anchor: now,
            running: true,
            rendered: None,
        }
    }

    pub fn sample(&self, now: Duration) -> OpticTime {
        let logical_now = if self.running {
            self.logical_anchor + now.saturating_sub(self.real_anchor)
        } else {
            self.logical_anchor
        };
        OpticTime {
            real_now: now,
            logical_now,
            running: self.running,
            real_anchor: self.real_anchor,
            logical_anchor: self.logical_anchor,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn record_render(&mut self, logical_now: Duration) {
        if self.running {
            self.rendered = Some(
                self.rendered
                    .map_or(logical_now, |prev| prev.max(logical_now)),
            );
        }
    }

    pub fn set_active(&mut self, active: bool, now: Duration) -> Option<OpticTime> {
        if self.running == active {
            return None;
        }
        if active {
            self.real_anchor = now;
        } else {
            self.logical_anchor = self
                .rendered
                .unwrap_or_default()
                .max(self.sample(now).logical_now);
        }
        self.running = active;
        self.rendered = None;
        Some(self.sample(now))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Poll {
    /// The threshold has elapsed; the caller drops its timer.
    Idle,
    /// Input arrived during the wait; re-arm for this absolute instant.
    Rearm(Duration),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputActivity {
    last: Duration,
    idle: bool,
    threshold: Duration,
}

impl InputActivity {
    pub fn new(now: Duration, threshold: Duration) -> Self {
        Self {
            last: now,
            idle: false,
            threshold,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.idle
    }

    /// Whether the gate is on at all.
    pub fn enabled(&self) -> bool {
        !self.threshold.is_zero()
    }

    /// The instant a timer should fire to check for idleness, or `None`
    /// when no timer is needed (gate off, or already idle).
    pub fn next_check(&self) -> Option<Duration> {
        (self.enabled() && !self.idle).then(|| self.last + self.threshold)
    }

    /// Input arrived. Returns true when this cleared idle.
    pub fn observe(&mut self, now: Duration) -> bool {
        self.last = now;
        std::mem::replace(&mut self.idle, false)
    }

    /// The timer fired.
    pub fn poll(&mut self, now: Duration) -> Poll {
        if !self.enabled() {
            return Poll::Rearm(Duration::MAX);
        }
        if now.saturating_sub(self.last) >= self.threshold {
            self.idle = true;
            Poll::Idle
        } else {
            Poll::Rearm(self.last + self.threshold)
        }
    }

    /// Config reload: recompute idle from the elapsed quiet time and the new
    /// threshold. Returns true when the idle state changed.
    pub fn set_threshold(&mut self, threshold: Duration, now: Duration) -> bool {
        self.threshold = threshold;
        let idle = self.enabled() && now.saturating_sub(self.last) >= threshold;
        std::mem::replace(&mut self.idle, idle) != idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optic_timeline_holds_and_resumes_without_catchup() {
        let ms = Duration::from_millis;
        let mut t = OpticTimeline::new(Duration::ZERO);
        assert_eq!(t.sample(ms(90)).logical_now, ms(90));
        assert_eq!(t.sample(ms(20)).logical_now, ms(20));
        t.record_render(ms(110));
        assert_eq!(t.set_active(false, ms(100)).unwrap().logical_now, ms(110));
        assert_eq!(t.sample(ms(1_300_000)).logical_now, ms(110));
        assert!(t.set_active(false, ms(1_300_000)).is_none());
        assert_eq!(
            t.set_active(true, ms(1_300_000)).unwrap().logical_now,
            ms(110)
        );
        assert_eq!(t.sample(ms(1_300_040)).logical_now, ms(150));
        assert!(t.set_active(true, ms(1_300_040)).is_none());
    }

    fn s(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    #[test]
    fn fresh_compositor_with_no_input_goes_idle_at_the_threshold() {
        let mut a = InputActivity::new(s(0), s(30));
        assert_eq!(a.next_check(), Some(s(30)));
        assert_eq!(a.poll(s(30)), Poll::Idle);
        assert!(a.is_idle());
        assert_eq!(a.next_check(), None);
    }

    #[test]
    fn input_inside_the_threshold_rearms_without_flipping() {
        let mut a = InputActivity::new(s(0), s(30));
        assert!(!a.observe(s(10)));
        assert_eq!(a.poll(s(30)), Poll::Rearm(s(40)));
        assert!(!a.is_idle());
        assert_eq!(a.poll(s(40)), Poll::Idle);
    }

    #[test]
    fn input_clears_idle_once() {
        let mut a = InputActivity::new(s(0), s(30));
        assert_eq!(a.poll(s(30)), Poll::Idle);
        assert!(a.observe(s(31)), "first input after idle reports the clear");
        assert!(!a.observe(s(32)), "further input is quiet");
        assert_eq!(a.next_check(), Some(s(62)));
    }

    #[test]
    fn zero_threshold_never_flips() {
        let mut a = InputActivity::new(s(0), s(0));
        assert_eq!(a.next_check(), None);
        assert_eq!(a.poll(s(1_000_000)), Poll::Rearm(Duration::MAX));
        assert!(!a.is_idle());
    }

    #[test]
    fn reload_recomputes_idle_in_both_directions() {
        let mut a = InputActivity::new(s(0), s(30));
        assert_eq!(a.poll(s(45)), Poll::Idle);
        // Raise past the elapsed 45 s: active again, timer due at 60.
        assert!(a.set_threshold(s(60), s(45)));
        assert!(!a.is_idle());
        assert_eq!(a.next_check(), Some(s(60)));
        // Lower under the elapsed time: idle at once.
        assert!(a.set_threshold(s(10), s(45)));
        assert!(a.is_idle());
        // Same side: no change reported.
        assert!(!a.set_threshold(s(20), s(45)));
        // Zero clears and disables.
        assert!(a.set_threshold(s(0), s(45)));
        assert!(!a.is_idle());
        assert_eq!(a.next_check(), None);
    }
}
