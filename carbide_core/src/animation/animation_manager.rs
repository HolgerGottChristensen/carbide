use crate::environment::{Environment, EnvironmentKey};
use crate::animation::Animation;
use crate::animation::keyframes::{TimelineController, TimelineHandle};
use crate::state::StateContract;
use crate::time::*;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct AnimationManager {
    frame_count: u32,
    frame_time: Instant,
    out_of_band_animations: OutOfBandAnimation
}

impl AnimationManager {
    pub fn new() -> AnimationManager {
        AnimationManager {
            frame_count: 0,
            frame_time: Instant::now(),
            out_of_band_animations: OutOfBandAnimation(vec![]),
        }
    }

    pub fn number_of_animation_frames(&self) -> u32 {
        self.frame_count
    }

    pub fn request_animation_frame(&mut self) {
        self.frame_count = self.frame_count.max(1);
    }

    pub fn request_multiple_animation_frames(&mut self, n: u32) {
        self.frame_count = self.frame_count.max(n);
    }

    pub fn take_frame(&mut self) -> bool {
        if self.frame_count > 0 {
            self.frame_count -= 1;
            true
        } else {
            self.out_of_band_animations.0.len() > 0
        }
    }

    pub fn frame_time(&self) -> Instant {
        self.frame_time
    }

    pub fn update_frame_time(&mut self) {
        self.frame_time = Instant::now();
        self.out_of_band_animations.0.retain(|update_animation| !update_animation(&self.frame_time));
    }

    pub fn insert_animation<A: StateContract>(&mut self, animation: Animation<A>) {
        let poll = move |time: &Instant| {
            let mut animation = animation.clone();
            animation.update(time)
        };

        self.out_of_band_animations.0.push(Box::new(poll));
    }

    /// Registers a [`TimelineController`] for per-frame polling, reusing the
    /// exact same `out_of_band_animations` storage/poll-loop that
    /// [`insert_animation`](Self::insert_animation) uses. Unlike
    /// `insert_animation`, the controller is not cloned-and-discarded every
    /// poll — it's wrapped in `Rc<RefCell<_>>` so play/pause/seek/speed
    /// mutations made through the returned [`TimelineHandle`] persist across
    /// frames. No generic parameter is needed (unlike `insert_animation<A>`)
    /// since type-erasure already happened inside the `Timeline` tree.
    pub fn insert_controller(&mut self, controller: TimelineController) -> TimelineHandle {
        let (handle, inner) = TimelineHandle::new(controller);
        let poll = move |time: &Instant| inner.borrow_mut().update(time);
        self.out_of_band_animations.0.push(Box::new(poll));
        handle
    }

    pub fn get(env: &mut Environment, f: impl FnOnce(&mut AnimationManager)) {
        if let Some(manager) = env.get_mut::<AnimationManager>() {
            f(manager)
        }
    }
}

impl EnvironmentKey for AnimationManager {
    type Value = AnimationManager;
}

struct OutOfBandAnimation(Vec<Box<dyn Fn(&Instant) -> bool>>);

impl Debug for OutOfBandAnimation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutOfBandAnimation")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::keyframes::Keyframes;
    use crate::state::LocalState;

    use super::*;

    /// The only test in this feature needing more than a bare `Timeline`/
    /// `LocalState` — but still no window/widget-tree/wgpu backend, since
    /// `Environment::new()` is a real, standalone public constructor. This
    /// exercises the exact same access pattern the `animate!` macro and
    /// event-handler code use (`ctx.env.get_mut::<AnimationManager>()`).
    #[test]
    fn insert_controller_via_environment_drives_state_end_to_end() {
        let state = LocalState::new(0.0f64);
        let leaf = Keyframes::new(state.clone(), 0.0).to(1.0, Duration::from_secs(1));
        let controller = TimelineController::new(leaf);

        let mut manager = AnimationManager::new();
        let mut env = Environment::new();
        let mut handle_opt: Option<TimelineHandle> = None;

        env.with_mut::<AnimationManager>(&mut manager, |env| {
            if let Some(m) = env.get_mut::<AnimationManager>() {
                handle_opt = Some(m.insert_controller(controller));
            }
        });

        let handle = handle_opt.expect("AnimationManager should be reachable via env.get_mut inside with_mut");

        manager.update_frame_time();
        assert!(!handle.is_finished());
    }

    /// Characterization test for the accepted v1 tradeoff: a registered
    /// controller that is paused still causes `take_frame()` to keep
    /// returning `true` (redraw requested), since `take_frame()` only checks
    /// whether `out_of_band_animations` is non-empty, not whether any entry
    /// is actually `Playing`. This documents the current, intentionally
    /// accepted behavior so a future v1.1 fix changes this test loudly
    /// rather than the regression going unnoticed.
    #[test]
    fn paused_controller_still_requests_a_redraw_characterization() {
        let state = LocalState::new(0.0f64);
        let leaf = Keyframes::new(state.clone(), 0.0).to(1.0, Duration::from_secs(1));
        let mut controller = TimelineController::new(leaf);
        controller.pause();

        let mut manager = AnimationManager::new();
        let _handle = manager.insert_controller(controller);

        assert!(manager.take_frame());
    }
}