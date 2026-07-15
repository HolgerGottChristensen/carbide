use std::cell::RefCell;
use std::rc::Rc;

use crate::animation::keyframes::timeline::{ResolveMode, Timeline};
use crate::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Playing,
    Paused,
    Stopped,
}

/// Owns a root [`Timeline`] plus a wall-clock playhead, exposing
/// `play`/`pause`/`stop`/`seek`/`set_speed`. Every controller call re-pins
/// `(accumulated, resumed_at)` atomically *before* changing the going-forward
/// rule (speed, root, etc.), so `now == resumed_at` at the instant of any
/// transition and the two rules agree there — this eliminates discontinuities
/// on pause/resume/speed-change/reverse without bespoke per-operation math.
///
/// Every public mutator (`play`/`pause`/`stop`/`seek`/`set_speed`/`set_root`)
/// is a thin wrapper around a private `_at(..., now: Instant)` variant that
/// does the actual work against an explicit instant. The public API reads
/// `Instant::now()` internally (the correct default for real usage driven by
/// event handlers), while the private `_at` variants are the seam unit tests
/// use to drive the whole state machine deterministically with fabricated
/// instants, exactly like [`update`](TimelineController::update) already does.
#[derive(Debug)]
pub struct TimelineController {
    root: Box<dyn Timeline>,
    /// Virtual playhead, banked while not `Playing`.
    accumulated: Duration,
    /// Wall-clock instant `accumulated` was last rebased from; `Some` only
    /// while `Playing`.
    resumed_at: Option<Instant>,
    run_state: RunState,
    speed: f64,
    finished: bool,
}

impl TimelineController {
    /// Starts playing immediately, matching `Animation::new`'s implicit
    /// "starts now" behavior. Use `.paused()` to build in a paused state
    /// instead.
    pub fn new(root: impl Timeline + 'static) -> Self {
        TimelineController {
            root: Box::new(root),
            accumulated: Duration::ZERO,
            resumed_at: Some(Instant::now()),
            run_state: RunState::Playing,
            speed: 1.0,
            finished: false,
        }
    }

    #[cfg(test)]
    fn new_at(root: impl Timeline + 'static, now: Instant) -> Self {
        TimelineController {
            root: Box::new(root),
            accumulated: Duration::ZERO,
            resumed_at: Some(now),
            run_state: RunState::Playing,
            speed: 1.0,
            finished: false,
        }
    }

    pub fn paused(mut self) -> Self {
        self.resumed_at = None;
        self.run_state = RunState::Paused;
        self
    }

    pub fn run_state(&self) -> RunState {
        self.run_state
    }

