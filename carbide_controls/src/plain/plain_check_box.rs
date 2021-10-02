use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::prelude::Environment;
use carbide_core::state::{FocusState, LocalState, MapOwnedState, StateKey, StringState};
use carbide_core::widget::{CommonWidget, HStack, Id, Rectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut};

use crate::carbide_core::prelude::State;
use crate::PlainButton;
use crate::types::*;

#[derive(Debug, Clone, Widget)]
//#[focusable(block_focus)]
pub struct PlainCheckBox {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(
        focus: FocusState,
        checked: CheckBoxState,
    ) -> Box<dyn Widget>,
    #[state]
    label: StringState,
    #[state]
    checked: CheckBoxState,
}

impl PlainCheckBox {
    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Self::new_internal(self.checked, self.focus, self.delegate, self.label)
    }

    pub fn new<S: Into<StringState>, L: Into<CheckBoxState>>(
        label: S,
        checked: L,
    ) -> Box<Self> {
        let focus = LocalState::new(Focus::Unfocused);

        fn delegate(_: FocusState, checked: CheckBoxState) -> Box<dyn Widget> {
            let highlight_color = MapOwnedState::new(checked.clone(), |check: &CheckBoxState, env: &Environment| {
                match *check.value() {
                    CheckBoxValue::True => {
                        env.get_color(&StateKey::Color(EnvironmentColor::Green)).unwrap()
                    }
                    CheckBoxValue::Intermediate => {
                        env.get_color(&StateKey::Color(EnvironmentColor::Blue)).unwrap()
                    }
                    CheckBoxValue::False => {
                        env.get_color(&StateKey::Color(EnvironmentColor::Red)).unwrap()
                    }
                }
            });
            let val = checked.mapped(|check: &CheckBoxValue| {
                format!("{:?}", *check)
            });
            Rectangle::new(vec![Text::new(val)]).fill(highlight_color)
        }

        Self::new_internal(
            checked.into(),
            focus.into(),
            delegate,
            label.into(),
        )
    }

    pub fn delegate(
        self,
        delegate: fn(
            focus: FocusState,
            selected: CheckBoxState,
        ) -> Box<dyn Widget>,
    ) -> Box<Self> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    fn new_internal(
        checked: CheckBoxState,
        focus: FocusState,
        delegate: fn(
            focus: FocusState,
            selected: CheckBoxState,
        ) -> Box<dyn Widget>,
        label_state: StringState,
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
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus.clone());

        let child = HStack::new(vec![
            button,
            Text::new(label_state.clone()),
            Spacer::new(),
        ])
            .spacing(5.0);

        Box::new(PlainCheckBox {
            id: Id::new_v4(),
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

impl CommonWidget for PlainCheckBox {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn flexibility(&self) -> u32 {
        10
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
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
