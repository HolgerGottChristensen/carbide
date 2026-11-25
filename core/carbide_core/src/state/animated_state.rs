use std::fmt::Debug;

use crate::animation::animation_curve::linear;
use crate::animation::{Animatable, AnimationManager};
use crate::cell::ValueRef;
use crate::environment::Environment;
use crate::state::{AnyReadState, IntoReadState, RMap1, ReadState, StateSync};
use crate::state::Map1;
use crate::time::*;

#[derive(Clone, Debug)]
pub enum RepeatMode {
    None,
    FromBeginning,
    Alternate,
}

/// A value from 0 to 1 that animates over time, read as a state: the share of the animation's
/// duration that has passed, shaped by its animation curve.
///
/// The animation's clock is the [`AnimationManager`]'s frame time, and it starts at the first
/// sync, not when the state is built: an animation starts when it is first shown, and a test
/// harness's simulated clock drives it exactly.
///
/// While it is running, it requests an animation frame on every sync, until it has finished. A
/// repeating animation never finishes. An animation can be paused with
/// [`running`](AnimatedState::running): while paused, it keeps its value and requests no frames,
/// and when it runs again it continues from where it was.
#[derive(Clone, Debug)]
pub struct AnimatedState {
    percent: f64,
    duration: Duration,
    repeat_mode: RepeatMode,
    repeat_count: Option<u32>,
    animation_curve: fn(f64) -> f64,
    /// Whether the animation runs. While it doesn't, its clock is stopped.
    running: Box<dyn AnyReadState<T=bool>>,
    /// The time animated in earlier runs, before the latest pause.
    elapsed: Duration,
    /// When the current run started, by the animation manager's clock. `None` until the first
    /// sync while running, and while paused.
    run_start: Option<Instant>,
    /// The animation manager's frame time at the last sync.
    frame_time: Option<Instant>,
}

impl AnimatedState {
    pub fn linear() -> AnimatedState {
        Self::custom(linear)
    }

    pub fn custom(curve: fn(f64) -> f64) -> AnimatedState {
        AnimatedState {
            percent: 0.0,
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            animation_curve: curve,
            running: Box::new(true),
            elapsed: Duration::ZERO,
            run_start: None,
            frame_time: None,
        }
    }

    pub fn duration(mut self, duration: Duration) -> AnimatedState {
        self.duration = duration;
        self
    }

    pub fn repeat_alternate(mut self) -> AnimatedState {
        self.repeat_mode = RepeatMode::Alternate;
        self
    }

    pub fn repeat(mut self) -> AnimatedState {
        self.repeat_mode = RepeatMode::FromBeginning;
        self
    }

    pub fn count(mut self, count: u32) -> AnimatedState {
        self.repeat_count = Some(count);
        self
    }

    /// Run the animation only while `running` is true. While it is false, the animation keeps
    /// its value and requests no animation frames; when it is true again, the animation continues
    /// from where it was paused.
    pub fn running<R: IntoReadState<bool>>(mut self, running: R) -> AnimatedState {
        self.running = Box::new(running.into_read_state());
        self
    }

    pub fn range<T: Animatable<T> + Copy + 'static + Debug>(
        self,
        from: T,
        to: T,
    ) -> RMap1<impl Fn(&f64)->T + Clone, f64, T, AnimatedState> {
        Map1::read_map(self, move |t| {
            from.interpolate(&to, *t)
        })
    }

    /// The time animated so far: earlier runs, plus the current run up to the last sync's frame
    /// time.
    fn animated_time(&self) -> Duration {
        match (self.run_start, self.frame_time) {
            (Some(start), Some(now)) => self.elapsed + now.duration_since(start),
            _ => self.elapsed,
        }
    }

    /// Whether the animation has reached its end and stays there. Repeating animations never do.
    fn finished(&self) -> bool {
        matches!(self.repeat_mode, RepeatMode::None) && self.animated_time() >= self.duration
    }

    /// Recalculate the value from the time animated as of the last sync.
    pub fn calc_percentage(&mut self) {
        let duration = self.animated_time();

        let percentage = match self.repeat_mode {
            RepeatMode::None => {
                if duration > self.duration {
                    1.0
                } else {
                    duration.as_secs_f64() / self.duration.as_secs_f64()
                }
            }
            RepeatMode::FromBeginning => duration.as_secs_f64() / self.duration.as_secs_f64() % 1.0,
            RepeatMode::Alternate => {
                let temp = duration.as_secs_f64() / self.duration.as_secs_f64() % 2.0;
                if temp >= 1.0 {
                    2.0 - temp
                } else {
                    temp
                }
            }
        };

        self.percent = (self.animation_curve)(percentage);
    }
}

impl StateSync for AnimatedState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.running.sync(env);
        let running = *self.running.value();

        let Some(manager) = env.get_mut::<AnimationManager>() else {
            return false;
        };

        let now = manager.frame_time();
        self.frame_time = Some(now);

        if running {
            // The clock starts, or starts again after a pause, at this frame.
            self.run_start.get_or_insert(now);
            self.calc_percentage();

            if !self.finished() {
                manager.request_animation_frame();
            }
        } else if let Some(start) = self.run_start.take() {
            // Pausing: keep the time animated so far, and stop the clock.
            self.elapsed += now.duration_since(start);
            self.calc_percentage();
        }

        false
    }
}

impl AnyReadState for AnimatedState {
    type T = f64;
    fn value_dyn(&self) -> ValueRef<'_, f64> {
        ValueRef::Borrow(&self.percent)
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::AnimationManager;
    use crate::environment::Environment;
    use crate::state::{AnimatedState, LocalState, ReadState, State, StateSync};
    use crate::time::{Duration, Instant};

    /// Sync `state` with an animation manager whose frame time is `at`, and return its value and
    /// whether it requested an animation frame.
    fn sync_at(state: &mut AnimatedState, manager: &mut AnimationManager, at: Instant) -> (f64, bool) {
        manager.update_frame_time_at(at);
        while manager.take_frame() {}

        let mut env = Environment::new();
        env.with::<AnimationManager>(manager, |env| {
            state.sync(env);
        });

        (*state.value(), manager.number_of_animation_frames() > 0)
    }

    #[test]
    fn the_clock_starts_at_the_first_sync() {
        let mut state = AnimatedState::linear().duration(Duration::from_secs(4));
        let mut manager = AnimationManager::new();
        let start = Instant::now() + Duration::from_secs(10);

        assert_eq!(sync_at(&mut state, &mut manager, start), (0.0, true));
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(1)), (0.25, true));
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(2)), (0.5, true));
    }

    #[test]
    fn a_finished_animation_requests_no_frames() {
        let mut state = AnimatedState::linear().duration(Duration::from_secs(1));
        let mut manager = AnimationManager::new();
        let start = Instant::now();

        sync_at(&mut state, &mut manager, start);
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(2)), (1.0, false));
    }

    #[test]
    fn a_paused_animation_keeps_its_value_and_requests_no_frames() {
        let mut running = LocalState::new(true);
        let mut state = AnimatedState::linear().duration(Duration::from_secs(4)).repeat().running(running.clone());
        let mut manager = AnimationManager::new();
        let start = Instant::now();

        sync_at(&mut state, &mut manager, start);
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(1)), (0.25, true));

        *running.value_mut() = false;
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(2)), (0.5, false));
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(10)), (0.5, false));

        // Running again continues from where it was paused.
        *running.value_mut() = true;
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(11)), (0.5, true));
        assert_eq!(sync_at(&mut state, &mut manager, start + Duration::from_secs(12)), (0.75, true));
    }
}
