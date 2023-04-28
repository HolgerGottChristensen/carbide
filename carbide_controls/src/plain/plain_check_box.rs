use carbide_core::Color;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::state::{LocalState, Map2, Map4, ReadState, TState};
use carbide_core::state::State;
use carbide_core::widget::{
    CommonWidget, HStack, Rectangle, Text, Widget, WidgetExt, WidgetId, WidgetIter,
    WidgetIterMut, ZStack,
};

use crate::PlainButton;
use crate::types::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainCheckBox {
    id: WidgetId,
    #[state]
    focus: TState<Focus>,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(focus: TState<Focus>, checked: CheckBoxState) -> Box<dyn Widget>,
    #[state]
    label: TState<String>,
    #[state]
    checked: CheckBoxState,
}

impl PlainCheckBox {
    pub fn focused(mut self, focused: impl Into<TState<Focus>>) -> Box<Self> {
        self.focus = focused.into();
        Self::new_internal(self.checked, self.focus, self.delegate, self.label)
    }

    pub fn new(label: impl Into<TState<String>>, checked: impl Into<CheckBoxState>) -> Box<Self> {
        let focus = LocalState::new(Focus::Unfocused);

        Self::new_internal(
            checked.into(),
            focus.into(),
            Self::default_delegate,
            label.into(),
        )
    }

    fn default_delegate(focus: TState<Focus>, checked: CheckBoxState) -> Box<dyn Widget> {
        let green = EnvironmentColor::Green.state();
        let blue = EnvironmentColor::Blue.state();
        let red = EnvironmentColor::Red.state();

        let background_color = Map4::read_map(
            checked.clone(),
            green,
            blue,
            red,
            |checked: &CheckBoxValue, green: &Color, blue: &Color, red: &Color| match *checked {
                CheckBoxValue::True => *green,
                CheckBoxValue::Intermediate => *blue,
                CheckBoxValue::False => *red,
            },
        )
        .ignore_writes();

        let val = Map2::read_map(checked, focus, |checked: &CheckBoxValue, focus: &Focus| {
            format!("{:?}, {:?}", *checked, focus)
        })
        .ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }

    pub fn delegate(
        self,
        delegate: fn(focus: TState<Focus>, selected: CheckBoxState) -> Box<dyn Widget>,
    ) -> Box<Self> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    fn new_internal(
        checked: CheckBoxState,
        focus: TState<Focus>,
        delegate: fn(focus: TState<Focus>, selected: CheckBoxState) -> Box<dyn Widget>,
        label_state: TState<String>,
    ) -> Box<Self> {
        let delegate_widget = delegate(focus.clone(), checked.clone());

        let button = PlainButton::new(delegate_widget)
            .on_click(capture!([checked, focus], |env: &mut Environment| {
                if *checked == CheckBoxValue::True {
                    *checked = CheckBoxValue::False;
                } else {
                    *checked = CheckBoxValue::True;
                }

                if *focus != Focus::Focused {
                    *focus = Focus::FocusRequested;
                    println!("Focus request");
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone());

        let child = HStack::new(vec![button, Text::new(label_state.clone())]).spacing(5.0);

        Box::new(PlainCheckBox {
            id: WidgetId::new(),
            focus,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            delegate,
            label: label_state,
            checked,
        })
    }
}

impl Focusable for PlainCheckBox {
    fn focus_children(&self) -> bool {
        false
    }
}

impl CommonWidget for PlainCheckBox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn flexibility(&self) -> u32 {
        10
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        println!("Set focus: {:?}", focus);
        *self.focus.value_mut() = focus;
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for PlainCheckBox {}
