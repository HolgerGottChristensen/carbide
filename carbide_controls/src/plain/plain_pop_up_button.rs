use std::fmt::Debug;
use std::ops::DerefMut;

use carbide_core::__private::Formatter;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{MouseEvent, MouseEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::layout::Layout;
use carbide_core::prelude::{EnvironmentColor, Primitive};
use carbide_core::render::Render;
use carbide_core::state::{BoolState, FocusState, LocalState, State, StateContract, StateKey, TState, UsizeState};
use carbide_core::widget::*;

use crate::{List, PlainButton};
use crate::plain::plain_pop_up_button_popup::PlainPopUpButtonPopUp;
use crate::plain::plain_pop_up_button_popup_item::PlainPopUpButtonPopUpItem;

#[derive(Clone)]
pub struct PopupDelegate<T> where T: StateContract + 'static {
    hover_model: TState<Vec<bool>>,
    selected_item: TState<T>,
    popup_item_delegate: PopupItemDelegateGenerator<T>,
}

impl<T: StateContract + 'static> Delegate<T> for PopupDelegate<T> {
    fn call(&self, item: TState<T>, index: UsizeState) -> Box<dyn Widget> {
        let hover_state = self.hover_model.index(index.clone());
        let selected_item_del = self.selected_item.clone();

        let popup_item_delegate =
            (self.popup_item_delegate)(item.clone(), index, hover_state.clone(), selected_item_del.clone());

        let hovered = PlainButton::new(
            popup_item_delegate
        ).hover(hover_state.clone());

        PlainPopUpButtonPopUpItem::new(
            hovered,
            hover_state,
            item,
            selected_item_del,
        )
    }
}

type DelegateGenerator<T: StateContract + PartialEq + 'static> = fn(selected_item: TState<T>, focused: FocusState) -> Box<dyn Widget>;
type PopupDelegateGenerator<T: StateContract + PartialEq + 'static> = fn(model: TState<Vec<T>>, delegate: PopupDelegate<T>) -> Box<dyn Widget>;
type PopupItemDelegateGenerator<T: StateContract + PartialEq + 'static> = fn(item: TState<T>, index: UsizeState, hover: BoolState, selected: TState<T>) -> Box<dyn Widget>;

#[derive(Clone, Widget)]
#[carbide_exclude(Render, Layout, MouseEvent)]
pub struct PlainPopUpButton<T> where T: StateContract + PartialEq + 'static {
    id: Id,
    #[state] focus: FocusState,
    child: Box<dyn Widget>,

    popup_item_delegate: PopupItemDelegateGenerator<T>,
    popup_delegate: PopupDelegateGenerator<T>,
    delegate: DelegateGenerator<T>,

    position: Position,
    dimension: Dimension,
    popup_list_spacing: f64,
    popup: Overlay,
    #[state]
    selected_item: TState<T>,
    #[state]
    model: TState<Vec<T>>,
}

