use std::fmt::Debug;
use std::ops::{Add, DerefMut, Mul};
use std::time::{Duration, Instant};

use crate::environment::Environment;
use crate::state::{InnerState, MapOwnedState, State, StateContract, TState};
use crate::state::animation_curve::linear;
use crate::state::value_cell::{ValueCell, ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

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
    frame_time: InnerState<Instant>,
    animation_curve: fn(f64) -> f64,
}

impl AnimatedState {
    pub fn linear(env: &Environment) -> Box<Self> {
        Box::new(AnimatedState {
            percent: InnerState::new(ValueCell::new(0.0)),
            start_time: Instant::now(),
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            frame_time: env.captured_time(),
            animation_curve: linear,
        })
    }

    pub fn custom(curve: fn(f64) -> f64, env: &Environment) -> Box<Self> {
        Box::new(AnimatedState {
            percent: InnerState::new(ValueCell::new(0.0)),
            start_time: Instant::now(),
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            frame_time: env.captured_time(),
            animation_curve: curve,
        })
    }

    pub fn duration(mut self, duration: Duration) -> Box<Self> {
        self.duration = duration;
        Box::new(self)
    }

    pub fn repeat_alternate(mut self) -> Box<Self> {
        self.repeat_mode = RepeatMode::Alternate;
        Box::new(self)
    }

    pub fn repeat(mut self) -> Box<Self> {
        self.repeat_mode = RepeatMode::FromBeginning;
        Box::new(self)
    }

    pub fn count(mut self, count: u32) -> Box<Self> {
        self.repeat_count = Some(count);
        Box::new(self)
    }

    pub fn range<T: Mul<f64, Output=U> + Copy + 'static, U: Add>(
        self,
        from: T,
        to: T,
    ) -> TState<<U as Add>::Output>
        where
            <T as Mul<f64>>::Output: Add<U>,
            <U as Add<U>>::Output: StateContract + Default + 'static,
    {
        MapOwnedState::new(WidgetState::new(Box::new(self)), move |t: &f64, _: &_, _: &_| {
            from * (1.0 - *t) + to * *t
        })
            .into()
    }

    pub fn calc_percentage(&self) {
        let current_time = self.frame_time.borrow();
        let duration = *current_time - self.start_time;

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

impl State<f64> for AnimatedState {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<f64> {
        self.calc_percentage();
        self.percent.borrow()
    }

    fn value_mut(&mut self) -> ValueRefMut<f64> {
        self.calc_percentage();
        self.percent.borrow_mut()
    }

    fn set_value(&mut self, value: f64) {
        self.calc_percentage();
        *self.percent.borrow_mut() = value;
    }
}

impl Into<TState<f64>> for Box<AnimatedState> {
    fn into(self) -> TState<f64> {
        WidgetState::new(self)
    }
}
