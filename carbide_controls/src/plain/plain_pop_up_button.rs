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
use std::fmt::Debug;
use carbide_core::DeserializeOwned;
use carbide_core::Serialize;

#[derive(Clone, Widget)]
#[focusable(block_focus)]
#[state_sync(update_all_widget_state)]
pub struct PlainPopUpButton<T, GS> where GS: GlobalState, T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static {
    id: Id,
    #[state] focus: Box<dyn State<Focus, GS>>,
    child: Box<dyn Widget<GS>>,
    popup_display_item: Option<fn (selected_item: Box<dyn State<T, GS>>, selected_index: Box<dyn State<usize, GS>>, index: Box<dyn State<usize, GS>>, hovered: Box<dyn State<bool, GS>>) -> Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
    popup_id: Id,
    popup_list_spacing: f64,
    #[state] opened: Box<dyn State<bool, GS>>,
    #[state] selected_state: Box<dyn State<usize, GS>>,
    #[state] selected_item: Box<dyn State<T, GS>>,
    #[state] model: Box<dyn State<Vec<T>, GS>>,
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> PlainPopUpButton<T, GS> {


    pub fn new(model: Box<dyn State<Vec<T>, GS>>, selected_state: Box<dyn State<usize, GS>>) -> Box<Self> {

        let opened = Box::new(CommonState::new_local_with_key(&false));

        let start_item = model.get_latest_value().first().unwrap();

        let selected_item = VecState::new(model.clone(), selected_state.clone(), start_item.clone());

        let text = selected_item.clone().mapped(|a| format!("{:?}", a));

        let child = PlainButton::<(bool, T), GS>::new(
            Rectangle::initialize(vec![
                Text::initialize(text)
            ])).local_state(TupleState2::new(opened.clone(), selected_item.clone()))
            .on_click(|myself, env, global_state| {
                let (opened, selected_item) = myself.get_local_state().get_latest_value_mut();
                *opened = true;
                println!("Opened popup. The currently selected item is: {:?}", selected_item);
            });

        Box::new(PlainPopUpButton {
            id: Id::new_v4(),
            focus: Box::new(CommonState::new_local_with_key(&Focus::Unfocused)),
            child,
            popup_display_item: None,
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            popup_id: Uuid::new_v4(),
            popup_list_spacing: 0.0,
            opened,
            selected_state,
            selected_item,
            model,
        })
    }

    pub fn display_item(mut self, item: fn(selected_item: Box<dyn State<T, GS>>) -> Box<dyn Widget<GS>>) -> Box<Self> {

        let display_item = item(self.selected_item.clone());

        let child = PlainButton::<(bool, T), GS>::new(display_item)
            .local_state(TupleState2::new(self.opened.clone(), self.selected_item.clone()))
            .on_click(|myself, env, global_state| {
                let (opened, _) = myself.get_local_state().get_latest_value_mut();
                *opened = true;
                //println!("Opened popup. The currently selected item is: {:?}", selected_item);
            });

        self.child = child;
        Box::new(self)
    }

    pub fn display_item_popup(mut self, item: fn (item: Box<dyn State<T, GS>>, selected_index: Box<dyn State<usize, GS>>, index: Box<dyn State<usize, GS>>, hovered: Box<dyn State<bool, GS>>) -> Box<dyn Widget<GS>>) -> Box<Self> {
        self.popup_display_item = Some(item);

        Box::new(self)
    }

    fn update_all_widget_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
        if *self.opened.get_latest_value() {

            let number_of_items_in_model = self.model.get_latest_value().len();

            let index_state = CommonState::new_local_with_key(&(0 as usize));
            let foreach_state = CommonState::<Vec<usize>, GS>::new_local_with_key(&(0..number_of_items_in_model).collect::<Vec<_>>());
            let foreach_selected_state = CommonState::<Vec<bool>, GS>::new_local_with_key(&foreach_state.get_latest_value().iter().map(|_| false).collect::<Vec<_>>());

            let selected_state = VecState::new_local(Box::new(foreach_selected_state.clone()), Box::new(index_state.clone()), false);

            let selected_item_state = VecState::new_local(Box::new(self.model.clone()), Box::new(index_state.clone()), T::default());
            let mapped_state = MappedState::new_local(selected_item_state.clone(), |a: &T| format!("{:?}", a), "".to_string());

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

            let display_item = if let Some(display_item_function) = self.popup_display_item {
                display_item_function(selected_item_state.clone(), self.selected_state.clone(), Box::new(index_state.clone()), selected_state.clone())
            } else {
                Rectangle::initialize(vec![
                    Text::initialize(mapped_state)
                        .color(selected_state.clone().mapped(|selected| {
                            if *selected {
                                Color::Rgba(0.0, 0.0, 0.0, 1.0)
                            } else {
                                Color::Rgba(1.0, 1.0, 1.0, 1.0)
                            }
                        }))
                ]).fill(selected_state.clone().mapped(|selected| {
                    if *selected {
                        Color::Rgba(0.0, 1.0, 0.0, 1.0)
                    } else {
                        Color::Rgba(0.0, 0.0, 1.0, 1.0)
                    }
                })).border()
                    .border_width(1)
                    .color(EnvironmentColor::Blue.into())
            };

            let mut overlay = Rectangle::initialize(vec![
                SharedState::new(
                    TupleState3::new(self.opened.clone(), self.selected_state.clone(), Box::new(foreach_selected_state.clone())),
                    List::new(Box::new(foreach_state),
                               PlainButton::<(usize, bool, usize), GS>::new(display_item)
                                       .local_state(TupleState3::new(Box::new(index_state.clone()), self.opened.clone(), self.selected_state.clone()))
                                       .on_click(|myself, env, global_state| {
                                           let (index, opened, selected_index) = myself.get_local_state().get_latest_value_mut();

                                           *selected_index = *index;
                                           *opened = false;

                                           //println!("Closed popup and selected: {}", index);
                                       })
                                       .on_click_outside(|myself, _, _| {
                                           let (_, opened, _) = myself.get_local_state().get_latest_value_mut();

                                           *opened = false
                                       })
                                       .hover(selected_state.clone())

                                   .frame(self.dimension[0].into(), self.dimension[1].into())
                            ).index_state(Box::new(index_state))
                        .spacing(self.popup_list_spacing)
                )
                    .clip()
                    .border()
                    .border_width(1)
                    .color(EnvironmentColor::OpaqueSeparator.into()),

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
}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> CommonWidget<GS> for PlainPopUpButton<T, GS> {
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

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> ChildRender for PlainPopUpButton<T, GS> {}

impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static, GS: GlobalState> Layout<GS> for PlainPopUpButton<T, GS> {
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


impl<T: Serialize + Clone + Debug + Default + DeserializeOwned + 'static + 'static, GS: GlobalState> WidgetExt<GS> for PlainPopUpButton<T, GS> {}
