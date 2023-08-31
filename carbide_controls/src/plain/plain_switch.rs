use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::focus::Refocus;
use carbide_core::state::{AnyState, IntoReadState, IntoState, LocalState, Map1, Map2, ReadState, State, StateExtNew, TState};
use carbide_core::widget::{CommonWidget, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack};

use crate::PlainButton;

/// # A plain switch widget
/// This widget contains the basic logic for a switch component, without any styling.
/// It can be styled by setting the delegate, using the delegate method.
///
/// For a styled version, use [crate::Switch] instead.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainSwitch<F, C> where
    F: State<T=Focus> + Clone,
    C: State<T=bool> + Clone
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>,
    #[state] checked: C,
}

impl PlainSwitch<Focus, bool> {
    pub fn new<C: IntoState<bool>>(checked: C) -> PlainSwitch<TState<Focus>, C::Output> {
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

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone> PlainSwitch<F, C> {

    pub fn delegate(
        self,
        delegate: fn(focus: Box<dyn AnyState<T=Focus>>, selected: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>,
    ) -> PlainSwitch<F, C> {
        let checked = self.checked;
        let focus_state = self.focus;

        Self::new_internal(checked, focus_state, delegate)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainSwitch<F2::Output, C> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate)
    }

    fn new_internal<F2: State<T=Focus> + Clone, C2: State<T=bool> + Clone>(
        checked: C2,
        focus: F2,
        delegate: fn(focus: Box<dyn AnyState<T=Focus>>, checked: Box<dyn AnyState<T=bool>>) -> Box<dyn Widget>,
    ) -> PlainSwitch<F2, C2> {

        let delegate_widget = delegate(focus.as_dyn(), checked.as_dyn());

        let button = Box::new(PlainButton::new(delegate_widget)
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                let current = !*checked.value();
                checked.set_value(current);

                if *focus.value() != Focus::Focused {
                    focus.set_value(Focus::FocusRequested);
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

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone> Focusable for PlainSwitch<F, C> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone> CommonWidget for PlainSwitch<F, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=bool> + Clone> WidgetExt for PlainSwitch<F, C> {}
