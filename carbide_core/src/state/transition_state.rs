use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use carbide::state::IntoState;
use crate::animation::{Animatable, ease_in_out};
use crate::environment::Environment;
use crate::state::{AnyReadState, AnyState, NewStateSync, State, StateContract, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub struct TransitionState<T, S>
    where
        T: StateContract + Animatable<T>,
        S: State<T=T>,
{
    /// The value contained as the state
    inner: S,
    duration: Duration,

    animation_curve: fn(f64) -> f64,

    custom_interpolation: fn(&T, &T, f64) -> T,

    range: Rc<RefCell<Option<(Instant, T, T)>>>,
}

impl TransitionState<f64, f64> {
    pub fn new<T: StateContract + Animatable<T>, S: IntoState<T>>(state: S) -> TransitionState<T, S::Output> {
        TransitionState {
            inner: state.into_state(),
            duration: Duration::from_secs_f64(1.0),
            animation_curve: ease_in_out,
            custom_interpolation: T::interpolate,
            range: Rc::new(RefCell::new(None)),
        }
    }
}

impl<T: StateContract + Animatable<T>, S: State<T=T>> TransitionState<T, S> {
    pub fn interpolation(mut self, interpolation: fn(&T, &T, f64) -> T) -> Self {
        self.custom_interpolation = interpolation;
        self
    }

    pub fn curve(mut self, curve: fn(f64) -> f64) -> Self {
        self.animation_curve = curve;
        self
    }

    pub fn duration(mut self, duration: Duration) -> TransitionState<T, S> {
        self.duration = duration;
        self
    }

    fn progression(&mut self) -> Option<T> {
        let res = match &*self.range.borrow() {
            None => None,
            Some((time, start, end)) => {
                let now = Instant::now();

                let duration = now - *time;

                let percentage = duration.as_secs_f64() / self.duration.as_secs_f64();

                if percentage > 1.0 {
                    None
                } else {
                    let animated = (self.animation_curve)(percentage);

                    let res = (self.custom_interpolation)(start, end, animated);

                    Some(res)
                }
            }
        };

        if res.is_none() && self.range.borrow().is_some() {
            *self.range.borrow_mut() = None;
        }

        res
    }
}

impl<T: StateContract + Animatable<T>, S: State<T=T>> NewStateSync for TransitionState<T, S> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self.progression() {
            None => false,
            Some(new) => {
                env.request_animation_frame();
                self.inner.set_value(new);
                true
            }
        }
    }
}

impl<T: StateContract + Animatable<T>, S: State<T=T>> AnyReadState for TransitionState<T, S> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.inner.value()
    }
}

impl<T: StateContract + Animatable<T>, S: State<T=T>> AnyState for TransitionState<T, S> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        unimplemented!() // Use value ref mut drop function
    }

    fn set_value_dyn(&mut self, value: T) {
        let current: T = self.inner.value().clone();
        let target = value;

        *self.range.borrow_mut() = Some((
            Instant::now(),
            current,
            target
        ));
    }
}