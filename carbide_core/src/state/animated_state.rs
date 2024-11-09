use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::state::{AnyReadState, StateSync, RMap1};
use crate::animation::{Animatable, AnimationManager};
use crate::animation::animation_curve::linear;
use crate::environment::{Environment, EnvironmentStack};
use crate::state::{AnyState, InnerState, Map1};
use crate::state::util::value_cell::{ValueCell, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub enum RepeatMode {
    None,
    FromBeginning,
    Alternate,
}

#[derive(Clone, Debug)]
pub struct AnimatedState {
    percent: InnerState<f64>,
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
        AnimatedState {
            percent: InnerState::new(ValueCell::new(0.0)),
            start_time: Instant::now(),
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            frame_time: Instant::now(),
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

    pub fn calc_percentage(&self) {
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

        if let Ok(mut borrow) = self.percent.try_borrow_mut() {
            *borrow.deref_mut() = (self.animation_curve)(percentage);
        }
    }
}

impl StateSync for AnimatedState {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {

        if let Some(manager) = env.get_mut::<AnimationManager>() {
            manager.request_animation_frame();
            self.frame_time = manager.frame_time();
        }

        false
    }
}

impl AnyReadState for AnimatedState {
    type T = f64;
    fn value_dyn(&self) -> ValueRef<f64> {
        self.calc_percentage();
        self.percent.borrow()
    }
}

impl AnyState for AnimatedState {
    fn value_dyn_mut(&mut self) -> ValueRefMut<f64> {
        self.calc_percentage();
        self.percent.borrow_mut()
    }

    fn set_value_dyn(&mut self, value: f64) {
        self.calc_percentage();
        *self.percent.borrow_mut() = value;
    }
}

/*impl Into<TState<f64>> for Box<AnimatedState> {
    fn into(self) -> TState<f64> {
        WidgetState::new(self)
    }
}*/
