use std::fmt::{Debug, Formatter};
use dyn_clone::DynClone;
use carbide_core::render::RenderContext;
use carbide_core::state::IntoReadState;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Color, Dimension, Position, Rect};
use crate::draw::draw_style::DrawStyle;
use crate::environment::Environment;
use crate::layout::{Layout, LayoutContext};
use crate::render::Render;
use crate::state::{ReadState, StateContract, StateSync};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

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
    id: WidgetId,
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

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> StateSync for OnChange<W, T, S, F> {
    fn capture_state(&mut self, env: &mut Environment) {
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

    fn release_state(&mut self, env: &mut Environment) {
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
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> WidgetExt for OnChange<W, T, S, F> {}

impl<W: Widget, T: StateContract + PartialEq, S: ReadState<T=T>, F: Changed<T>> Debug for OnChange<W, T, S, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnChange")
            .field("state", &self.state)
            .field("prev", &self.prev)
            .field("child", &self.child)
            .finish()
    }
}