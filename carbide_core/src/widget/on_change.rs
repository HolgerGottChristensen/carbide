use std::fmt::{Debug, Formatter};
use carbide::environment::Environment;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::state::{ReadState, StateContract};
use crate::widget::{CommonWidget, Empty, Widget, WidgetId, WidgetSync};

pub trait Changed<T: StateContract>: Fn(Option<&T>, &T) + Clone + 'static {}

impl<I, T: StateContract> Changed<T> for I where I: Fn(Option<&T>, &T) + Clone + 'static {}

/// https://www.hackingwithswift.com/quick-start/swiftui/how-to-run-some-code-when-state-changes-using-onchange
#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct OnChange<W, T, S, F> where
    W: Widget,
    T: StateContract + PartialEq,
    S: ReadState<T=T>,
    F: Changed<T>,
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: W,

    state: S,
    f: F,

    prev: Option<T>,
}

impl OnChange<Empty, (), (), fn(Option<&()>, &())> {
    #[carbide_default_builder2]
    pub fn new<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>>(child: W, state: S, f: F) -> OnChange<W, T, S, F> {
        OnChange {
            id: WidgetId::new(),
            child,
            state,
            f,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            prev: None,
        }
    }
}

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> OnChange<W, T, S, F> {

}

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> WidgetSync for OnChange<W, T, S, F> {
    fn sync(&mut self, env: &mut Environment) {
        self.state.sync(env);

        if let Some(val) = &mut self.prev {
            if &*self.state.value() != val {
                (self.f)(Some(val), &*self.state.value());
            }

            *val = self.state.value().clone();
        } else {
            (self.f)(None, &self.state.value());
            self.prev = Some(self.state.value().clone())
        }
    }
}

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> CommonWidget for OnChange<W, T, S, F> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> Debug for OnChange<W, T, S, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnChange")
            .field("state", &self.state)
            .field("prev", &self.prev)
            .field("child", &self.child)
            .finish()
    }
}