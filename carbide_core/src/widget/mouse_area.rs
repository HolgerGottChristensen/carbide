use std::fmt::{Debug, Formatter};

use dyn_clone::DynClone;

use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::event::{
    Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent,
    MouseEventHandler, OtherEventHandler, WidgetEvent,
};
use carbide_core::flags::Flags;
use carbide_core::focus::Focus;
use carbide_core::state::State;
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};
use carbide_macro::carbide_default_builder2;

use crate::event::{MouseEventContext, OtherEventContext};
use crate::state::IntoState;
use crate::widget::Empty;

pub trait Action: Fn(&mut Environment, ModifierKey) + DynClone {}

impl<I> Action for I where I: Fn(&mut Environment, ModifierKey) + Clone {}

dyn_clone::clone_trait_object!(Action);

#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent, OtherEvent)]
pub struct MouseArea<I, O, F, C, H, P> where
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
{
    id: WidgetId,
    #[state] focus: F,
    child: C,
    position: Position,
    dimension: Dimension,
    click: I,
    click_outside: O,
    #[state] is_hovered: H,
    #[state] is_pressed: P,
    hover_cursor: MouseCursor,
    pressed_cursor: Option<MouseCursor>,
}

impl MouseArea<fn(&mut Environment, ModifierKey), fn(&mut Environment, ModifierKey), Focus, Empty, bool, bool> {

    #[carbide_default_builder2]
    pub fn new<C: Widget>(child: C) -> MouseArea<fn(&mut Environment, ModifierKey), fn(&mut Environment, ModifierKey), Focus, C, bool, bool> {
        MouseArea {
            id: WidgetId::new(),
            focus: Focus::Unfocused,
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            click: |_, _| {},
            click_outside: |_, _| {},
            is_hovered: false,
            is_pressed: false,
            hover_cursor: MouseCursor::Pointer,
            pressed_cursor: None,
        }
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> MouseArea<I, O, F, C, H, P> {
    /// Example: .on_click(move |env: &mut Environment, modifier: ModifierKey| {})
    pub fn on_click<A: Action + Clone>(self, action: A) -> MouseArea<A, O, F, C, H, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: action,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn on_click_outside<A: Action + Clone>(self, action: A) -> MouseArea<I, A, F, C, H, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: action,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn hovered<T: IntoState<bool>>(self, is_hovered: T) -> MouseArea<I, O, F, C, T::Output, P> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: self.click_outside,
            is_hovered: is_hovered.into_state(),
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn pressed<T: IntoState<bool>>(self, pressed: T) -> MouseArea<I, O, F, C, H, T::Output> {
        MouseArea {
            id: self.id,
            focus: self.focus,
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: pressed.into_state(),
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn focused<T: IntoState<Focus>>(self, focused: T) -> MouseArea<I, O, <T as IntoState<Focus>>::Output, C, H, P> {
        MouseArea {
            id: self.id,
            focus: focused.into_state(),
            child: self.child,
            position: self.position,
            dimension: self.dimension,
            click: self.click,
            click_outside: self.click_outside,
            is_hovered: self.is_hovered,
            is_pressed: self.is_pressed,
            hover_cursor: self.hover_cursor,
            pressed_cursor: self.pressed_cursor,
        }
    }

    pub fn hover_cursor(mut self, cursor: MouseCursor) -> Self {
        self.hover_cursor = cursor;
        self
    }

    pub fn pressed_cursor(mut self, cursor: MouseCursor) -> Self {
        self.pressed_cursor = Some(cursor);
        self
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> OtherEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_other_event(&mut self, _event: &WidgetEvent, ctx: &mut OtherEventContext) {
        if *self.is_hovered.value() {
            ctx.env.set_cursor(self.hover_cursor);
        }
        if *self.is_pressed.value() {
            if let Some(cursor) = self.pressed_cursor {
                ctx.env.set_cursor(cursor);
            }
        }
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> KeyboardEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        if self.get_focus() != Focus::Focused {
            return;
        }

        match event {
            KeyboardEvent::Press(Key::Enter, _) => {
                self.is_pressed.set_value(true);
            }
            KeyboardEvent::Click(Key::Enter, _) => {
                self.is_pressed.set_value(false);
                (self.click)(env, ModifierKey::NO_MODIFIER);
            }
            _ => (),
        }
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> MouseEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Press(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    self.is_pressed.set_value(true);
                }
            }
            MouseEvent::Release(MouseButton::Left, mouse_position, _) => {
                if self.is_inside(*mouse_position) {
                    self.is_pressed.set_value(false);
                }
            }
            MouseEvent::Move { to, .. } => {
                if *self.is_hovered.value() {
                    if !self.is_inside(*to) {
                        self.is_pressed.set_value(false);
                        self.is_hovered.set_value(false);
                    }
                } else {
                    if self.is_inside(*to) {
                        self.is_hovered.set_value(true);
                    }
                }
            }
            MouseEvent::Click(MouseButton::Left, mouse_position, modifier)
            | MouseEvent::NClick(MouseButton::Left, mouse_position, modifier, _) => {
                if self.is_inside(*mouse_position) {
                    (self.click)(ctx.env, *modifier);
                } else {
                    (self.click_outside)(ctx.env, *modifier);
                }
            }
            _ => (),
        }
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> CommonWidget for MouseArea<I, O, F, C, H, P> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        Flags::FOCUSABLE
    }

    fn get_focus(&self) -> Focus {
        self.focus.value().clone()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.focus.set_value(focus);
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

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> Debug for MouseArea<I, O, F, C, H, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MouseArea")
            .field("child", &self.child)
            .finish()
    }
}

impl<
    I: Action + Clone + 'static,
    O: Action + Clone + 'static,
    F: State<T=Focus>,
    C: Widget,
    H: State<T=bool>,
    P: State<T=bool>,
> WidgetExt for MouseArea<I, O, F, C, H, P> {}
