use std::fmt::Debug;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::layout::Layout;
use carbide_core::state::{BoolState, FocusState, StateContract, StringState, TState};
use carbide_core::widget::{HStack, Id, Spacer, Text, Widget};

use crate::PlainButton;

#[derive(Clone, Widget)]
//#[focusable(block_focus)]
pub struct PlainRadioButton<T> where T: 'static + StateContract + PartialEq {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions,
    delegate: fn(
        focus: FocusState,
        selected: BoolState,
        button: Box<dyn Widget>,
    ) -> Box<dyn Widget>,
    reference: T,
    label: StringState,
    #[state]
    local_state: TState<T>,
}

impl<T: 'static + StateContract + PartialEq> PlainRadioButton<T> {
    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Box::new(self)
    }

    pub fn new<S: Into<StringState>, L: Into<TState<T>>>(
        label: S,
        reference: T,
        local_state: L,
    ) -> Box<Self> {
        let focus_state = Box::new(CommonState::new_local_with_key(&Focus::Unfocused));

        let default_delegate = |_focus_state: FocusState<GS>,
                                selected_state: BoolState<GS>,
                                button: Box<dyn Widget<GS>>|
                                -> Box<dyn Widget<GS>> {
            let highlight_color = TupleState3::new(
                selected_state,
                EnvironmentColor::Red,
                EnvironmentColor::Green,
            )
                .mapped(|(selected, selected_color, deselected_color)| {
                    if *selected {
                        *selected_color
                    } else {
                        *deselected_color
                    }
                });

            Rectangle::new(vec![button]).fill(highlight_color)
        };

        Self::new_internal(
            reference,
            local_state.into(),
            focus_state.into(),
            default_delegate,
            label.into(),
        )
    }

    pub(crate) fn delegate(
        self,
        delegate: fn(
            focus: FocusState<GS>,
            selected: BoolState<GS>,
            button: Box<dyn Widget<GS>>,
        ) -> Box<dyn Widget<GS>>,
    ) -> Box<Self> {
        let reference = self.reference;
        let local_state = self.local_state;
        let focus_state = self.focus;
        let label_state = self.label;

        Self::new_internal(reference, local_state, focus_state, delegate, label_state)
    }

    fn new_internal(
        reference: T,
        local_state: TState<T, GS>,
        focus_state: FocusState<GS>,
        delegate: fn(
            focus: FocusState<GS>,
            selected: BoolState<GS>,
            button: Box<dyn Widget<GS>>,
        ) -> Box<dyn Widget<GS>>,
        label_state: StringState<GS>,
    ) -> Box<Self> {
        let reference_state: TState<T, GS> = CommonState::new(&reference).into();

        let selected_state = TupleState2::new(reference_state.clone(), local_state.clone())
            .mapped(|(reference, local_state)| reference == local_state);

        let button = PlainButton::<(T, T), GS>::new(Spacer::new())
            .local_state(TupleState2::new(reference_state, local_state.clone()))
            .on_click(|myself, env, _| {
                let (reference, local_state) = myself.get_local_state().get_latest_value_mut();
                *local_state = reference.clone();
                myself.set_focus_and_request(Focus::FocusRequested, env);
            })
            .focused(focus_state.clone());

        let delegate_widget = delegate(focus_state.clone(), selected_state.into(), button);

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
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            delegate,
            reference,
            label: label_state,
            local_state: local_state.into(),
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

impl<T: 'static + StateContract + PartialEq> Layout for PlainRadioButton<T> {
    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        if let Some(child) = self.children_mut().next() {
            child.calculate_size(requested_size, env);
        }

        self.set_dimension(requested_size);

        requested_size
    }
}

impl<T: 'static + StateContract + PartialEq> WidgetExt for PlainRadioButton<T> {}
