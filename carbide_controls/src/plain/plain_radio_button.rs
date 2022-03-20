use std::fmt::Debug;
use carbide_core::{Color, Scalar};

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable, Refocus};
use carbide_core::layout::Layouter;
use carbide_core::state::{BoolState, FocusState, LocalState, Map2, Map3, MapOwnedState, ReadState, State, StateContract, StateExt, StateKey, StringState, TState};
use carbide_core::widget::{CommonWidget, HStack, Id, Rectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};

use crate::PlainButton;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Focusable)]
pub struct PlainRadioButton<T> where T: StateContract + PartialEq {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    delegate: fn(
        focus: FocusState,
        selected: BoolState,
    ) -> Box<dyn Widget>,
    reference: T,
    #[state]
    label: StringState,
    #[state]
    local_state: TState<T>,
    #[state]
    selected_state: BoolState,
}

impl<T: StateContract + PartialEq> PlainRadioButton<T> {
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

        Self::new_internal(
            reference,
            selected_state.into(),
            focus_state.into(),
            Self::default_delegate,
            label.into(),
        )
    }

    fn default_delegate(focus: FocusState, selected: BoolState) -> Box<dyn Widget> {
        let background_color: TState<Color> = selected
            .choice(EnvironmentColor::Green.state(), EnvironmentColor::Red.state())
            .ignore_writes();

        let val = Map2::read_map(selected, focus, |checked: &bool, focus: &Focus| {
            format!("{:?}, {:?}", *checked, focus)
        }).ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(val),
        ])
    }

    pub fn delegate(
        self,
        delegate: fn(
            focus: FocusState,
            selected: BoolState,
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
        ) -> Box<dyn Widget>,
        label_state: StringState,
    ) -> Box<Self> {
        let selected = local_state.eq(reference.clone()).ignore_writes();

        let delegate_widget = delegate(focus_state.clone(), selected.clone());

        let reference2 = reference.clone();
        let button = PlainButton::new(delegate_widget)
            .on_click(capture!([local_state, focus_state], |env: &mut Environment| {
                *local_state = reference2.clone();

                if *focus_state != Focus::Focused {
                    *focus_state = Focus::FocusRequested;
                    env.request_focus(Refocus::FocusRequest);
                }
            }))
            .focused(focus_state.clone());

        let child = HStack::new(vec![
            button,
            Text::new(label_state.clone()),
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
            local_state,
            selected_state: selected,
        })
    }
}

impl<T: StateContract + PartialEq> Focusable for PlainRadioButton<T> {
    fn focus_children(&self) -> bool {
        false
    }
}

impl<T: StateContract + PartialEq> CommonWidget for PlainRadioButton<T> {
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

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        *self.focus.value_mut() = focus;
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


impl<T: StateContract + PartialEq> WidgetExt for PlainRadioButton<T> {}