impl<T: StateContract + PartialEq + 'static> PlainPopUpButton<T> {
    pub fn new<M: Into<TState<Vec<T>>>, S: Into<TState<T>>>(
        model: M,
        selected_state: S,
    ) -> Box<Self> {
        let focus: FocusState = LocalState::new(Focus::Unfocused).into();

        Self::new_internal(
            focus,
            model,
            selected_state,
            Self::default_popup_item_delegate,
            Self::default_popup_delegate,
            Self::default_delegate,
        )
    }

    pub fn delegate(mut self, delegate: DelegateGenerator<T>) -> Box<Self> {
        self.delegate = delegate;
        Self::new_internal(
            self.focus,
            self.model,
            self.selected_item,
            self.popup_item_delegate,
            self.popup_delegate,
            self.delegate,
        )
    }

    pub fn popup_item_delegate(mut self, popup_item_delegate: PopupItemDelegateGenerator<T>) -> Box<Self> {
        self.popup_item_delegate = popup_item_delegate;
        Self::new_internal(
            self.focus,
            self.model,
            self.selected_item,
            self.popup_item_delegate,
            self.popup_delegate,
            self.delegate,
        )
    }

    pub fn popup_delegate(mut self, popup_delegate: PopupDelegateGenerator<T>) -> Box<Self> {
        self.popup_delegate = popup_delegate;
        Self::new_internal(
            self.focus,
            self.model,
            self.selected_item,
            self.popup_item_delegate,
            self.popup_delegate,
            self.delegate,
        )
    }

    fn new_internal<M: Into<TState<Vec<T>>>, S: Into<TState<T>>>(
        focus: FocusState,
        model: M,
        selected_state: S,
        popup_item_delegate: PopupItemDelegateGenerator<T>,
        popup_delegate: PopupDelegateGenerator<T>,
        delegate: DelegateGenerator<T>,
    ) -> Box<PlainPopUpButton<T>> {
        let model = model.into();
        let selected_item = selected_state.into();
        let hover_model: TState<Vec<bool>> = LocalState::new(vec![false; model.value().len()]).into();

        let del = PopupDelegate {
            hover_model: hover_model.clone(),
            selected_item: selected_item.clone(),
            popup_item_delegate,
        };

        let popup_delegate_widget = popup_delegate(model.clone(), del);

        let popup = Overlay::new(
            PlainPopUpButtonPopUp::new(popup_delegate_widget, hover_model)
        );

        let child =
            delegate(selected_item.clone(), focus.clone());

        Box::new(PlainPopUpButton {
            id: Id::new_v4(),
            focus,
            child,
            popup_item_delegate,
            popup_delegate,
            delegate,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            popup_list_spacing: 0.0,
            popup,
            selected_item,
            model,
        })
    }

    fn default_delegate(selected_item: TState<T>, focused: FocusState) -> Box<dyn Widget> {
        let text = selected_item.mapped(|a: &T| format!("{:?}", a));
        Rectangle::new(vec![Text::new(text)])
    }

    fn default_popup_item_delegate(
        item: TState<T>, index: UsizeState, hover_state: BoolState, selected_state: TState<T>,
    ) -> Box<dyn Widget> {
        let color = hover_state.mapped_env(|hovered: &bool, _: &_, env: &Environment| {
            if *hovered {
                env.get_color(&StateKey::Color(EnvironmentColor::Pink)).unwrap()
            } else {
                env.get_color(&StateKey::Color(EnvironmentColor::Gray)).unwrap()
            }
        });

        Rectangle::new(vec![
            Text::new(item.mapped(|a: &T| format!("{:?}", *a)))
        ]).fill(color).frame(SCALE, 30.0)
    }

    fn default_popup_delegate(
        model: TState<Vec<T>>,
        delegate: PopupDelegate<T>,
    ) -> Box<dyn Widget> {
        List::new(model, delegate)
            .spacing(0.0)
            .clip()
            .frame(SCALE, 200.0)
    }

    /*fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        _: &bool,
        env: &mut Environment<GS>,
        _: &mut GS,
    ) {
        if !self.is_inside(event.get_current_mouse_position()) {
            match event {
                MouseEvent::Press(_, _, _) => {
                    self.release_focus(env);
                }
                _ => (),
            }
        } else {
            match event {
                MouseEvent::Press(_, _, _) => {
                    self.request_focus(env);
                }
                _ => (),
            }
        }
    }

    fn handle_keyboard_event(
        &mut self,
        event: &KeyboardEvent,
        env: &mut Environment<GS>,
        global_state: &mut GS,
    ) {
        if self.get_focus() != Focus::Focused {
            return;
        }

        match event {
            KeyboardEvent::Press(key, modifier) => match (key, modifier) {
                (Key::Space, _) | (Key::Return, _) => {
                    *self.opened.get_value_mut(env, global_state) = true;
                }
                (_, _) => {}
            },
            _ => {}
        }
    }*/

    /*fn update_all_widget_state(&mut self, env: &mut Environment<GS>, _: &GS) {
        if *self.opened.get_latest_value() {
            let display_item = if let Some(display_item_function) = self.popup_function {
                display_item_function
            } else {
                |item: Box<dyn State<T, GS>>,
                 _parent_selected_index: Box<dyn State<usize, GS>>,
                 _item_index: Box<dyn State<usize, GS>>,
                 partially_chosen: Box<dyn State<bool, GS>>|
                 -> Box<dyn Widget<GS>> {
                    let text = item.mapped(|item| format!("{:?}", item));
                    Rectangle::new(vec![Text::new(text).color(
                        partially_chosen.clone().mapped(|partially_chosen| {
                            if *partially_chosen {
                                Color::Rgba(0.0, 0.0, 0.0, 1.0)
                            } else {
                                Color::Rgba(1.0, 1.0, 1.0, 1.0)
                            }
                        }),
                    )])
                        .fill(partially_chosen.clone().mapped(|partially_chosen| {
                            if *partially_chosen {
                                Color::Rgba(0.0, 1.0, 0.0, 1.0)
                            } else {
                                Color::Rgba(0.0, 0.0, 1.0, 1.0)
                            }
                        }))
                        .border()
                        .border_width(1)
                        .color(EnvironmentColor::Blue)
                }
            };

            let mut overlay = PlainPopUpButtonPopUp::new(
                display_item,
                self.opened.clone(),
                self.model.clone(),
                self.selected_state.clone(),
                self.popup_list_spacing,
                self.dimension,
                env.get_corrected_dimensions(),
            );

            overlay.calculate_size([5000.0, 5000.0], env);

            let popup_x = self.x() - 1.0;

            let mut popup_y = self.y() - overlay.height() / 2.0 + self.height() / 2.0;

            if popup_y < 0.0 {
                popup_y = 0.0;
            } else if popup_y + overlay.height() > env.get_corrected_height() {
                popup_y = env.get_corrected_height() - overlay.height();
            }

            overlay.set_position([popup_x, popup_y]);
            overlay.set_id(self.popup_id);

            env.add_overlay("controls_popup_layer", overlay);
        }
    }*/
}

