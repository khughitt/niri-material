use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::activity::{OpticTime, OpticTimeline};
use crate::utils::get_monotonic_time;

/// Shareable lazy clock that can change rate.
///
/// The clock will fetch the time once and then retain it until explicitly cleared with
/// [`Clock::clear`].
#[derive(Debug, Default, Clone)]
pub struct Clock {
    inner: Rc<RefCell<AdjustableClock>>,
}

#[derive(Debug, Default)]
struct LazyClock {
    time: Option<Duration>,
}

/// Clock that can adjust its rate.
#[derive(Debug)]
struct AdjustableClock {
    inner: LazyClock,
    optic_timeline: OpticTimeline,
    current_time: Duration,
    last_seen_time: Duration,
    rate: f64,
    complete_instantly: bool,
}

impl Clock {
    /// Creates a new clock with the given time.
    pub fn with_time(time: Duration) -> Self {
        let clock = AdjustableClock::new(LazyClock::with_time(time));
        Self {
            inner: Rc::new(RefCell::new(clock)),
        }
    }

    /// Returns the current time.
    pub fn now(&self) -> Duration {
        self.inner.borrow_mut().now()
    }

    /// Returns the underlying time not adjusted for rate change.
    pub fn now_unadjusted(&self) -> Duration {
        self.inner.borrow_mut().inner.now()
    }

    pub(crate) fn optic_time(&self, now: Duration) -> OpticTime {
        self.inner.borrow().optic_timeline.sample(now)
    }

    pub(crate) fn record_optic_render(&self, logical_now: Duration) {
        self.inner
            .borrow_mut()
            .optic_timeline
            .record_render(logical_now);
    }

    pub(crate) fn set_optic_active(&self, active: bool, now: Duration) -> Option<OpticTime> {
        self.inner
            .borrow_mut()
            .optic_timeline
            .set_active(active, now)
    }

    /// Sets the unadjusted clock time.
    pub fn set_unadjusted(&mut self, time: Duration) {
        self.inner.borrow_mut().inner.set(time);
    }

    /// Clears the stored time so it's re-fetched again next.
    pub fn clear(&mut self) {
        self.inner.borrow_mut().inner.clear();
    }

    /// Gets the clock rate.
    pub fn rate(&self) -> f64 {
        self.inner.borrow().rate()
    }

    /// Sets the clock rate.
    pub fn set_rate(&mut self, rate: f64) {
        self.inner.borrow_mut().set_rate(rate);
    }

    /// Returns whether animations should complete instantly.
    pub fn should_complete_instantly(&self) -> bool {
        self.inner.borrow().should_complete_instantly()
    }

    /// Sets whether animations should complete instantly.
    pub fn set_complete_instantly(&mut self, value: bool) {
        self.inner.borrow_mut().set_complete_instantly(value);
    }
}

impl PartialEq for Clock {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for Clock {}

impl LazyClock {
    pub fn with_time(time: Duration) -> Self {
        Self { time: Some(time) }
    }

    pub fn clear(&mut self) {
        self.time = None;
    }

    pub fn set(&mut self, time: Duration) {
        self.time = Some(time);
    }

    pub fn now(&mut self) -> Duration {
        *self.time.get_or_insert_with(get_monotonic_time)
    }
}

impl AdjustableClock {
    pub fn new(mut inner: LazyClock) -> Self {
        let time = inner.now();
        Self {
            inner,
            optic_timeline: OpticTimeline::new(time),
            current_time: time,
            last_seen_time: time,
            rate: 1.,
            complete_instantly: false,
        }
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate.clamp(0., 1000.);
    }

    pub fn should_complete_instantly(&self) -> bool {
        self.complete_instantly
    }

    pub fn set_complete_instantly(&mut self, value: bool) {
        self.complete_instantly = value;
    }

