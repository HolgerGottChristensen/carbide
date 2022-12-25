use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{
    Key, KeyboardEvent, KeyboardEventHandler, MouseButton, MouseEvent, MouseEventHandler,
};
use carbide_core::state::{State, TState};
use carbide_core::widget::{CommonWidget, Widget, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent)]
pub struct PlainPopUpButtonPopUp {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    hover_model: TState<Vec<bool>>,
}

impl PlainPopUpButtonPopUp {
    pub fn new(child: Box<dyn Widget>, hover_model: TState<Vec<bool>>) -> Box<Self> {
        Box::new(PlainPopUpButtonPopUp {
            id: WidgetId::new(),
            child,
            position: Default::default(),
            dimension: Default::default(),
            hover_model,
        })
    }

    fn close_overlay(env: &mut Environment) {
        env.add_overlay("controls_popup_layer", None)
    }
}

impl KeyboardEventHandler for PlainPopUpButtonPopUp {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        match event {
            KeyboardEvent::Press(key, _) => match key {
                Key::Escape => Self::close_overlay(env),
                Key::Up => {
                    let mut hovers = self.hover_model.value_mut();
                    let mut true_at_any_point = false;
                    let mut last_item_had_hover = false;

                    for hover in hovers.iter_mut().rev() {
                        if last_item_had_hover {
                            last_item_had_hover = false;
                            *hover = true;
                        } else if *hover {
                            last_item_had_hover = true;
                            true_at_any_point = true;
                            *hover = false;
                        }
                    }
                    let hover_len = hovers.len();
                    if last_item_had_hover || (!true_at_any_point && hover_len > 0) {
                        hovers[hover_len - 1] = true;
                    }
                }
                Key::Down => {
                    let mut hovers = self.hover_model.value_mut();
                    let mut true_at_any_point = false;
                    let mut last_item_had_hover = false;

                    for hover in hovers.iter_mut() {
                        if last_item_had_hover {
                            last_item_had_hover = false;
                            *hover = true;
                        } else if *hover {
                            last_item_had_hover = true;
                            true_at_any_point = true;
                            *hover = false;
                        }
                    }

                    if last_item_had_hover || (!true_at_any_point && hovers.len() > 0) {
                        hovers[0] = true;
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

impl MouseEventHandler for PlainPopUpButtonPopUp {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, env: &mut Environment) {
        match event {
            MouseEvent::Click(MouseButton::Left, mouse_position, _) => {
                if !self.is_inside(*mouse_position) {
                    Self::close_overlay(env)
                }
            }
            _ => (),
        }
    }
}

impl CommonWidget for PlainPopUpButtonPopUp {
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
