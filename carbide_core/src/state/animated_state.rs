use std::fmt::Debug;
use std::ops::{Add, DerefMut, Mul};
use std::time::{Duration, Instant};

use carbide_core::state::{AnyReadState, NewStateSync, RMap1};

use crate::animation::animation_curve::linear;
use crate::environment::Environment;
use crate::state::{AnyState, InnerState, Map1, ReadState, State, StateContract, TState};
use crate::state::util::value_cell::{ValueCell, ValueRef, ValueRefMut};
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
    frame_time: Option<InnerState<Instant>>,
    animation_curve: fn(f64) -> f64,
}

impl AnimatedState {
    pub fn linear(env: Option<&Environment>) -> Box<Self> {
        Self::custom(linear, env)
    }

    pub fn custom(curve: fn(f64) -> f64, env: Option<&Environment>) -> Box<Self> {
        Box::new(AnimatedState {
            percent: InnerState::new(ValueCell::new(0.0)),
            start_time: Instant::now(),
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            frame_time: env.map(|e| e.captured_time()),
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

    pub fn range<T: Mul<f64, Output = U> + Copy + 'static, U: Add>(
        self,
        from: T,
        to: T,
    ) -> RMap1<impl Fn(&f64)-><U as Add<U>>::Output + Clone, f64, <U as Add<U>>::Output, AnimatedState>
    where
        <T as Mul<f64>>::Output: Add<U>,
        <U as Add<U>>::Output: StateContract + Default + 'static,
    {
        Map1::read_map(self, move |t| {
            from * (1.0 - *t) + to * *t
        })
    }

    pub fn calc_percentage(&self) {
        let now = Instant::now();

        let current_time = self
            .frame_time
            .as_ref()
            .map(|time| time.borrow())
            .unwrap_or(ValueRef::Borrow(&now));

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

impl NewStateSync for AnimatedState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        env.request_animation_frame();
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
