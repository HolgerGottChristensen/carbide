use carbide_core::widget::*;
use carbide_core::event_handler::{MouseEvent, KeyboardEvent};
use carbide_core::input::MouseButton;
use carbide_core::input::Key;
use carbide_core::state::state::State;
use crate::PlainButton;
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::state::TupleState2;

#[derive(Clone, Widget)]
#[event(handle_keyboard_event, handle_mouse_event)]
#[focusable(block_focus)]
#[state_sync(update_all_widget_state)]
pub struct PlainPopUp<GS> where GS: GlobalState {
    id: Id,
    #[state] focus: Box<dyn State<Focus, GS>>,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    //popup: Box<dyn Widget<GS>>,
    #[state] opened: Box<dyn State<bool, GS>>,
    #[state] selected_state: Box<dyn State<u32, GS>>,
}

impl<GS: GlobalState> PlainPopUp<GS> {


    pub fn new(selected_state: Box<dyn State<u32, GS>>) -> Box<Self> {

        let opened = Box::new(CommonState::new_local_with_key(&false));

        Box::new(PlainPopUp {
            id: Id::new_v4(),
            focus: Box::new(CommonState::new_local_with_key(&Focus::Unfocused)),
            child: PlainButton::<bool, GS>::new(Rectangle::initialize(vec![]))
                .local_state(opened.clone())
                .on_click(|a, b, c| {
                    let local = a.get_local_state().get_latest_value_mut();
                    *local = !*local;
                    println!("{}", a.get_local_state().get_latest_value());
                }),
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            opened,
            selected_state
        })
    }

    fn update_all_widget_state(&mut self, env: &mut Environment<GS>, _: &GS) {
        if *self.opened.get_latest_value() {
            env.add_overlay("overlay_test", Rectangle::initialize(vec![
                PlainButton::<(bool, u32), GS>::new(Rectangle::initialize(vec![])
                    .fill(EnvironmentColor::Green.into())
                    .frame(10.0.into(),10.0.into())
                )
                    .local_state(TupleState2::new(self.opened.clone(), self.selected_state.clone()))
                    .on_click(|myself, env, global_state| {
                        let (local, selected_index) = myself.get_local_state().get_latest_value_mut();
                        *local = false;
                        *selected_index = *selected_index + 1;
                        println!("Closed popup and selected: {}", selected_index);
                    })
            ])
                .fill(EnvironmentColor::Red.into())
                .frame(100.0.into(), 100.0.into()))
        }
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment<GS>, global_state: &mut GS) {

    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {

    }
}

impl<GS: GlobalState> CommonWidget<GS> for PlainPopUp<GS> {
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

impl<GS: GlobalState> ChildRender for PlainPopUp<GS> {}

impl<GS: GlobalState> Layout<GS> for PlainPopUp<GS> {
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


impl<GS: GlobalState> WidgetExt<GS> for PlainPopUp<GS> {}
