use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use carbide::animation::AnimationManager;
use crate::animation::{Animatable, ease_in_out};
use crate::environment::{Environment, EnvironmentStack};
use crate::state::{AnyReadState, Fn2, Functor, InnerState, IntoReadState, Map1, StateSync, ReadState, RMap1, StateContract, ValueCell, ValueRef};

#[derive(Clone)]
pub struct TransitionState<T, S>
    where
        T: StateContract + Animatable<T> + PartialEq,
        S: ReadState<T=T>,
{
    /// The value contained as the state
    inner: S,
    value: InnerState<T>,

    duration: Duration,
    curve: fn(f64) -> f64,
    interpolation: fn(&T, &T, f64) -> T,

    transition: Rc<RefCell<Option<Transition<T>>>>,

    initialized: Rc<AtomicBool>,
}

struct Transition<T> where T: StateContract + Animatable<T> + PartialEq {
    start: Instant,
    from: T,
    to: T,
}

impl TransitionState<f64, f64> {
    pub fn new<T: StateContract + Animatable<T> + Default + PartialEq, S: ReadState<T=T>>(state: S) -> TransitionState<T, S> {
        TransitionState {
            inner: state,
            value: Rc::new(ValueCell::new(T::default())),
            duration: Duration::new(1, 0),
            curve: ease_in_out,
            interpolation: T::interpolate,
            transition: Rc::new(RefCell::new(None)),
            initialized: Default::default(),
        }
    }
}

impl<T: StateContract + Animatable<T> + PartialEq, S: ReadState<T=T>> TransitionState<T, S> {
    pub fn interpolation(mut self, interpolation: fn(&T, &T, f64) -> T) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn curve(mut self, curve: fn(f64) -> f64) -> Self {
        self.curve = curve;
        self
    }

    pub fn duration(mut self, duration: Duration) -> TransitionState<T, S> {
        self.duration = duration;
        self
    }

    /*fn progression(&mut self) -> Option<T> {
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
    }*/
}

impl<T: StateContract + Animatable<T> + PartialEq, S: ReadState<T=T>> StateSync for TransitionState<T, S> {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        let res = self.inner.sync(env);

        if let Some(manager) = env.get_mut::<AnimationManager>() {
            if self.transition.borrow().is_some() {
                manager.request_animation_frame();
            } else if &*self.value.borrow() != &*self.inner.value() {
                manager.request_animation_frame();
            }
        }

        res
    }
}

impl<T: StateContract + Animatable<T> + PartialEq, S: ReadState<T=T>> AnyReadState for TransitionState<T, S> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        if !self.initialized.load(Ordering::Relaxed) {
            self.initialized.store(true, Ordering::Relaxed);
            *self.value.borrow_mut() = self.inner.value().clone();
        }

        let update = if let Some(transition) = &*self.transition.borrow() {
            transition.to != *self.inner.value()
        } else if &*self.value.borrow() != &*self.inner.value() {
            true
        } else {
            false
        };

        if update {
            let transition = Transition {
                start: Instant::now(),
                from: self.value.borrow().clone(),
                to: self.inner.value().clone(),
            };

            *self.transition.borrow_mut() = Some(transition);
        }

        let remove = if let Some(transition) = &*self.transition.borrow() {
            let percentage = transition.start.elapsed().as_secs_f64() / self.duration.as_secs_f64();
            if percentage > 1.0 {
                *self.value.borrow_mut() = self.inner.value().clone();
            } else {
                let animated = (self.curve)(percentage);

                let res = (self.interpolation)(&transition.from, &transition.to, animated);

                *self.value.borrow_mut() = res;
            }

            transition.start.elapsed() > self.duration
        } else {
            false
        };

        if remove {
            *self.transition.borrow_mut() = None;
        }

        self.value.borrow()
    }
}

impl<T: StateContract + Animatable<T> + PartialEq, S: ReadState<T=T>> Debug for TransitionState<T, S> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T: StateContract, V: StateContract + Animatable<V> + PartialEq, S: ReadState<T=V>> Functor<T> for TransitionState<V, S> where TransitionState<V, S>: IntoReadState<T> {
    // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
    type Output<G: StateContract, F: Fn2<T, G>> = RMap1<F, T, G, <TransitionState<V, S> as IntoReadState<T>>::Output>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F> {
        Map1::read_map(self.into_read_state(), f)
    }
}

pub trait ReadStateExtTransition<T>: ReadState<T=T> + Sized + Clone + 'static where T: StateContract + Default + PartialEq + Animatable<T> {
    fn transition(&self) -> TransitionState<T, Self> {
        TransitionState::new(self.clone())
    }
}

impl<T: StateContract + Default + PartialEq + Animatable<T>, S> ReadStateExtTransition<T> for S where S: ReadState<T=T> + Sized + Clone + 'static {}
