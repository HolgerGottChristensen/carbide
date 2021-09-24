use std::fmt::Debug;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Refocus};
use carbide_core::state::{BoolState, FocusState, LocalState, MapOwnedState, State, StateContract, StateKey, StringState, TState};
use carbide_core::widget::{CommonWidget, HStack, Id, Rectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut};

use crate::PlainButton;

#[derive(Debug, Clone, Widget)]
//#[focusable(block_focus)]
pub struct PlainRadioButton<T> where T: 'static + StateContract + PartialEq {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(
        focus: FocusState,
        selected: BoolState,
        button: Box<dyn Widget>,
    ) -> Box<dyn Widget>,
    reference: T,
    #[state]
    label: StringState,
    #[state]
    local_state: TState<T>,
    #[state]
    selected_state: BoolState,
}

impl<T: 'static + StateContract + PartialEq> PlainRadioButton<T> {
    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Self::new_internal(self.reference, self.local_state, self.focus, self.delegate, self.label)
    }

    pub fn new<S: Into<StringState>, L: Into<TState<T>>>(
        label: S,
        reference: T,
        selected_state: L,
    ) -> Box<Self> {
        let focus_state = LocalState::new(Focus::Unfocused);

        fn delegate(_: FocusState, selected: BoolState, button: Box<dyn Widget>) -> Box<dyn Widget> {
            let highlight_color = MapOwnedState::new(selected.clone(), |selected: &BoolState, env: &Environment| {
                if *selected.value() {
                    env.get_color(&StateKey::Color(EnvironmentColor::Green)).unwrap()
                } else {
                    env.get_color(&StateKey::Color(EnvironmentColor::Red)).unwrap()
                }
            });
            let val = selected.mapped(|selected: &bool| {
                format!("{:?}", *selected)
            });
            Rectangle::new(vec![Text::new(val), button]).fill(highlight_color)
        }

        Self::new_internal(
            reference,
            selected_state.into(),
            focus_state.into(),
            delegate,
            label.into(),
        )
    }

    pub fn delegate(
        self,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
            button: Box<dyn Widget>,
        ) -> Box<dyn Widget>,
    ) -> Box<Self> {
        let reference = self.reference;
        let local_state = self.local_state;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(reference, local_state, focus_state, delegate, label_state)
    }

    fn new_internal(
        reference: T,
        local_state: TState<T>,
        focus_state: FocusState,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
            button: Box<dyn Widget>,
        ) -> Box<dyn Widget>,
        label_state: StringState,
    ) -> Box<Self> {
        let reference1 = reference.clone();

        let selected_state: BoolState = MapOwnedState::new(local_state.clone(), move |selected: &TState<T>, env: &Environment| {
            reference1 == *selected.value()
        }).into();

        let reference2 = reference.clone();
        let button = PlainButton::new(Spacer::new())
            .on_click(capture!([local_state, focus_state], |env: &mut Environment| {
                *local_state = reference2.clone();

                if *focus_state != Focus::Focused {
                    *focus_state = Focus::FocusRequested;
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus_state.clone());

        let delegate_widget = delegate(focus_state.clone(), selected_state.clone(), button);

        let child = HStack::new(vec![
            delegate_widget,
            Text::new(label_state.clone()),
            Spacer::new(),
        ])
            .spacing(5.0);

        Box::new(PlainRadioButton {
            id: Id::new_v4(),
            focus: focus_state,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            delegate,
            reference,
            label: label_state,
            local_state: local_state,
            selected_state,
        })
    }
}

impl<T: 'static + StateContract + PartialEq> CommonWidget for PlainRadioButton<T> {
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


impl<T: 'static + StateContract + PartialEq> WidgetExt for PlainRadioButton<T> {}
