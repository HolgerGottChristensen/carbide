use crate::animation::keyframes::timeline::{ResolveMode, Timeline, TimelineStatus};
use crate::time::Duration;

/// Parallel composition: every child is fed the same local time each tick.
/// A shorter child holds its final value once finished while longer
/// children keep animating — matches SwiftUI's independent-parallel-track
/// behavior and Qt's `QParallelAnimationGroup`.
///
/// Exposes both the fluent `.and()` builder and [`Tracks::from_vec`] as
/// first-class construction paths, mirroring [`Chain`](crate::animation::keyframes::Chain).
#[derive(Debug, Default)]
pub struct Tracks {
    children: Vec<Box<dyn Timeline>>,
    /// Cached max of children's `duration()`; `None` if any child is `None`.
    duration: Option<Duration>,
    /// Cached max of children's `seek_duration()` — always finite.
    seek_duration: Duration,
}

impl Tracks {
    pub fn new() -> Self {
        Tracks { children: Vec::new(), duration: Some(Duration::ZERO), seek_duration: Duration::ZERO }
    }

    pub fn and(self, child: impl Timeline + 'static) -> Self {
        self.and_boxed(Box::new(child))
    }

    pub fn and_boxed(mut self, child: Box<dyn Timeline>) -> Self {
        self.duration = match (self.duration, child.duration()) {
            (Some(a), Some(b)) => Some(a.max(b)),
            _ => None,
        };
        self.seek_duration = self.seek_duration.max(child.seek_duration());
        self.children.push(child);
        self
    }

    /// Construct directly from a `Vec` of already-boxed children — the
    /// natural fit for programmatically-built groups (e.g. staggered
    /// per-item entrance animations for a runtime-sized list).
    pub fn from_vec(children: Vec<Box<dyn Timeline>>) -> Self {
        let mut tracks = Tracks::new();
        for child in children {
            tracks = tracks.and_boxed(child);
        }
        tracks
    }
}

impl Timeline for Tracks {
    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn seek_duration(&self) -> Duration {
        self.seek_duration
    }

    fn seek(&mut self, local_time: Duration, mode: ResolveMode) -> TimelineStatus {
        let total = match mode {
            ResolveMode::Playback => self.duration,
            ResolveMode::Seek => Some(self.seek_duration),
        };

        let clamped = match total {
            Some(t) => local_time.min(t),
            None => local_time,
        };

        for child in self.children.iter_mut() {
            let child_width = match mode {
                ResolveMode::Playback => child.duration(),
                ResolveMode::Seek => Some(child.seek_duration()),
            };
            let child_local = match child_width {
                Some(w) => clamped.min(w),
                None => clamped,
            };
            child.seek(child_local, mode);
        }

        TimelineStatus::new(
            match total {
                Some(t) if !t.is_zero() => clamped.as_secs_f64() / t.as_secs_f64(),
                Some(_) => 1.0,
                None => 0.0,
            },
            match total {
                Some(t) => local_time >= t,
                None => false,
            },
            match total {
                Some(t) => local_time.saturating_sub(t),
                None => Duration::ZERO,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::keyframes::{Keyframes, Repeat};
    use crate::state::{LocalState, ReadState, RepeatMode};

    use super::*;

    #[test]
    fn shorter_child_holds_final_value_while_longer_keeps_animating() {
        let short_state = LocalState::new(0.0f64);
        let long_state = LocalState::new(0.0f64);
        let short = Keyframes::new(short_state.clone(), 0.0).to(10.0, Duration::from_secs(1));
        let long = Keyframes::new(long_state.clone(), 0.0).to(10.0, Duration::from_secs(2));

        let mut tracks = Tracks::new().and(short).and(long);
        assert_eq!(tracks.duration(), Some(Duration::from_secs(2)));

        tracks.seek(Duration::from_millis(1500), ResolveMode::Playback);
        assert_eq!(*short_state.value(), 10.0); // finished, holding
        assert_eq!(*long_state.value(), 7.5);   // still animating
    }

    #[test]
    fn both_children_receive_the_same_local_time_scaled_to_their_own_duration() {
        let a_state = LocalState::new(0.0f64);
        let b_state = LocalState::new(0.0f64);
        let a = Keyframes::new(a_state.clone(), 0.0).to(100.0, Duration::from_secs(1));
        let b = Keyframes::new(b_state.clone(), 0.0).to(100.0, Duration::from_secs(4));

        let mut tracks = Tracks::new().and(a).and(b);
        tracks.seek(Duration::from_secs(1), ResolveMode::Playback);

        assert_eq!(*a_state.value(), 100.0); // 1s / 1s = fully done
        assert_eq!(*b_state.value(), 25.0);  // 1s / 4s = 25%
    }

    #[test]
    fn none_propagation_with_max_semantics() {
        let bounded_state = LocalState::new(0.0f64);
        let unbounded_state = LocalState::new(0.0f64);
        let bounded = Keyframes::new(bounded_state.clone(), 0.0).to(10.0, Duration::from_secs(1));
        let unbounded = Repeat::forever(
            Keyframes::new(unbounded_state.clone(), 0.0).to(360.0, Duration::from_secs(3)),
            RepeatMode::FromBeginning,
        );

        let tracks = Tracks::new().and(bounded).and(unbounded);
        assert_eq!(tracks.duration(), None);
        assert_eq!(tracks.seek_duration(), Duration::from_secs(3)); // max(1s, 3s one-loop)
    }

    #[test]
    fn from_vec_and_builder_agree() {
        let a1 = LocalState::new(0.0f64);
        let b1 = LocalState::new(0.0f64);
        let mut builder_tracks = Tracks::new()
            .and(Keyframes::new(a1.clone(), 0.0).to(10.0, Duration::from_secs(1)))
            .and(Keyframes::new(b1.clone(), 0.0).to(20.0, Duration::from_secs(2)));

        let a2 = LocalState::new(0.0f64);
        let b2 = LocalState::new(0.0f64);
        let mut vec_tracks = Tracks::from_vec(vec![
            Box::new(Keyframes::new(a2.clone(), 0.0).to(10.0, Duration::from_secs(1))),
            Box::new(Keyframes::new(b2.clone(), 0.0).to(20.0, Duration::from_secs(2))),
        ]);

        assert_eq!(builder_tracks.duration(), vec_tracks.duration());
        assert_eq!(builder_tracks.seek_duration(), vec_tracks.seek_duration());

        for t in [0, 500, 1000, 1500, 2000] {
            builder_tracks.seek(Duration::from_millis(t), ResolveMode::Playback);
            vec_tracks.seek(Duration::from_millis(t), ResolveMode::Playback);
            assert_eq!(*a1.value(), *a2.value());
            assert_eq!(*b1.value(), *b2.value());
        }
    }
}