    pub fn is_playing(&self) -> bool {
        self.run_state == RunState::Playing
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Current progress in `0.0..=1.0`, normalized against the root's
    /// `seek_duration()` (so it stays meaningful even for an
    /// unbounded/looping root). Computes the *live* position while playing
    /// (not just the last-banked `accumulated`, which would otherwise read
    /// stale between pause/seek/speed-change calls).
    pub fn progress(&self) -> f64 {
        self.progress_at(Instant::now())
    }

    fn progress_at(&self, now: Instant) -> f64 {
        let elapsed = self.virtual_elapsed(now);
        let total = self.root.seek_duration();
        if total.is_zero() {
            1.0
        } else {
            (elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
        }
    }

    /// `Paused`/`Stopped` -> `Playing`, re-pinning the wall-clock anchor
    /// without changing the banked position — resumes from where it was
    /// left.
    pub fn play(&mut self) {
        self.play_at(Instant::now());
    }

    fn play_at(&mut self, now: Instant) {
        if self.run_state != RunState::Playing {
            self.resumed_at = Some(now);
            self.run_state = RunState::Playing;
        }
    }

    /// `Playing` -> `Paused`, banking the current virtual position and
    /// freezing it in place.
    pub fn pause(&mut self) {
        self.pause_at(Instant::now());
    }

    fn pause_at(&mut self, now: Instant) {
        self.freeze_at(now);
        self.run_state = RunState::Paused;
    }

    /// Freezes in place exactly like `pause()` (does *not* reset to progress
    /// 0) but is tracked as a distinct `RunState::Stopped` — `play()`
    /// afterward resumes from the stopped position, not from `0`. This
    /// reconciles "release resources" language with "play() resumes from
    /// where it was left": a controller can't be both removed from the
    /// polling loop and resumable without a re-registration API, so `stop()`
    /// stays registered/resumable and only differs from `pause()` in the
    /// `RunState` label (useful for a future UI distinguishing "paused" from
    /// "stopped", and as a hook for a later redraw-while-idle optimization).
    pub fn stop(&mut self) {
        self.stop_at(Instant::now());
    }

    fn stop_at(&mut self, now: Instant) {
        self.freeze_at(now);
        self.run_state = RunState::Stopped;
    }

    /// `progress` in `0.0..=1.0` of the root's `seek_duration()`. Applies
    /// immediately (visible without waiting for a frame tick) and does not
    /// change `run_state` — if already playing, playback continues forward
    /// from the new position; if paused/stopped, the position is simply
    /// updated.
    pub fn seek(&mut self, progress: f64) {
        self.seek_at(progress, Instant::now());
    }

    fn seek_at(&mut self, progress: f64, now: Instant) {
        let total = self.root.seek_duration();
        self.accumulated = total.mul_f64(progress.clamp(0.0, 1.0));
        if self.run_state == RunState::Playing {
            self.resumed_at = Some(now);
        }
        let status = self.root.seek(self.accumulated, ResolveMode::Seek);
        self.finished = status.finished;
    }

    /// Rebases position under the *old* speed first (so the change takes
    /// effect from now, with no jump), then updates `speed`. Negative speed
    /// is genuine reverse playback — no separate mechanism.
    pub fn set_speed(&mut self, speed: f64) {
        self.set_speed_at(speed, Instant::now());
    }

    fn set_speed_at(&mut self, speed: f64, now: Instant) {
        if self.run_state == RunState::Playing {
            self.accumulated = self.virtual_elapsed(now);
            self.resumed_at = Some(now);
        }
        self.speed = speed;
    }

    /// Swaps in a freshly-built subtree and restarts from `0`, keeping this
    /// controller's identity/speed/run_state stable.
    pub fn set_root(&mut self, root: impl Timeline + 'static) {
        self.set_root_at(root, Instant::now());
    }

    fn set_root_at(&mut self, root: impl Timeline + 'static, now: Instant) {
        self.root = Box::new(root);
        self.accumulated = Duration::ZERO;
        self.finished = false;
        if self.run_state == RunState::Playing {
            self.resumed_at = Some(now);
        }
    }

    /// Called once per frame by `AnimationManager`'s poll. Returns whether
    /// the whole tree is naturally, terminally finished (used to prune).
    pub fn update(&mut self, frame_time: &Instant) -> bool {
        let elapsed = self.virtual_elapsed(*frame_time);
        let status = self.root.seek(elapsed, ResolveMode::Playback);
        self.finished = status.finished;
        self.finished
    }

    fn freeze_at(&mut self, now: Instant) {
        if self.run_state == RunState::Playing {
            self.accumulated = self.virtual_elapsed(now);
            self.resumed_at = None;
        }
    }

    /// `accumulated + (now - resumed_at) * speed`, done in signed `f64`
    /// seconds and clamped to `>= 0.0` before converting back to `Duration`
    /// (which is unsigned and would otherwise panic the first time reverse
    /// playback drove this below zero).
    fn virtual_elapsed(&self, now: Instant) -> Duration {
        match self.resumed_at {
            Some(anchor) => {
                let delta_secs = now.saturating_duration_since(anchor).as_secs_f64() * self.speed;
                let secs = (self.accumulated.as_secs_f64() + delta_secs).max(0.0);
                Duration::from_secs_f64(secs)
            }
            None => self.accumulated,
        }
    }
}

/// Cheap `Rc<RefCell<_>>` handle returned by
/// [`AnimationManager::insert_controller`](crate::animation::AnimationManager::insert_controller).
/// Clone freely — all clones share the *same* running controller (this is a
/// handle, not a value type, unlike `Animation<T>` which is independently
/// cloneable).
#[derive(Clone, Debug)]
pub struct TimelineHandle {
    inner: Rc<RefCell<TimelineController>>,
}

impl TimelineHandle {
    pub fn play(&self) {
        self.inner.borrow_mut().play();
    }

    pub fn pause(&self) {
        self.inner.borrow_mut().pause();
    }

    pub fn stop(&self) {
        self.inner.borrow_mut().stop();
    }

    pub fn seek(&self, progress: f64) {
        self.inner.borrow_mut().seek(progress);
    }

    pub fn set_speed(&self, speed: f64) {
        self.inner.borrow_mut().set_speed(speed);
    }

    pub fn set_root(&self, root: impl Timeline + 'static) {
        self.inner.borrow_mut().set_root(root);
    }

    pub fn is_finished(&self) -> bool {
        self.inner.borrow().is_finished()
    }

    pub fn is_playing(&self) -> bool {
        self.inner.borrow().is_playing()
    }

    pub fn progress(&self) -> f64 {
        self.inner.borrow().progress()
    }

    pub(crate) fn new(controller: TimelineController) -> (Self, Rc<RefCell<TimelineController>>) {
        let inner = Rc::new(RefCell::new(controller));
        (TimelineHandle { inner: inner.clone() }, inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::keyframes::Keyframes;
    use crate::state::{LocalState, ReadState};

    use super::*;

    fn one_second_leaf(state: &LocalState<f64>) -> Keyframes<f64> {
        Keyframes::new(state.clone(), 0.0).to(1.0, Duration::from_secs(1))
    }

    #[test]
    fn basic_playback_progresses_and_finishes() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new_at(one_second_leaf(&state), t0);

        controller.update(&t0);
        assert_eq!(*state.value(), 0.0);

        controller.update(&(t0 + Duration::from_millis(250)));
        assert_eq!(*state.value(), 0.25);

        let finished = controller.update(&(t0 + Duration::from_secs(2)));
        assert!(finished);
        assert_eq!(*state.value(), 1.0);
    }

    #[test]
    fn pause_freezes_exactly_in_place() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new_at(one_second_leaf(&state), t0);

        controller.update(&(t0 + Duration::from_millis(300)));
        let value_a = *state.value();

        // Freeze at exactly the fabricated t0+300ms position, not real "now".
        controller.pause_at(t0 + Duration::from_millis(300));
        controller.update(&(t0 + Duration::from_millis(900)));
        assert_eq!(*state.value(), value_a);
    }

    #[test]
    fn play_resumes_without_a_jump() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new_at(one_second_leaf(&state), t0);

        controller.update(&(t0 + Duration::from_millis(300)));
        controller.pause_at(t0 + Duration::from_millis(300));
        controller.update(&(t0 + Duration::from_millis(900))); // ignored while paused

        controller.play_at(t0 + Duration::from_millis(900));
        controller.update(&(t0 + Duration::from_millis(900) + Duration::from_millis(200)));

        // Virtual elapsed should be 300ms + 200ms = 500ms, NOT 900ms + 200ms.
        assert_eq!(*state.value(), 0.5);
    }

    #[test]
    fn stop_freezes_in_place_and_play_resumes_from_there_not_zero() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new_at(one_second_leaf(&state), t0);

        controller.update(&(t0 + Duration::from_millis(400)));
        controller.stop_at(t0 + Duration::from_millis(400));
        assert_eq!(controller.run_state(), RunState::Stopped);

        controller.update(&(t0 + Duration::from_secs(5))); // ignored while stopped
        assert_eq!(*state.value(), 0.4);

        controller.play_at(t0 + Duration::from_secs(5));
        controller.update(&(t0 + Duration::from_secs(5) + Duration::from_millis(100)));
        assert_eq!(*state.value(), 0.5); // 400ms + 100ms, not reset to 0
    }

