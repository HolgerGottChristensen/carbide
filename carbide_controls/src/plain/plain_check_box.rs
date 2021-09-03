use std::ops::{Deref, DerefMut};

use carbide_core::{Color, widget};
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::prelude::Environment;
use carbide_core::state::{FocusState, LocalState, MapOwnedState, StateKey, StringState};
use carbide_core::widget::{CommonWidget, HStack, Id, Spacer, SpacerDirection, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut};

use crate::carbide_core::prelude::State;
use crate::PlainButton;
use crate::types::*;

#[derive(Clone, Widget)]
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
        button: Box<dyn Widget>,
    ) -> Box<dyn Widget>,
    label: StringState,
    #[state]
    checked: CheckBoxState,
}

impl PlainCheckBox {
    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Box::new(self)
    }

    pub fn new<S: Into<StringState>, L: Into<CheckBoxState>>(
        label: S,
        checked: L,
    ) -> Box<Self> {
        let focus_state = LocalState::new(Focus::Unfocused);

        let default_delegate = |_focus_state: FocusState,
                                checked: CheckBoxState,
                                button: Box<dyn Widget>|
                                -> Box<dyn Widget> {
            let highlight_color = MapOwnedState::new(checked, |check: &CheckBoxState, env: &Environment| {
                match *check.value() {
                    CheckBoxValue::True => {
                        env.get_color(&StateKey::Color(EnvironmentColor::Red)).unwrap()
                    }
                    CheckBoxValue::Intermediate => {
                        env.get_color(&StateKey::Color(EnvironmentColor::Green)).unwrap()
                    }
                    CheckBoxValue::False => {
                        env.get_color(&StateKey::Color(EnvironmentColor::Blue)).unwrap()
                    }
                }
            });

            widget::Rectangle::new(vec![button]).fill(highlight_color)
        };

        Self::new_internal(
            checked.into(),
            focus_state.into(),
            default_delegate,
            label.into(),
        )
    }

    pub fn delegate(
        self,
        delegate: fn(
            focus: FocusState,
            selected: CheckBoxState,
            button: Box<dyn Widget>,
        ) -> Box<dyn Widget>,
    ) -> Box<Self> {
        let checked = self.checked;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(checked, focus_state, delegate, label_state)
    }

    fn new_internal(
        checked: CheckBoxState,
        focus_state: FocusState,
        delegate: fn(
            focus: FocusState,
            selected: CheckBoxState,
            button: Box<dyn Widget>,
        ) -> Box<dyn Widget>,
        label_state: StringState,
    ) -> Box<Self> {
        let checked_for_button = checked.clone();
        let focus_for_button = focus_state.clone();
        let button = PlainButton::new(Spacer::new(SpacerDirection::Vertical))
            .on_click(move |env: &mut Environment| {
                let mut checked = checked_for_button.clone();

                if *checked.value() == CheckBoxValue::True {
                    *checked.value_mut() = CheckBoxValue::False;
                } else {
                    *checked.value_mut() = CheckBoxValue::True;
                }

                let mut focus_for_button = focus_for_button.clone();

                if *focus_for_button.value() != Focus::Focused {
                    *focus_for_button.value_mut() = Focus::FocusRequested;
                    env.request_focus(Refocus::FocusRequest);
                }
            })
            .focused(focus_state.clone());

        let delegate_widget = delegate(focus_state.clone(), checked.clone(), button);

        let child = HStack::new(vec![
            delegate_widget,
            Text::new(label_state.clone()),
            Spacer::new(SpacerDirection::Horizontal),
        ])
            .spacing(5.0);

        Box::new(PlainCheckBox {
            id: Id::new_v4(),
            focus: focus_state,
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

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
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
