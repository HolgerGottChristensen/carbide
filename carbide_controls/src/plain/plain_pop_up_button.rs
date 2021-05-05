use std::fmt::Debug;

use carbide_core::DeserializeOwned;
use carbide_core::event_handler::{KeyboardEvent, MouseEvent};
use carbide_core::input::Key;
use carbide_core::prelude::Uuid;
use carbide_core::Serialize;
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::state::state::State;
use carbide_core::state::TupleState2;
use carbide_core::state::vec_state::VecState;
use carbide_core::widget::*;

use crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp;
use crate::PlainButton;

#[derive(Clone, Widget)]
#[focusable(block_focus)]
#[event(handle_keyboard_event, handle_mouse_event)]
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

        let opened = CommonState::new_local_with_key(&false);

        let start_item = model.get_latest_value().first().unwrap();

        let selected_item = VecState::new(model.clone(), selected_state.clone(), start_item.clone());

        let text = selected_item.clone().mapped(|a| format!("{:?}", a));

        let child = PlainButton::<(bool, T), GS>::new(
            Rectangle::initialize(vec![
                Text::new(text)
            ])).local_state(TupleState2::new(opened.clone(), selected_item.clone()))
            .on_click(|myself, _, _| {
                let (opened, selected_item) = myself.get_local_state().get_latest_value_mut();
                *opened = true;
                println!("Opened popup. The currently selected item is: {:?}", selected_item);
            });

        Box::new(PlainPopUpButton {
            id: Id::new_v4(),
            focus: Box::new(CommonState::new_local_with_key(&Focus::Focused)),
            child,
            popup_display_item: None,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            popup_id: Uuid::new_v4(),
            popup_list_spacing: 0.0,
            opened: Box::new(opened),
            selected_state,
            selected_item,
            model,
        })
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment<GS>, _: &mut GS) {
        if !self.is_inside(event.get_current_mouse_position()) {
            match event {
                MouseEvent::Press(_, _, _) => {
                    self.release_focus(env);
                }
                _ => ()
            }
        } else {
            match event {
                MouseEvent::Press(_, _, _) => {
                    self.request_focus(env);
                }
                _ => ()
            }

        }
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        if self.get_focus() != Focus::Focused { return }

        match event {
            KeyboardEvent::Press(key, modifier) => {
                match (key, modifier) {
                    (Key::Space, _) |
                    (Key::Return, _) => {
                        *self.opened.get_value_mut(env, global_state) = true;
                    }
                    (_, _) => {}
                }
            }
            _ => {}
        }
    }

    pub fn display_item(mut self, item: fn(selected_item: Box<dyn State<T, GS>>, focus: Box<dyn State<Focus, GS>>) -> Box<dyn Widget<GS>>) -> Box<Self> {

        let display_item = item(self.selected_item.clone(), self.focus.clone());

        let child = PlainButton::<(bool, T), GS>::new(display_item)
            .local_state(TupleState2::new(self.opened.clone(), self.selected_item.clone()))
            .on_click(|myself, _, _| {
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

    fn update_all_widget_state(&mut self, env: &mut Environment<GS>, _: &GS) {
        if *self.opened.get_latest_value() {
            let display_item = if let Some(display_item_function) = self.popup_display_item {
                display_item_function
            } else {
                |item: Box<dyn State<T, GS>>, _parent_selected_index: Box<dyn State<usize, GS>>, _item_index: Box<dyn State<usize, GS>>, partially_chosen: Box<dyn State<bool, GS>>| -> Box<dyn Widget<GS>>{
                    let text = item.mapped(|item| format!("{:?}", item));
                    Rectangle::initialize(vec![
                        Text::new(text)
                            .color(partially_chosen.clone().mapped(|partially_chosen| {
                                if *partially_chosen {
                                    Color::Rgba(0.0, 0.0, 0.0, 1.0)
                                } else {
                                    Color::Rgba(1.0, 1.0, 1.0, 1.0)
                                }
                            }))
                    ]).fill(partially_chosen.clone().mapped(|partially_chosen| {
                        if *partially_chosen {
                            Color::Rgba(0.0, 1.0, 0.0, 1.0)
                        } else {
                            Color::Rgba(0.0, 0.0, 1.0, 1.0)
                        }
                    })).border()
                        .border_width(1)
                        .color(EnvironmentColor::Blue.into())
                }
            };

            let mut overlay = PlainPopUpButtonPopUp::new(
                display_item,
                self.opened.clone(),
                self.model.clone(),
                self.selected_state.clone(),
                self.popup_list_spacing,
                self.dimension,
                env.get_corrected_dimensions());

            overlay.calculate_size([5000.0, 5000.0], env);

            let popup_x = self.get_x() - 1.0;

            let mut popup_y = self.get_y() - overlay.get_height() / 2.0 + self.get_height() / 2.0;

            if popup_y < 0.0 {
                popup_y = 0.0;
            } else if popup_y + overlay.get_height() > env.get_corrected_height() {
                popup_y = env.get_corrected_height() - overlay.get_height();
            }

            overlay.set_position([popup_x, popup_y]);
            overlay.set_id(self.popup_id);

            env.add_overlay("controls_popup_layer", overlay);
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