    #[test]
    fn seek_while_paused_applies_immediately() {
        let state = LocalState::new(0.0f64);
        let mut controller = TimelineController::new(one_second_leaf(&state)).paused();

        controller.seek(0.5);
        assert_eq!(*state.value(), 0.5);
    }

    #[test]
    fn seek_while_playing_does_not_pause() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new(one_second_leaf(&state));

        controller.update(&t0);
        controller.seek_at(0.5, t0);
        assert!(controller.is_playing());

        controller.update(&(t0 + Duration::from_millis(200)));
        assert_eq!(*state.value(), 0.7);
    }

    #[test]
    fn set_speed_scales_the_apparent_rate_and_supports_reverse() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new(one_second_leaf(&state));

        controller.set_speed_at(2.0, t0);
        controller.update(&(t0 + Duration::from_millis(250)));
        assert_eq!(*state.value(), 0.5); // 2x speed: 250ms -> 500ms virtual

        let t1 = t0 + Duration::from_millis(250);
        controller.seek_at(0.5, t1);
        controller.set_speed_at(-1.0, t1);
        controller.update(&(t1 + Duration::from_millis(200)));
        assert!(*state.value() < 0.5); // reversing: value should have decreased
    }

    #[test]
    fn reverse_past_start_clamps_without_panicking() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new(one_second_leaf(&state));
        controller.seek_at(0.1, t0);
        controller.set_speed_at(-1.0, t0);

        // Advance wall-clock far enough that virtual elapsed would go negative.
        controller.update(&(t0 + Duration::from_secs(10)));

        assert_eq!(*state.value(), 0.0);
    }

    #[test]
    fn progress_and_is_playing_reflect_state() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new_at(one_second_leaf(&state), t0);
        assert!(controller.is_playing());

        // progress_at computes the LIVE position while playing, not just the
        // last-banked `accumulated` (which update() alone never touches).
        controller.update(&(t0 + Duration::from_millis(250)));
        assert_eq!(controller.progress_at(t0 + Duration::from_millis(250)), 0.25);

        controller.pause_at(t0 + Duration::from_millis(250));
        assert!(!controller.is_playing());
        assert_eq!(controller.progress_at(t0 + Duration::from_secs(99)), 0.25); // frozen once paused
    }

    #[test]
    fn is_finished_is_idempotent_past_completion() {
        let state = LocalState::new(0.0f64);
        let t0 = Instant::now();
        let mut controller = TimelineController::new_at(one_second_leaf(&state), t0);

        assert!(controller.update(&(t0 + Duration::from_secs(2))));
        assert!(controller.is_finished());
        assert!(controller.update(&(t0 + Duration::from_secs(3))));
        assert!(controller.is_finished());
    }
}
