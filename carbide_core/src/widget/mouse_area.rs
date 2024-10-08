use std::fmt::{Debug, Formatter};

use dyn_clone::DynClone;

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::cursor::MouseCursor;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::event::{
    Key, KeyboardEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent,
    MouseEventHandler, KeyboardEventContext, MouseEventContext
};
use crate::flags::WidgetFlag;
use crate::focus::Focus;
use crate::state::{IntoState, State};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, Empty};

pub trait Action: Fn(&mut Environment, ModifierKey) + DynClone {}

impl<I> Action for I where I: Fn(&mut Environment, ModifierKey) + Clone {}

dyn_clone::clone_trait_object!(Action);

#[derive(Clone, Widget)]
#[carbide_exclude(MouseEvent, KeyboardEvent)]
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
> KeyboardEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        if self.get_focus() != Focus::Focused {
            return;
        }

        match event {
            KeyboardEvent::Press(Key::Enter, _) => {
                self.is_pressed.set_value(true);
            }
            KeyboardEvent::Release(Key::Enter, _) => {
                if *self.is_pressed.value() {
                    self.is_pressed.set_value(false);
                    (self.click)(ctx.env, ModifierKey::empty());
                } else {
                    self.is_pressed.set_value(false);
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
> MouseEventHandler for MouseArea<I, O, F, C, H, P> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        match event {
            MouseEvent::Press { button: MouseButton::Left, position: mouse_position, .. } => {
                if self.is_inside(*mouse_position) {
                    self.is_pressed.set_value(true);
                }
            }
            MouseEvent::Release { button: MouseButton::Left, position: mouse_position, .. } => {
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
                    //self.request_focus(ctx.env);
                    (self.click)(ctx.env, *modifier);
                } else {
                    //self.set_focus(Focus::Unfocused);
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
    CommonWidgetImpl!(self, id: self.id, child: self.child, flag: WidgetFlag::FOCUSABLE, focus: self.focus, position: self.position, dimension: self.dimension);

    fn cursor(&self) -> Option<MouseCursor> {
        if *self.is_hovered.value() {
            return Some(self.hover_cursor)
        }

        if *self.is_pressed.value() {
            return self.pressed_cursor;
        }

        None
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
