use dyn_clone::DynClone;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{Key, KeyboardEvent, KeyboardEventHandler, MouseButton, MouseEvent, MouseEventHandler};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::layout::Layout;
use carbide_core::state::{BoolState, FocusState, State};
use carbide_core::widget::{CommonWidget, Id, Widget, WidgetExt, WidgetIter, WidgetIterMut};

pub trait Action: Fn(&mut Environment) + DynClone {}

impl<I> Action for I where I: Fn(&mut Environment) + Clone {}

dyn_clone::clone_trait_object!(Action);

#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, Layout)]
pub struct PlainButton {
    id: Id,
    #[state]
    focus: FocusState,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    click: Box<dyn Action>,
    click_outside: Box<dyn Action>,
    #[state]
    is_hovered: BoolState,
    #[state]
    is_pressed: BoolState,
}

impl PlainButton {
    pub fn on_click(
        mut self,
        fire: impl Action + 'static,
    ) -> Box<Self> {
        self.click = Box::new(fire);
        Box::new(self)
    }

    pub fn on_click_outside(
        mut self,
        fire: impl Action + 'static,
    ) -> Box<Self> {
        self.click_outside = Box::new(fire);
        Box::new(self)
    }

    pub fn hover<K: Into<BoolState>>(mut self, is_hovered: K) -> Box<Self> {
        self.is_hovered = is_hovered.into();
        Box::new(self)
    }

    pub fn pressed<K: Into<BoolState>>(mut self, pressed: K) -> Box<Self> {
        self.is_pressed = pressed.into();
        Box::new(self)
    }

    pub fn focused<K: Into<FocusState>>(mut self, focused: K) -> Box<Self> {
        self.focus = focused.into();
        Box::new(self)
    }

    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(PlainButton {
            id: Id::new_v4(),
            focus: Focus::Unfocused.into(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            click: Box::new(|_| {}),
            click_outside: Box::new(|_| {}),
            is_hovered: false.into(),
            is_pressed: false.into(),
        })
    }
}

impl KeyboardEventHandler for PlainButton {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.get_focus() != Focus::Focused {
            return;
        }

        match event {
            KeyboardEvent::Click(Key::Return, _) => {
                (self.click)(env);
            }
            _ => (),
        }
    }
}

impl MouseEventHandler for PlainButton {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, env: &mut Environment) {
        match event {
            MouseEvent::Press(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    *self.is_pressed.value_mut() = true;
                }
            }
            MouseEvent::Release(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    *self.is_pressed.value_mut() = false;
                }
            }
            MouseEvent::Move { to, .. } => {
                if *self.is_hovered.value() {
                    if !self.is_inside(*to) {
                        *self.is_hovered.value_mut() = false;
                        *self.is_pressed.value_mut() = false;
                    }
                } else {
                    if self.is_inside(*to) {
                        *self.is_hovered.value_mut() = true;
                    }
                }
            }
            MouseEvent::Click(MouseButton::Left, mouse_position, _)
            | MouseEvent::NClick(MouseButton::Left, mouse_position, _, _) => {
                if self.is_inside(*mouse_position) {
                    (self.click)(env);
                } else {
                    (self.click_outside)(env);
                }
            }
            _ => (),
        }
    }
}

impl Layout for PlainButton {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.child.calculate_size(requested_size, env);
        self.dimension = requested_size;
        self.dimension
    }
}

impl CommonWidget for PlainButton {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        *self.focus.value_mut() = focus;
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

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for PlainButton {}
