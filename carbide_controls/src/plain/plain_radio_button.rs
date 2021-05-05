use std::fmt::Debug;

use carbide_core::{DeserializeOwned, Serialize};
use carbide_core::event_handler::{KeyboardEvent, MouseEvent};
use carbide_core::input::Key;
use carbide_core::input::MouseButton;
use carbide_core::prelude::Uuid;
use carbide_core::state::state::State;
use carbide_core::widget::*;

use crate::PlainButton;

#[derive(Clone, Widget)]
#[focusable(block_focus)]
pub struct PlainRadioButton<T, GS> where GS: GlobalState, T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq {
    id: Id,
    #[state] focus: FocusState<GS>,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    delegate: fn(focus: FocusState<GS>, selected: BoolState<GS>, button: Box<dyn Widget<GS>>) -> Box<dyn Widget<GS>>,
    reference: T,
    label: StringState<GS>,
    #[state] local_state: TState<T, GS>,
}

impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq, GS: GlobalState> PlainRadioButton<T, GS> {

    pub fn focused(mut self, focused: Box<dyn State<Focus, GS>>) -> Box<Self> {
        self.focus = focused;
        Box::new(self)
    }

    pub fn new<S: Into<StringState<GS>>, L: Into<TState<T, GS>>>(label: S, reference: T, local_state: L) -> Box<Self> {

        let focus_state =  Box::new(CommonState::new_local_with_key(&Focus::Unfocused));

        let default_delegate= |focus_state: FocusState<GS>, selected_state: BoolState<GS>, button: Box<dyn Widget<GS>>| -> Box<dyn Widget<GS>> {
            let highlight_color = TupleState3::new(selected_state, EnvironmentColor::Red, EnvironmentColor::Green)
                .mapped(|(selected, selected_color, deselected_color)| {
                    if *selected {
                        *selected_color
                    } else {
                        *deselected_color
                    }
                });

            Rectangle::initialize(vec![
                button
            ]).fill(highlight_color)
        };

        Self::new_internal(reference, local_state.into(), focus_state, default_delegate, label.into())
    }

    pub(crate) fn delegate(self, delegate: fn(focus: FocusState<GS>, selected: BoolState<GS>, button: Box<dyn Widget<GS>>) -> Box<dyn Widget<GS>>) -> Box<Self> {
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
        delegate: fn(focus: FocusState<GS>, selected: BoolState<GS>, button: Box<dyn Widget<GS>>) -> Box<dyn Widget<GS>>,
        label_state: StringState<GS>
    ) -> Box<Self> {

        let reference_state: TState<T, GS> = CommonState::new(&reference).into();

        let selected_state = TupleState2::new(reference_state.clone(), local_state.clone())
            .mapped(|(reference, local_state)| {
                reference == local_state
            });

        let button = PlainButton::<(T, T), GS>::new(Spacer::new(SpacerDirection::Vertical))
            .local_state(TupleState2::new(reference_state, local_state.clone()))
            .on_click(|myself, env, global_state| {
                let (reference, local_state) = myself.get_local_state().get_latest_value_mut();
                *local_state = reference.clone();
                myself.set_focus_and_request(Focus::FocusRequested, env);
            }).focused(focus_state.clone());

        let delegate_widget = delegate(focus_state.clone(), selected_state, button);

        let child = HStack::initialize(vec![
            delegate_widget,
            Text::new(label_state.clone()),
            Spacer::new(SpacerDirection::Horizontal)
        ]).spacing(5.0);

        Box::new(PlainRadioButton {
            id: Id::new_v4(),
            focus: focus_state,
            child,
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            delegate,
            reference,
            label: label_state,
            local_state: local_state.into()
        })
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq, GS: GlobalState> CommonWidget<GS> for PlainRadioButton<T, GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn get_children(&self) -> WidgetIter<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq, GS: GlobalState> ChildRender for PlainRadioButton<T, GS> {}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq, GS: GlobalState> Layout<GS> for PlainRadioButton<T, GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment<GS>) -> [f64; 2] {
        if let Some(child) = self.get_children_mut().next() {
            child.calculate_size(requested_size, env);
        }

        self.set_dimension(requested_size);

        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();

        if let Some(child) = self.get_children_mut().next() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}


impl<T: 'static + Serialize + Clone + Debug + Default + DeserializeOwned + PartialEq, GS: GlobalState> WidgetExt<GS> for PlainRadioButton<T, GS> {}
