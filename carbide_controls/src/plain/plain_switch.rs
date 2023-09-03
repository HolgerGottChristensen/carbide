use std::fmt::{Debug, Formatter};
use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::state::{AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, State, StateExtNew, TState};
use carbide_core::widget::{CommonWidget, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

use crate::PlainButton;

pub trait PlainSwitchDelegate: Clone + 'static {
    fn call(&self, focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>;
}

impl<K> PlainSwitchDelegate for K where K: Fn(Box<dyn AnyState<T=Focus>>, Box<dyn AnyState<T=bool>>) -> Box<dyn Widget> + Clone + 'static {
    fn call(&self, item: Box<dyn AnyState<T=Focus>>, index: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget> {
        self(item, index)
    }
}

/// # A plain switch widget
/// This widget contains the basic logic for a switch component, without any styling.
/// It can be styled by setting the delegate, using the delegate method.
///
/// For a styled version, use [crate::Switch] instead.
#[derive(Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainSwitch<F, C, D> where
    F: State<T=Focus> + Clone,
    C: State<T=bool> + Clone,
    D: PlainSwitchDelegate,
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: D,
    #[state] checked: C,
}

impl PlainSwitch<Focus, bool, fn(Box<dyn AnyState<T=Focus>>, Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>> {
    pub fn new<C: IntoState<bool>>(checked: C) -> PlainSwitch<TState<Focus>, C::Output, fn(Box<dyn AnyState<T=Focus>>, Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainSwitch::default_delegate,
        )
    }

    fn default_delegate(focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(checked.clone(), |is_checked| {
            if *is_checked {
                EnvironmentColor::Green
            } else {
                EnvironmentColor::Red
            }
        });

        let val = Map2::read_map(checked, focus, |checked: &bool, focus: &Focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate> PlainSwitch<F, C, D> {

    pub fn delegate<D2: PlainSwitchDelegate>(
        self,
        delegate: D2,
    ) -> PlainSwitch<F, C, D2> {
        let checked = self.checked;
        let focus_state = self.focus;

        Self::new_internal(checked, focus_state, delegate)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainSwitch<F2::Output, C, D> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate)
    }

    fn new_internal<F2: State<T=Focus> + Clone, C2: State<T=bool> + Clone, D2: PlainSwitchDelegate>(
        checked: C2,
        focus: F2,
        delegate: D2,
    ) -> PlainSwitch<F2, C2, D2> {

        let delegate_widget = delegate.call(focus.as_dyn(), checked.as_dyn());

        let button = Box::new(PlainButton::new(delegate_widget)
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                let current = !*checked.value();
                checked.set_value(current);

                if *focus.value() != Focus::Focused {
                    focus.set_value(Focus::FocusRequested);
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .on_click_outside(capture!([focus], |env: &mut Environment| {
                if *focus.value() == Focus::Focused {
                    focus.set_value(Focus::FocusReleased);
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone()));

        let child = button;

        PlainSwitch {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            checked,
        }
    }

}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate> Focusable for PlainSwitch<F, C, D> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate> CommonWidget for PlainSwitch<F, C, D> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate> WidgetExt for PlainSwitch<F, C, D> {}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone, D: PlainSwitchDelegate> Debug for PlainSwitch<F, C, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainSwitch")
            .field("id", &self.id)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("focus", &self.focus)
            .field("checked", &self.checked)
            .finish()
    }
}
