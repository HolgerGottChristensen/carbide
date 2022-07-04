use std::ops::DerefMut;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{
    Key, KeyboardEvent, KeyboardEventHandler, MouseButton, MouseEvent, MouseEventHandler,
};
use carbide_core::state::{ReadState, State, StateContract, TState};
use carbide_core::widget::{CommonWidget, Widget, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent)]
pub struct PlainPopUpButtonPopUpItem<T>
where
    T: StateContract,
{
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    hovered: TState<bool>,
    selected_item: TState<T>,
    item: TState<T>,
}

impl<T: StateContract> PlainPopUpButtonPopUpItem<T> {
    pub fn new(
        child: Box<dyn Widget>,
        hovered: TState<bool>,
        item: TState<T>,
        selected_item: TState<T>,
    ) -> Box<Self> {
        Box::new(PlainPopUpButtonPopUpItem {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            hovered,
            selected_item,
            item,
        })
    }

    fn close_overlay(env: &mut Environment) {
        env.add_overlay("controls_popup_layer", None)
    }
}

impl<T: StateContract> KeyboardEventHandler for PlainPopUpButtonPopUpItem<T> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        match event {
            KeyboardEvent::Press(key, _) => match key {
                Key::Return | Key::Return2 => {
                    if *self.hovered.value() {
                        *self.selected_item.value_mut() = self.item.value().clone();
                        Self::close_overlay(env);
                        env.request_animation_frame();
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

impl<T: StateContract> MouseEventHandler for PlainPopUpButtonPopUpItem<T> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, env: &mut Environment) {
        match event {
            MouseEvent::Click(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    *self.selected_item.value_mut() = self.item.value().clone();
                    Self::close_overlay(env);
                    env.request_animation_frame();
                }
            }
            _ => (),
        }
    }
}

impl<T: StateContract> CommonWidget for PlainPopUpButtonPopUpItem<T> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::single(&self.child)
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
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

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}
