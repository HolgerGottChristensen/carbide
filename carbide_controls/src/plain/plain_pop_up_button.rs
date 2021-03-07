use carbide_core::widget::*;
use carbide_core::event_handler::{MouseEvent, KeyboardEvent};
use carbide_core::input::MouseButton;
use carbide_core::input::Key;
use carbide_core::state::state::State;
use crate::PlainButton;
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::state::{TupleState2, TupleState3};
use carbide_core::widget::primitive::foreach::ForEach;
use carbide_core::state::mapped_state::MappedState;

#[derive(Clone, Widget)]
#[event(handle_keyboard_event, handle_mouse_event)]
#[focusable(block_focus)]
#[state_sync(update_all_widget_state)]
pub struct PlainPopUpButton<GS> where GS: GlobalState {
    id: Id,
    #[state] focus: Box<dyn State<Focus, GS>>,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    //popup: Box<dyn Widget<GS>>,
    #[state] opened: Box<dyn State<bool, GS>>,
    #[state] selected_state: Box<dyn State<usize, GS>>,
}

impl<GS: GlobalState> PlainPopUpButton<GS> {


    pub fn new(selected_state: Box<dyn State<usize, GS>>) -> Box<Self> {

        let opened = Box::new(CommonState::new_local_with_key(&false));

        Box::new(PlainPopUpButton {
            id: Id::new_v4(),
            focus: Box::new(CommonState::new_local_with_key(&Focus::Unfocused)),
            child: PlainButton::<(bool, usize), GS>::new(Rectangle::initialize(vec![]))
                .local_state(TupleState2::new(opened.clone(), selected_state.clone()))
                .on_click(|myself, env, global_state| {
                    let (opened, selected_index) = myself.get_local_state().get_latest_value_mut();
                    *opened = true;
                    println!("Opened popup. The currently selected item is: {}", selected_index);
                }),
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            opened,
            selected_state
        })
    }

    fn update_all_widget_state(&mut self, env: &mut Environment<GS>, _: &GS) {
        if *self.opened.get_latest_value() {

            let index_state = CommonState::new_local_with_key(&0);
            let foreach_state = CommonState::<Vec<u32>, GS>::new_local_with_key(&(0..5).collect::<Vec<u32>>());

            let mapped_state = MappedState::new_local(Box::new(index_state.clone()), |a: &usize| format!("{}", a), "0".to_string());

            env.add_overlay("overlay_test", Rectangle::initialize(vec![
                SharedState::new(TupleState2::new(self.opened.clone(), self.selected_state.clone()),
                VStack::initialize(
                    vec![
                        ForEach::new(Box::new(foreach_state),
                            Rectangle::initialize(vec![
                                PlainButton::<(usize, bool, usize), GS>::new(Text::initialize(mapped_state))
                                    .local_state(TupleState3::new(Box::new(index_state.clone()), self.opened.clone(), self.selected_state.clone()))
                                    .on_click(|myself, env, global_state| {
                                        let (index, opened, selected_index) = myself.get_local_state().get_latest_value_mut();

                                        *selected_index = *index;
                                        *opened = false;

                                        println!("Closed popup and selected: {}", index);
                                    })
                            ])
                                .fill(EnvironmentColor::Green.into())
                                .frame(30.0.into(), 30.0.into())
                        ).index_state(Box::new(index_state))
                    ]
                )),

            ])
                .fill(EnvironmentColor::Red.into())
                .frame(100.0.into(), 300.0.into()))
        }
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment<GS>, global_state: &mut GS) {

    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {

    }
}

impl<GS: GlobalState> CommonWidget<GS> for PlainPopUpButton<GS> {
    fn get_id(&self) -> Id {
        self.id
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

impl<GS: GlobalState> ChildRender for PlainPopUpButton<GS> {}

impl<GS: GlobalState> Layout<GS> for PlainPopUpButton<GS> {
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


impl<GS: GlobalState> WidgetExt<GS> for PlainPopUpButton<GS> {}
