use std::fmt::Debug;
use crate::time::Duration;

/// Distinguishes a real wall-clock-driven playback step from an explicit
/// seek. A [`Chain`](crate::animation::keyframes::Chain) needs this to decide
/// how wide an unbounded (infinitely repeating) child's slot is: during
/// [`Playback`](ResolveMode::Playback) it blocks forever (its `duration()` is
/// `None`), but during [`Seek`](ResolveMode::Seek) it is treated as exactly
/// one representative loop (`seek_duration()`) so later siblings stay
/// reachable by scrubbing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveMode {
    Playback,
    Seek,
}

/// A node in a composable keyframe/timeline tree. Implemented by the leaf
/// [`Keyframes<T>`](crate::animation::keyframes::Keyframes) and
/// [`Delay`](crate::animation::keyframes::Delay), and by the composites
/// [`Chain`](crate::animation::keyframes::Chain),
/// [`Tracks`](crate::animation::keyframes::Tracks) and
/// [`Repeat`](crate::animation::keyframes::Repeat), all stored type-erased as
/// `Box<dyn Timeline>` so a composite can hold heterogeneous children (e.g. a
/// `Keyframes<f64>` next to a `Keyframes<Color>`).
///
/// A `Timeline` is a pure function of local time: it holds no wall-clock
/// state of its own (no `Instant`, no "am I playing" flag). All wall-clock
/// bookkeeping (play/pause/speed/looping) lives in
/// [`TimelineController`](crate::animation::keyframes::TimelineController),
/// which repeatedly calls [`seek`](Timeline::seek) with an absolute
/// `Duration` measured from this node's own start. This is what makes
/// `seek(progress)` exact and idempotent at every nesting level: seeking is
/// not a diff from last time, it's a direct evaluation.
pub trait Timeline: Debug {
    /// Total duration of this node at 1x speed, with no outer repeat applied.
    /// `None` means this node never settles on its own (an unbounded
    /// [`Repeat`](crate::animation::keyframes::Repeat) somewhere in this
    /// subtree). Composites cache/fold this from their children — never
    /// recompute it inside `seek`.
    fn duration(&self) -> Option<Duration>;

    /// Always-finite duration used only to normalize `seek(progress: 0..1)`.
    /// For any unbounded subtree, substitutes "one representative loop"
    /// (as if `count` were 1) at every level. This is a distinct query from
    /// [`duration`](Timeline::duration) specifically so that "does this ever
    /// finish" and "what does progress = 1.0 mean for scrubbing" can
    /// disagree when unbounded repeats are involved.
    fn seek_duration(&self) -> Duration;

    /// Evaluate this node at `local_time` (0 at this node's own start),
    /// writing into any bound state as a side effect. `mode` distinguishes a
    /// real wall-clock-driven playback step (where an unbounded child blocks
    /// forever) from an explicit seek (where an unbounded child is treated as
    /// one representative loop so later siblings stay reachable). Composites
    /// must consult `mode` when picking which of `duration()`/
    /// `seek_duration()` to use for child slot widths.
    fn seek(&mut self, local_time: Duration, mode: ResolveMode) -> TimelineStatus;
}

/// Result of evaluating a [`Timeline`] node at a given local time.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct TimelineStatus {
    /// This node's progress through its own span, clamped to `0.0..=1.0`.
    pub progress: f64,
    /// `true` once `local_time >= duration()` for this node (only meaningful
    /// for nodes with a finite `duration()`).
    pub finished: bool,
    /// How far past this node's own span the requested `local_time` was
    /// (`Duration::ZERO` if not finished).
    pub overshoot: Duration,
}

impl TimelineStatus {
    pub fn new(progress: f64, finished: bool, overshoot: Duration) -> Self {
        TimelineStatus {
            progress: progress.clamp(0.0, 1.0),
            finished,
            overshoot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_progress_into_0_1() {
        let below = TimelineStatus::new(-0.2, false, Duration::ZERO);
        assert_eq!(below.progress, 0.0);

        let above = TimelineStatus::new(1.3, true, Duration::ZERO);
        assert_eq!(above.progress, 1.0);

        let inside = TimelineStatus::new(0.5, false, Duration::ZERO);
        assert_eq!(inside.progress, 0.5);
    }
}
