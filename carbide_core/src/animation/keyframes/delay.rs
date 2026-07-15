use crate::animation::keyframes::timeline::{ResolveMode, Timeline, TimelineStatus};
use crate::time::Duration;

/// A leaf [`Timeline`] that occupies a fixed time slot but touches no state.
/// Useful as a middle step of a [`Chain`](crate::animation::keyframes::Chain)
/// to express "wait, doing nothing" (e.g. a toast that slides in, holds,
/// then slides out).
#[derive(Debug, Clone, Copy)]
pub struct Delay(Duration);

impl Delay {
    pub fn new(duration: Duration) -> Self {
        Delay(duration)
    }
}

impl Timeline for Delay {
    fn duration(&self) -> Option<Duration> {
        Some(self.0)
    }

    fn seek_duration(&self) -> Duration {
        self.0
    }

    // `mode` is unused by design, not oversight: like every leaf, `Delay` has
    // no children and can never be unbounded, so `duration()` and
    // `seek_duration()` always agree — Playback vs Seek has nothing to
    // disambiguate here (see `Keyframes::seek` for the fuller explanation).
    fn seek(&mut self, local_time: Duration, _mode: ResolveMode) -> TimelineStatus {
        let progress = if self.0.is_zero() {
            1.0
        } else {
            (local_time.as_secs_f64() / self.0.as_secs_f64()).min(1.0)
        };

        TimelineStatus::new(progress, local_time >= self.0, local_time.saturating_sub(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_and_seek_duration_equal_the_constructed_duration() {
        let delay = Delay::new(Duration::from_secs(2));
        assert_eq!(delay.duration(), Some(Duration::from_secs(2)));
        assert_eq!(delay.seek_duration(), Duration::from_secs(2));
    }

    #[test]
    fn seek_at_start_mid_and_end() {
        let mut delay = Delay::new(Duration::from_secs(2));

        let start = delay.seek(Duration::ZERO, ResolveMode::Playback);
        assert_eq!(start.progress, 0.0);
        assert!(!start.finished);

        let mid = delay.seek(Duration::from_secs(1), ResolveMode::Playback);
        assert_eq!(mid.progress, 0.5);
        assert!(!mid.finished);

        let end = delay.seek(Duration::from_secs(2), ResolveMode::Playback);
        assert_eq!(end.progress, 1.0);
        assert!(end.finished);
        assert_eq!(end.overshoot, Duration::ZERO);
    }

    #[test]
    fn beyond_the_end_reports_overshoot() {
        let mut delay = Delay::new(Duration::from_secs(2));
        let status = delay.seek(Duration::from_secs(3), ResolveMode::Playback);
        assert!(status.finished);
        assert_eq!(status.overshoot, Duration::from_secs(1));
    }

    #[test]
    fn zero_duration_delay_is_immediately_finished_without_dividing_by_zero() {
        let mut delay = Delay::new(Duration::ZERO);
        let status = delay.seek(Duration::ZERO, ResolveMode::Playback);
        assert_eq!(status.progress, 1.0);
        assert!(status.finished);
    }
}
