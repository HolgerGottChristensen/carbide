use carbide_core::widget::*;
use carbide_core::event_handler::{MouseEvent, KeyboardEvent};
use carbide_core::input::MouseButton;
use carbide_core::input::Key;
use carbide_core::state::state::State;
use crate::{PlainButton, List, PlainPopUpButton};
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::state::{TupleState2, TupleState3};
use carbide_core::widget::primitive::foreach::ForEach;
use carbide_core::state::mapped_state::MappedState;
use carbide_core::prelude::Uuid;
use carbide_core::state::vec_state::VecState;
use std::fmt::Debug;
use carbide_core::DeserializeOwned;
use carbide_core::Serialize;
use std::ops::{DerefMut, Deref};

#[derive(Clone, Widget)]
pub struct PopUpButton<T, GS> where GS: GlobalState, T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static {
    id: Id,
    child: PlainPopUpButton<T, GS>,
    position: Point,
    dimension: Dimensions,
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> PopUpButton<T, GS> {

    pub fn new(model: Box<dyn State<Vec<T>, GS>>, selected_state: Box<dyn State<usize, GS>>) -> Box<Self> {

        let mut child = *PlainPopUpButton::new(model, selected_state);

        /*child = child.display_item(|selected_item| {

            let text = selected_item.mapped(|item| format!("{:?}", item));

            Rectangle::initialize(vec![
                Text::initialize(text)
            ]).fill(EnvironmentColor::SecondarySystemBackground.into())
                .border().color(EnvironmentColor::OpaqueSeparator.into()).border_width(1)
        });*/

        Box::new(PopUpButton {
            id: Id::new_v4(),
            child,
            position: [0.0,0.0],
            dimension: [235.0, 26.0],
        })
    }

}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> CommonWidget<GS> for PopUpButton<T, GS> {
    fn get_id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<GS> {
        WidgetIter::single(&self.child)
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<GS> {
        self.child.get_children_mut()
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<GS> {
        self.child.get_proxied_children()
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<GS> {
        self.child.get_proxied_children_rev()
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

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> ChildRender for PopUpButton<T, GS> {}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> Layout<GS> for PopUpButton<T, GS> {
    fn flexibility(&self) -> u32 {
        5
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<GS>) -> Dimensions {
        self.set_width(requested_size[0]);

        self.child.calculate_size(self.dimension, env);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();


        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}


impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static + 'static, GS: GlobalState> WidgetExt<GS> for PopUpButton<T, GS> {}