    pub fn now(&mut self) -> Duration {
        let time = self.inner.now();

        if self.last_seen_time == time {
            return self.current_time;
        }

        if self.last_seen_time < time {
            let delta = time - self.last_seen_time;
            let delta = delta.mul_f64(self.rate);
            self.current_time = self.current_time.saturating_add(delta);
        } else {
            let delta = self.last_seen_time - time;
            let delta = delta.mul_f64(self.rate);
            self.current_time = self.current_time.saturating_sub(delta);
        }

        self.last_seen_time = time;
        self.current_time
    }
}

impl Default for AdjustableClock {
    fn default() -> Self {
        Self::new(LazyClock::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optic_timeline_is_shared_by_clones_but_not_new_clocks() {
        let ms = Duration::from_millis;
        let clock = Clock::with_time(ms(100));
        let clone = clock.clone();
        assert_eq!(clone.optic_time(ms(120)).logical_now, ms(120));
        clock.record_optic_render(ms(140));
        assert_eq!(
            clone.set_optic_active(false, ms(130)).unwrap().logical_now,
            ms(140)
        );
        assert_eq!(clock.optic_time(ms(500)).logical_now, ms(140));
        assert_eq!(
            Clock::with_time(ms(100)).optic_time(ms(500)).logical_now,
            ms(500)
        );
        clone.record_optic_render(ms(900));
        assert_eq!(clock.optic_time(ms(500)).logical_now, ms(140));
        clock.set_optic_active(true, ms(500));
        assert_eq!(clone.optic_time(ms(510)).logical_now, ms(150));
    }

    #[test]
    fn optic_timeline_render_record_resets_between_active_intervals() {
        let ms = Duration::from_millis;
        let clock = Clock::with_time(Duration::ZERO);
        clock.optic_time(ms(900));
        assert_eq!(
            clock.set_optic_active(false, ms(100)).unwrap().logical_now,
            ms(100)
        );
        clock.set_optic_active(true, ms(200));
        clock.record_optic_render(ms(250));
        clock.record_optic_render(ms(220));
        assert_eq!(
            clock.set_optic_active(false, ms(210)).unwrap().logical_now,
            ms(250)
        );
        clock.set_optic_active(true, ms(300));
        assert_eq!(
            clock.set_optic_active(false, ms(310)).unwrap().logical_now,
            ms(260)
        );
    }

    #[test]
    fn optic_pause_does_not_change_animation_clock() {
        let ms = Duration::from_millis;
        let mut paused = Clock::with_time(Duration::ZERO);
        let mut reference = Clock::with_time(Duration::ZERO);
        paused.set_rate(0.5);
        reference.set_rate(0.5);
        paused.set_unadjusted(ms(100));
        reference.set_unadjusted(ms(100));
        assert_eq!(paused.now(), reference.now());
        paused.set_optic_active(false, ms(100));
        paused.set_complete_instantly(true);
        assert_eq!(paused.optic_time(ms(500)).logical_now, ms(100));
        paused.set_unadjusted(ms(500));
        reference.set_unadjusted(ms(500));
        assert_eq!(paused.now(), reference.now());
        paused.set_optic_active(true, ms(500));
        assert_eq!(paused.optic_time(ms(520)).logical_now, ms(120));
        paused.set_complete_instantly(false);
        paused.set_unadjusted(ms(520));
        reference.set_unadjusted(ms(520));
        assert_eq!(paused.now(), reference.now());
    }

    #[test]
    fn frozen_clock() {
        let mut clock = Clock::with_time(Duration::ZERO);
        assert_eq!(clock.now(), Duration::ZERO);

        clock.set_unadjusted(Duration::from_millis(100));
        assert_eq!(clock.now(), Duration::from_millis(100));

        clock.set_unadjusted(Duration::from_millis(200));
        assert_eq!(clock.now(), Duration::from_millis(200));
    }

    #[test]
    fn rate_change() {
        let mut clock = Clock::with_time(Duration::ZERO);
        clock.set_rate(0.5);

        clock.set_unadjusted(Duration::from_millis(100));
        assert_eq!(clock.now_unadjusted(), Duration::from_millis(100));
        assert_eq!(clock.now(), Duration::from_millis(50));

        clock.set_unadjusted(Duration::from_millis(200));
        assert_eq!(clock.now_unadjusted(), Duration::from_millis(200));
        assert_eq!(clock.now(), Duration::from_millis(100));

        clock.set_unadjusted(Duration::from_millis(150));
        assert_eq!(clock.now_unadjusted(), Duration::from_millis(150));
        assert_eq!(clock.now(), Duration::from_millis(75));

        clock.set_rate(2.0);

        clock.set_unadjusted(Duration::from_millis(250));
        assert_eq!(clock.now_unadjusted(), Duration::from_millis(250));
        assert_eq!(clock.now(), Duration::from_millis(275));
    }
}