impl<T: StateContract + PartialEq + 'static> MouseEventHandler for PlainPopUpButton<T> {
    // Implementing this instead of handle_mouse_event makes all the children not receive events.
    fn process_mouse_event(&mut self, event: &MouseEvent, _: &bool, env: &mut Environment) {
        match event {
            MouseEvent::Click(_, position, _) => {
                if self.is_inside(*position) {
                    self.popup.set_showing(true);
                    //println!("{:#?}", self.popup);
                    env.add_overlay("controls_popup_layer", Some(self.popup.clone()));
                }
            }
            _ => ()
        }
    }
}

impl<T: StateContract + PartialEq + 'static> CommonWidget for PlainPopUpButton<T> {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
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

    fn flexibility(&self) -> u32 {
        10
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl<T: StateContract + PartialEq + 'static> Layout for PlainPopUpButton<T> {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let dimensions = self.child.calculate_size(requested_size, env);
        self.set_dimension(dimensions);

        // We calculate the size for the popup if it is open
        if self.popup.is_showing() {
            let popup_request = Dimension::new(dimensions.width, requested_size.height);
            self.popup.calculate_size(popup_request, env);
        }
        dimensions
    }

    fn position_children(&mut self) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();

        if let Some(mut child) = self.children_mut().next() {
            positioning(position, dimension, child.deref_mut());
            child.position_children();
        }

        if self.popup.is_showing() {
            let positioning = self.alignment().positioner();
            let position = self.position();
            let dimension = self.dimension();
            positioning(position, dimension, &mut self.popup as &mut dyn Widget);
            self.popup.position_children();
        }
    }
}

impl<T: StateContract + PartialEq + 'static> Render for PlainPopUpButton<T> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.child.process_get_primitives(primitives, env);
    }
}

impl<T: StateContract + PartialEq + 'static> Debug for PlainPopUpButton<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T: StateContract + PartialEq + 'static> WidgetExt for PlainPopUpButton<T> {}
