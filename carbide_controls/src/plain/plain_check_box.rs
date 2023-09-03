use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map2, Map4, ReadState, ReadStateExtNew, TState};
use carbide_core::state::State;
use carbide_core::widget::{
    CommonWidget, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack,
};

use crate::PlainButton;
use crate::types::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainCheckBox<F, C> where
    F: State<T=Focus> + Clone,
    C: State<T=CheckBoxValue> + Clone
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget>,
    #[state] checked: C,
}

impl PlainCheckBox<Focus, CheckBoxValue> {
    pub fn new<C: IntoState<CheckBoxValue>>(checked: C) -> PlainCheckBox<TState<Focus>, C::Output> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainCheckBox::default_delegate,
        )
    }

    fn default_delegate(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget> {
        let background_color = Map1::read_map(checked.clone(), |value| {
            match value {
                CheckBoxValue::True => EnvironmentColor::Green,
                CheckBoxValue::Intermediate => EnvironmentColor::Blue,
                CheckBoxValue::False => EnvironmentColor::Red,
            }
        });

        let val = Map2::read_map(checked, focus, |checked, focus| {
            format!("{:?}, {:?}", *checked, focus)
        });

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone> PlainCheckBox<F, C> {

    pub fn delegate(
        self,
        delegate: fn(focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget>,
    ) -> PlainCheckBox<F, C> {
        Self::new_internal(self.checked, self.focus, delegate)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainCheckBox<F2::Output, C> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate)
    }

    fn new_internal<F2: State<T=Focus> + Clone, C2: State<T=CheckBoxValue> + Clone>(
        checked: C2,
        focus: F2,
        delegate: fn(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget>,
    ) -> PlainCheckBox<F2, C2> {
        let delegate_widget = delegate(focus.as_dyn_read(), checked.as_dyn_read());

        let button = PlainButton::new(delegate_widget)
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                if *checked.value() == CheckBoxValue::True {
                    checked.set_value(CheckBoxValue::False);
                } else {
                    checked.set_value(CheckBoxValue::True);
                }

                if *focus.value() != Focus::Focused {
                    focus.set_value(Focus::FocusRequested);
                    println!("Focus request");
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone());

        let button = Box::new(button);

        let child = button;

        PlainCheckBox {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            delegate,
            checked,
        }
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone> Focusable for PlainCheckBox<F, C> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone> CommonWidget for PlainCheckBox<F, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10, focus: self.focus);
}

impl<F: State<T=Focus> + Clone, C: State<T=CheckBoxValue> + Clone> WidgetExt for PlainCheckBox<F, C> {}
