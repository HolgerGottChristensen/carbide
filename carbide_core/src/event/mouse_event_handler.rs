use std::time::Duration;
use crate::draw::{ImageContext, Position, Scalar};
use crate::environment::{Environment};
use crate::event::{EventId, ModifierKey, TouchPhase};
use crate::focus::Focusable;
use crate::text::TextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait MouseEventHandler: CommonWidget + WidgetSync + Focusable {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    #[allow(unused_variables)]
    fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {}

    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        if !*ctx.consumed && *ctx.is_current {
            self.sync(ctx.env);
            self.handle_mouse_event(event, ctx);
        }

        self.foreach_child_direct(&mut |child| {
            child.process_mouse_event(event, ctx);
            if *ctx.consumed {
                return;
            }
        });
    }
}


// TODO: Consider changing to Event Context
pub struct MouseEventContext<'a, 'b: 'a> {
    pub text: &'a mut dyn TextContext,
    pub image: &'a mut dyn ImageContext,
    pub is_current: &'a bool,
    pub window_id: &'a u64,
    pub consumed: &'a mut bool,
    pub env: &'a mut Environment<'b>,
}

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum MouseButton {
    /// Unknown mouse button.
    Unknown,
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// Extra mouse button number 1.
    Back,
    /// Extra mouse button number 2.
    Forward,
    /// Mouse button number 6.
    Button6,
    /// Mouse button number 7.
    Button7,
    /// Mouse button number 8.
    Button8,
}

impl From<u32> for MouseButton {
    fn from(n: u32) -> MouseButton {
        match n {
            0 => MouseButton::Unknown,
            1 => MouseButton::Left,
            2 => MouseButton::Right,
            3 => MouseButton::Middle,
            4 => MouseButton::Back,
            5 => MouseButton::Forward,
            6 => MouseButton::Button6,
            7 => MouseButton::Button7,
            8 => MouseButton::Button8,
            _ => MouseButton::Unknown,
        }
    }
}

impl From<MouseButton> for u32 {
    fn from(button: MouseButton) -> u32 {
        match button {
            MouseButton::Unknown => 0,
            MouseButton::Left => 1,
            MouseButton::Right => 2,
            MouseButton::Middle => 3,
            MouseButton::Back => 4,
            MouseButton::Forward => 5,
            MouseButton::Button6 => 6,
            MouseButton::Button7 => 7,
            MouseButton::Button8 => 8,
        }
    }
}


#[derive(Clone, Debug)]
pub enum MouseEvent {
    Press {
        id: EventId,
        button: MouseButton,
        position: Position,
        modifiers: ModifierKey
    },
    Release {
        id: EventId,
        button: MouseButton,
        position: Position,
        modifiers: ModifierKey,
        press_id: EventId,
        duration: Duration,
    },
    Click(MouseButton, Position, ModifierKey),
    NClick(MouseButton, Position, ModifierKey, u32),
    Move {
        from: Position,
        to: Position,
        delta_xy: Position,
        modifiers: ModifierKey,
    },
    Scroll {
        x: Scalar,
        y: Scalar,
        mouse_position: Position,
        modifiers: ModifierKey,
    },
    Rotation(Scalar, Position, TouchPhase),
    Scale(Scalar, Position, TouchPhase),
    SmartScale(Position),
    Drag {
        button: MouseButton,
        origin: Position,
        from: Position,
        to: Position,
        delta_xy: Position,
        total_delta_xy: Position,
        modifiers: ModifierKey,
    },
    Entered,
    Left,
}

impl MouseEvent {
    pub fn get_current_mouse_position(&self) -> Position {
        match self {
            MouseEvent::Press { position: n, .. } => *n,
            MouseEvent::Release { position: n, .. } => *n,
            MouseEvent::Click(_, n, _) => *n,
            MouseEvent::Move { to, .. } => *to,
            MouseEvent::NClick(_, n, _, _) => *n,
            MouseEvent::Scroll { mouse_position, .. } => *mouse_position,
            MouseEvent::Drag { to, .. } => *to,
            MouseEvent::Rotation(_, position, _) |
            MouseEvent::Scale(_, position, _) |
            MouseEvent::SmartScale(position) => *position,
            MouseEvent::Entered => todo!(),
            MouseEvent::Left => todo!(),
        }
    }

    pub fn id(&self) -> EventId {
        match self {
            MouseEvent::Press { id, .. } => *id,
            MouseEvent::Release { id, .. } => *id,
            _ => unimplemented!(),
        }
    }
}