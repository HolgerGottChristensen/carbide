use std::fmt::Debug;
use std::marker::PhantomData;

use carbide_core::DeserializeOwned;
use carbide_core::event_handler::KeyboardEvent;
use carbide_core::input::Key;
use carbide_core::prelude::EnvironmentColor;
use carbide_core::prelude::Uuid;
use carbide_core::Serialize;
use carbide_core::state::state::State;
use carbide_core::state::TupleState3;
use carbide_core::state::vec_state::VecState;
use carbide_core::widget::*;

use crate::{List, PlainButton};

#[derive(Clone, Widget)]
#[event(handle_keyboard_event)]
pub struct PlainPopUpButtonPopUp<T, GS> where GS: GlobalStateContract, T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static {
    id: Id,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    // The option in the list that is currently hovered or chosen with the arrow keys
    #[state] foreach_hovered_state: Box<dyn State<Vec<bool>, GS>>,
    // State to close this popup
    #[state] opened: Box<dyn State<bool, GS>>,
    #[state] parent_selected_index: Box<dyn State<usize, GS>>,
    phantom: PhantomData<T>,
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> PlainPopUpButtonPopUp<T, GS> {
    pub fn new(item: fn(item: Box<dyn State<T, GS>>, parent_selected_index: Box<dyn State<usize, GS>>, item_index: Box<dyn State<usize, GS>>, partially_chosen: Box<dyn State<bool, GS>>) -> Box<dyn Widget<GS>>,
               opened: Box<dyn State<bool, GS>>,
               model: Box<dyn State<Vec<T>, GS>>,
               parent_selected_index: Box<dyn State<usize, GS>>,
               popup_list_spacing: f64,
               parent_size: Dimensions,
               window_size: Dimensions,
    ) -> Box<Self> {
        let index_state = CommonState::new_local_with_key(&(0 as usize)).into_box();
        let selected_item_state = VecState::new_local(Box::new(model.clone()), index_state.clone(), T::default());

        // The model for the foreach
        let number_of_items_in_model = model.get_latest_value().len();
        let foreach_state = CommonState::<Vec<usize>, GS>::new_local_with_key(&(0..number_of_items_in_model).collect::<Vec<_>>());

        // Hover state
        let mut foreach_hovered_state = CommonState::<Vec<bool>, GS>::new_local_with_key(&foreach_state.get_latest_value().iter().map(|_| false).collect::<Vec<_>>()).into_box();

        foreach_hovered_state.get_latest_value_mut()[*parent_selected_index.get_latest_value()] = true;

        let hovered_state = VecState::new_local(foreach_hovered_state.clone(), index_state.clone(), false);

        let display_item = item(selected_item_state.clone(), parent_selected_index.clone(), index_state.clone(), hovered_state.clone());


        // Calculate the height of the popup
        let height: F64State<GS> = parent_size[1].into();
        let popup_list_spacing_state: F64State<GS> = popup_list_spacing.into();
        let length: UsizeState<GS> = foreach_state.get_latest_value().len().into();

        let max_height_state: Box<dyn State<f64, GS>> = Box::new(CommonState::<f64, GS>::EnvironmentState {
            function: |e: &Environment<GS>| {
                e.get_corrected_height()
            },
            function_mut: None,
            latest_value: window_size[1],
        });

        let tup = TupleState4::new(height, length, max_height_state, popup_list_spacing_state);

        let popup_height_state = tup.mapped(|(parent_height, number_of_elements, window_height, popup_list_spacing)| {
            window_height.min(*number_of_elements as f64 * parent_height + (*number_of_elements - 1) as f64 * popup_list_spacing + 2.0).max(*parent_height)
        });


        let child = Rectangle::initialize(vec![
            List::new(Box::new(foreach_state),
                      PlainButton::<(usize, bool, usize), GS>::new(display_item)
                          .local_state(TupleState3::new(index_state.clone(), opened.clone(), parent_selected_index.clone()))
                          .on_click(|myself, _, _| {
                              let (index, opened, selected_index) = myself.get_local_state().get_latest_value_mut();

                              *selected_index = *index;
                              *opened = false;

                              //println!("Closed popup and selected: {}", index);
                          })
                          .on_click_outside(|myself, _, _| {
                              let (_, opened, _) = myself.get_local_state().get_latest_value_mut();

                              *opened = false
                          })
                          .hover(hovered_state.clone())
                          .frame(parent_size[0], parent_size[1]),
            ).index_state(index_state)
                .spacing(popup_list_spacing)
                .clip()
                .border()
                .border_width(1)
                .color(EnvironmentColor::OpaqueSeparator),
        ])
            .fill(EnvironmentColor::Red)
            .frame(parent_size[0] + 2.0, popup_height_state);


        Box::new(PlainPopUpButtonPopUp {
            id: Id::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            foreach_hovered_state,
            opened,
            phantom: Default::default(),
            parent_selected_index,
        })
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        match event {
            KeyboardEvent::Press(key, _) => {
                match key {
                    Key::Return => {
                        let focused = self.foreach_hovered_state.get_value(env, global_state);

                        for (index, item) in focused.iter().enumerate() {
                            if *item {
                                *self.parent_selected_index.get_value_mut(env, global_state) = index;
                                *self.opened.get_value_mut(env, global_state) = false;
                            }
                        }
                    }
                    Key::Up => {
                        let focused = self.foreach_hovered_state.get_value_mut(env, global_state);
                        // This is an linear operation and might make it slow when the model is large

                        let mut true_at_any_point = false;
                        let mut last_item = false;
                        for item in focused.iter_mut().rev() {
                            let temp = *item;
                            *item = last_item;
                            last_item = temp;
                            if last_item {
                                true_at_any_point = true;
                            }
                        }

                        let last_index = focused.len() - 1;
                        if !true_at_any_point {
                            focused[last_index] = true;
                        } else {
                            focused[last_index] = last_item;
                        }
                    }
                    Key::Down => {
                        let focused = self.foreach_hovered_state.get_value_mut(env, global_state);
                        // This is an linear operation and might make it slow when the model is large

                        let mut true_at_any_point = false;
                        let mut last_item = false;
                        for item in focused.iter_mut() {
                            let temp = *item;
                            *item = last_item;
                            last_item = temp;
                            if last_item {
                                true_at_any_point = true;
                            }
                        }

                        if !true_at_any_point {
                            focused[0] = true;
                        } else {
                            focused[0] = last_item;
                        }
                    }
                    _ => {}
                }
            }
            _ => ()
        }
    }
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> CommonWidget<GS> for PlainPopUpButtonPopUp<T, GS> {
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

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> ChildRender for PlainPopUpButtonPopUp<T, GS> {}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalStateContract> Layout<GS> for PlainPopUpButtonPopUp<T, GS> {
    fn flexibility(&self) -> u32 {
        10
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        let size = self.child.calculate_size(requested_size, env);

        self.set_dimension(size);

        size
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


impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static + 'static, GS: GlobalStateContract> WidgetExt<GS> for PlainPopUpButtonPopUp<T, GS> {}
