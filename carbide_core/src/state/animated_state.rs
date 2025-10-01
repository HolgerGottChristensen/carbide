use std::fmt::Debug;

use crate::animation::animation_curve::linear;
use crate::animation::{Animatable, AnimationManager};
use crate::environment::Environment;
use crate::state::util::value_cell::ValueRef;
use crate::state::{AnyReadState, RMap1, StateSync};
use crate::state::Map1;
use crate::time::*;

#[derive(Clone, Debug)]
pub enum RepeatMode {
    None,
    FromBeginning,
    Alternate,
}

#[derive(Clone, Debug)]
pub struct AnimatedState {
    percent: f64,
    start_time: Instant,
    duration: Duration,
    repeat_mode: RepeatMode,
    repeat_count: Option<u32>,
    frame_time: Instant,
    animation_curve: fn(f64) -> f64,
}

impl AnimatedState {
    pub fn linear() -> AnimatedState {
        Self::custom(linear)
    }

    pub fn custom(curve: fn(f64) -> f64) -> AnimatedState {
        let now = Instant::now();
        AnimatedState {
            percent: 0.0,
            start_time: now,
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            frame_time: now,
            animation_curve: curve,
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

    pub fn range<T: Animatable<T> + Copy + 'static + Debug>(
        self,
        from: T,
        to: T,
    ) -> RMap1<impl Fn(&f64)->T + Clone, f64, T, AnimatedState> {
        Map1::read_map(self, move |t| {
            from.interpolate(&to, *t)
        })
    }

    pub fn calc_percentage(&mut self) {
        let duration = self.frame_time - self.start_time;

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
        if let Some(manager) = env.get_mut::<AnimationManager>() {
            manager.request_animation_frame();
            self.frame_time = manager.frame_time();
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
