use carbide_core::widget::*;
use carbide_core::event_handler::{MouseEvent, KeyboardEvent};
use carbide_core::input::MouseButton;
use carbide_core::input::Key;
use carbide_core::state::state::State;
use crate::{PlainButton, List};
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::state::{TupleState2, TupleState3};
use carbide_core::widget::primitive::foreach::ForEach;
use carbide_core::state::mapped_state::MappedState;
use carbide_core::prelude::Uuid;
use carbide_core::state::vec_state::VecState;

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
    popup_id: Id,
    popup_list_spacing: f64,
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
            popup_id: Uuid::new_v4(),
            popup_list_spacing: 0.0,
            opened,
            selected_state
        })
    }

    fn update_all_widget_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        if *self.opened.get_latest_value() {


            let index_state = CommonState::new_local_with_key(&0);
            let foreach_state = CommonState::<Vec<u32>, GS>::new_local_with_key(&(0..4).collect::<Vec<u32>>());
            let foreach_selected_state = CommonState::<Vec<bool>, GS>::new_local_with_key(&foreach_state.get_latest_value().iter().map(|_| false).collect::<Vec<_>>());

            let selected_state = VecState::new_local(Box::new(foreach_selected_state.clone()), Box::new(index_state.clone()), false);

            let mapped_state = MappedState::new_local(Box::new(index_state.clone()), |a: &usize| format!("{}", a), "0".to_string());

            let height: Box<dyn State<f64, GS>> = self.dimension[1].into();
            let popup_list_spacing: Box<dyn State<f64, GS>> = self.popup_list_spacing.into();
            let length: Box<dyn State<usize, GS>> = foreach_state.get_latest_value().len().into();

            let max_height_state: Box<dyn State<f64, GS>> = Box::new(CommonState::<f64, GS>::EnvironmentState {
                function: |e: &Environment<GS>| {
                    (e.window_dimension[1])
                },
                function_mut: None,
                latest_value: env.window_dimension[1]
            });

            let tup = TupleState4::new(height, length, max_height_state, popup_list_spacing);

            let mut popup_height_state = tup.mapped(|(parent_height, number_of_elements, window_height, popup_list_spacing)| {
                window_height.min(*number_of_elements as f64 * parent_height + (*number_of_elements - 1) as f64 * popup_list_spacing + 2.0).max(*parent_height)
            });

            let popup_x = self.get_x() - 1.0;

            let popup_height = *popup_height_state.get_value(env, global_state);

            let mut popup_y = self.get_y() - popup_height / 2.0 + self.get_height() / 2.0;

            if popup_y < 0.0 {
                popup_y = 0.0;
            } else if popup_y + popup_height > env.window_dimension[1]{
                popup_y = env.window_dimension[1];
            }

            let mut overlay = Rectangle::initialize(vec![
                SharedState::new(
                    TupleState3::new(self.opened.clone(), self.selected_state.clone(), Box::new(foreach_selected_state.clone())),
                    List::new(Box::new(foreach_state),
                               Rectangle::initialize(vec![
                                   PlainButton::<(usize, bool, usize), GS>::new(
                                       Text::initialize(mapped_state)
                                           .color(selected_state.clone().mapped(|selected| {
                                               if *selected {
                                                   Color::Rgba(0.0, 0.0, 0.0, 1.0)
                                               } else {
                                                   Color::Rgba(1.0, 1.0, 1.0, 1.0)
                                               }
                                           }))
                                   )
                                       .local_state(TupleState3::new(Box::new(index_state.clone()), self.opened.clone(), self.selected_state.clone()))
                                       .on_click(|myself, env, global_state| {
                                           let (index, opened, selected_index) = myself.get_local_state().get_latest_value_mut();

                                           *selected_index = *index;
                                           *opened = false;

                                           println!("Closed popup and selected: {}", index);
                                       })
                                       .on_click_outside(|myself, _, _| {
                                           let (_, opened, _) = myself.get_local_state().get_latest_value_mut();

                                           *opened = false
                                       })
                                       .hover(selected_state.clone())
                               ]).fill(selected_state.mapped(|selected| {
                                   if *selected {
                                       Color::Rgba(0.0, 1.0, 0.0, 1.0)
                                   } else {
                                       Color::Rgba(0.0, 0.0, 1.0, 1.0)
                                   }
                               }))
                                   .padding(EdgeInsets::all(1.0))
                                   .border()
                                   .border_width(1)
                                   .color(EnvironmentColor::Blue.into())
                                   .frame(self.dimension[0].into(), self.dimension[1].into())
                            ).index_state(Box::new(index_state))
                        .spacing(self.popup_list_spacing)
                )
                    .clip()
                    .padding(EdgeInsets::all(1.0))
                    .border()
                    .border_width(1)
                    .color(EnvironmentColor::Yellow.into()),

            ])
                .fill(EnvironmentColor::Red.into())
                .frame((self.dimension[0] + 2.0).into(), popup_height_state);
                //.with_fixed_position(popup_x.into(), 100.0.into());

            overlay.calculate_size([300.0, 600.0], env);
            overlay.set_position([popup_x, popup_y]);
            overlay.set_id(self.popup_id);

            env.add_overlay("overlay_test", overlay);
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

impl<GS: GlobalState> ChildRender for PlainPopUpButton<GS> {}

impl<GS: GlobalState> Layout<GS> for PlainPopUpButton<GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<GS>) -> Dimensions {
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
