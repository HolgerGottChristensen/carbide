use std::fmt::{Debug, Formatter};
use std::time::{Duration, Instant};

use crate::animation::animatable::Animatable;
use crate::state::{linear, RepeatMode, State, StateContract};

#[derive(Clone)]
pub struct Animation<T>
where
    T: StateContract,
{
    start_time: Instant,
    duration: Duration,
    repeat_mode: RepeatMode,
    repeat_count: Option<u32>,
    animation_curve: fn(f64) -> f64,
    custom_interpolation: fn(&T, &T, f64) -> T,
    state: Box<dyn State<T>>,
    from: T,
    to: T,
}

impl<T: StateContract + Animatable<T>> Animation<T> {
    pub fn new<S: Into<Box<dyn State<T>>>>(state: S, from: T, to: T) -> Self {
        Animation {
            start_time: Instant::now(),
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            animation_curve: linear,
            custom_interpolation: T::interpolate,
            state: state.into(),
            from,
            to,
        }
    }
}

impl<T: StateContract> Animation<T> {
    pub fn new_custom<S: Into<Box<dyn State<T>>>>(
        state: S,
        from: T,
        to: T,
        interpolation: fn(&T, &T, f64) -> T,
    ) -> Self {
        Animation {
            start_time: Instant::now(),
            duration: Duration::new(1, 0),
            repeat_mode: RepeatMode::None,
            repeat_count: None,
            animation_curve: linear,
            custom_interpolation: interpolation,
            state: state.into(),
            from,
            to,
        }
    }

    pub fn interpolation(mut self, interpolation: fn(&T, &T, f64) -> T) -> Self {
        self.custom_interpolation = interpolation;
        self
    }

    pub fn from(mut self, from: T) -> Self {
        self.from = from;
        self
    }

    pub fn curve(mut self, curve: fn(f64) -> f64) -> Self {
        self.animation_curve = curve;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn repeat_mode(mut self, mode: RepeatMode) -> Self {
        self.repeat_mode = mode;
        self
    }

    pub fn update(&mut self, current_time: &Instant) -> bool {
        let duration = *current_time - self.start_time;

        match self.repeat_mode {
            RepeatMode::None => {
                let un_curved_percentage = if duration > self.duration {
                    1.0
                } else {
                    duration.as_secs_f64() / self.duration.as_secs_f64()
                };

                let percentage = (self.animation_curve)(un_curved_percentage);

                let interpolated = (self.custom_interpolation)(&self.from, &self.to, percentage);
                self.state.set_value(interpolated);

                un_curved_percentage == 1.0
            }
            RepeatMode::FromBeginning => {
                let percentage = duration.as_secs_f64() / self.duration.as_secs_f64() % 1.0;

                let percentage = (self.animation_curve)(percentage);

                let interpolated = (self.custom_interpolation)(&self.from, &self.to, percentage);
                self.state.set_value(interpolated);

                false
            }
            RepeatMode::Alternate => {
                let temp = duration.as_secs_f64() / self.duration.as_secs_f64() % 2.0;
                let percentage = if temp >= 1.0 { 2.0 - temp } else { temp };

                let percentage = (self.animation_curve)(percentage);

                let interpolated = (self.custom_interpolation)(&self.from, &self.to, percentage);
                self.state.set_value(interpolated);

                false
            }
        }
    }
}

impl<T: StateContract> Debug for Animation<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animation").finish()
    }
}
