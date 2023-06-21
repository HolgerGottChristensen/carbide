use carbide_core::{CommonWidgetImpl};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{AnyReadState, IntoReadState, IntoState, LocalState, Map1, Map2, Map4, ReadState, ReadStateExtNew, TState};
use carbide_core::state::State;
use carbide_core::widget::{
    CommonWidget, HStack, Rectangle, Text, Widget, WidgetExt, WidgetId, ZStack,
};

use crate::PlainButton;
use crate::types::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainCheckBox<F, L, C> where
    F: State<T=Focus> + Clone,
    L: ReadState<T=String> + Clone,
    C: State<T=CheckBoxValue> + Clone
{
    id: WidgetId,
    #[state] focus: F,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget>,
    #[state] label: L,
    #[state] checked: C,
}

impl PlainCheckBox<Focus, String, CheckBoxValue> {
    pub fn new<L: IntoReadState<String>, C: IntoState<CheckBoxValue>>(label: L, checked: C) -> PlainCheckBox<TState<Focus>, L::Output, C::Output> {
        let focus_state = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into_state(),
            focus_state,
            PlainCheckBox::default_delegate,
            label.into_read_state(),
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

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=CheckBoxValue> + Clone> PlainCheckBox<F, L, C> {

    pub fn delegate(
        self,
        delegate: fn(focus: Box<dyn AnyReadState<T=Focus>>, selected: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget>,
    ) -> PlainCheckBox<F, L, C> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    pub fn focused<F2: IntoState<Focus>>(mut self, focused: F2) -> PlainCheckBox<F2::Output, L, C> {
        Self::new_internal(self.checked, focused.into_state(), self.delegate, self.label)
    }

    fn new_internal<F2: State<T=Focus> + Clone, L2: ReadState<T=String> + Clone, C2: State<T=CheckBoxValue> + Clone>(
        checked: C2,
        focus: F2,
        delegate: fn(focus: Box<dyn AnyReadState<T=Focus>>, checked: Box<dyn AnyReadState<T=CheckBoxValue>>) -> Box<dyn Widget>,
        label_state: L2,
    ) -> PlainCheckBox<F2, L2, C2> {
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

        let child = HStack::new(vec![
            button,
            Text::new(label_state.clone())
        ]).spacing(5.0);

        PlainCheckBox {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            delegate,
            label: label_state,
            checked,
        }
    }
}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=CheckBoxValue> + Clone> Focusable for PlainCheckBox<F, L, C> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=CheckBoxValue> + Clone> CommonWidget for PlainCheckBox<F, L, C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: Flags::FOCUSABLE, flexibility: 10);
}

impl<F: State<T=Focus> + Clone, L: ReadState<T=String> + Clone, C: State<T=CheckBoxValue> + Clone> WidgetExt for PlainCheckBox<F, L, C> {}
