//! A function for converting a `winit::Event` to a `carbide::event::Input`.
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{
    ElementState,
    MouseButton as WinitMouseButton,
    MouseScrollDelta,
    Touch as WinitTouch,
    TouchPhase as WinitTouchPhase,
    VirtualKeyCode,
    WindowEvent
};
use winit::window::CursorIcon;

use carbide_core::cursor::MouseCursor;
use carbide_core::draw::Position;
use carbide_core::event::{Button, Gesture, Input, Key, Motion, MouseButton, Touch, TouchId, TouchPhase};
pub use event_loop::*;

mod event_loop;

const ARBITRARY_POINTS_PER_LINE_FACTOR: f64 = 10.0;

/// Types that have access to a `winit::Window` and can provide the necessary dimensions and hidpi
/// factor for converting `winit::Event`s to `carbide::event::Input`, as well as set the mouse
/// cursor.
///
/// This allows users to pass references to window render like `glium::Display`,
/// `glium::glutin::Window` or `winit::Window`
pub trait WinitWindow {
    /// Return the inner size of the window in logical pixels.
    fn get_inner_size(&self) -> Option<(u32, u32)>;
    /// Return the window's DPI factor so that we can convert from pixel values to scalar values.
    fn hidpi_factor(&self) -> f32;
}

/// Maps winit key to carbide core key
pub fn convert_key(key: VirtualKeyCode) -> Key {
    match key {
        VirtualKeyCode::Key0 => Key::D0,
        VirtualKeyCode::Key1 => Key::D1,
        VirtualKeyCode::Key2 => Key::D2,
        VirtualKeyCode::Key3 => Key::D3,
        VirtualKeyCode::Key4 => Key::D4,
        VirtualKeyCode::Key5 => Key::D5,
        VirtualKeyCode::Key6 => Key::D6,
        VirtualKeyCode::Key7 => Key::D7,
        VirtualKeyCode::Key8 => Key::D8,
        VirtualKeyCode::Key9 => Key::D9,
        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,
        VirtualKeyCode::Apostrophe => Key::Unknown,
        VirtualKeyCode::Backslash => Key::Backslash,
        VirtualKeyCode::Back => Key::Backspace,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::Comma => Key::Comma,
        VirtualKeyCode::Down => Key::Down,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::Return => Key::Return,
        VirtualKeyCode::Equals => Key::Equals,
        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::F1 => Key::F1,
        VirtualKeyCode::F2 => Key::F2,
        VirtualKeyCode::F3 => Key::F3,
        VirtualKeyCode::F4 => Key::F4,
        VirtualKeyCode::F5 => Key::F5,
        VirtualKeyCode::F6 => Key::F6,
        VirtualKeyCode::F7 => Key::F7,
        VirtualKeyCode::F8 => Key::F8,
        VirtualKeyCode::F9 => Key::F9,
        VirtualKeyCode::F10 => Key::F10,
        VirtualKeyCode::F11 => Key::F11,
        VirtualKeyCode::F12 => Key::F12,
        VirtualKeyCode::F13 => Key::F13,
        VirtualKeyCode::F14 => Key::F14,
        VirtualKeyCode::F15 => Key::F15,
        VirtualKeyCode::Numpad0 => Key::NumPad0,
        VirtualKeyCode::Numpad1 => Key::NumPad1,
        VirtualKeyCode::Numpad2 => Key::NumPad2,
        VirtualKeyCode::Numpad3 => Key::NumPad3,
        VirtualKeyCode::Numpad4 => Key::NumPad4,
        VirtualKeyCode::Numpad5 => Key::NumPad5,
        VirtualKeyCode::Numpad6 => Key::NumPad6,
        VirtualKeyCode::Numpad7 => Key::NumPad7,
        VirtualKeyCode::Numpad8 => Key::NumPad8,
        VirtualKeyCode::Numpad9 => Key::NumPad9,
        VirtualKeyCode::NumpadComma | VirtualKeyCode::NumpadDecimal => Key::NumPadDecimal,
        VirtualKeyCode::NumpadDivide => Key::NumPadDivide,
        VirtualKeyCode::NumpadMultiply => Key::NumPadMultiply,
        VirtualKeyCode::NumpadSubtract => Key::NumPadMinus,
        VirtualKeyCode::NumpadAdd => Key::NumPadPlus,
        VirtualKeyCode::NumpadEnter => Key::NumPadEnter,
        VirtualKeyCode::NumpadEquals => Key::NumPadEquals,
        VirtualKeyCode::LShift => Key::LShift,
        VirtualKeyCode::LControl => Key::LCtrl,
        VirtualKeyCode::LAlt => Key::LAlt,
        VirtualKeyCode::RShift => Key::RShift,
        VirtualKeyCode::RControl => Key::RCtrl,
        VirtualKeyCode::RAlt => Key::RAlt,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Left => Key::Left,

        // On mac this is the command (cmd) key
        VirtualKeyCode::LWin => Key::LGui,
        VirtualKeyCode::RWin => Key::RGui,

        VirtualKeyCode::LBracket => Key::LeftBracket,
        VirtualKeyCode::Minus => Key::Minus,
        VirtualKeyCode::Numlock => Key::NumLockClear,
        VirtualKeyCode::PageDown => Key::PageDown,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::Pause => Key::Pause,
        VirtualKeyCode::Period => Key::Period,
        VirtualKeyCode::Right => Key::Right,
        VirtualKeyCode::RBracket => Key::RightBracket,
        VirtualKeyCode::Semicolon => Key::Semicolon,
        VirtualKeyCode::Slash => Key::Slash,
        VirtualKeyCode::Space => Key::Space,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Up => Key::Up,
        n => {
            println!("Warning, unknown keycode: {:?}", n);
            Key::Unknown
        },
    }
}

pub fn convert_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Other(0) => MouseButton::X1,
        WinitMouseButton::Other(1) => MouseButton::X2,
        WinitMouseButton::Other(2) => MouseButton::Button6,
        WinitMouseButton::Other(3) => MouseButton::Button7,
        WinitMouseButton::Other(4) => MouseButton::Button8,
        n => {
            println!("Unknown mouse button: {:?}", n);
            MouseButton::Unknown
        },
    }
}

pub fn convert_mouse_cursor(cursor: MouseCursor) -> CursorIcon {
    match cursor {
        MouseCursor::Default => CursorIcon::Arrow,
        MouseCursor::Crosshair => CursorIcon::Crosshair,
        MouseCursor::Hand => CursorIcon::Hand,
        MouseCursor::Arrow => CursorIcon::Arrow,
        MouseCursor::Move => CursorIcon::Move,
        MouseCursor::Text => CursorIcon::Text,
        MouseCursor::Wait => CursorIcon::Wait,
        MouseCursor::Help => CursorIcon::Help,
        MouseCursor::Progress => CursorIcon::Progress,
        MouseCursor::NotAllowed => CursorIcon::NotAllowed,
        MouseCursor::ContextMenu => CursorIcon::ContextMenu,
        MouseCursor::Cell => CursorIcon::Cell,
        MouseCursor::VerticalText => CursorIcon::VerticalText,
        MouseCursor::Alias => CursorIcon::Alias,
        MouseCursor::Copy => CursorIcon::Copy,
        MouseCursor::NoDrop => CursorIcon::NoDrop,
        MouseCursor::Grab => CursorIcon::Grab,
        MouseCursor::Grabbing => CursorIcon::Grabbing,
        MouseCursor::AllScroll => CursorIcon::AllScroll,
        MouseCursor::ZoomIn => CursorIcon::ZoomIn,
        MouseCursor::ZoomOut => CursorIcon::ZoomOut,
        MouseCursor::EResize => CursorIcon::EResize,
        MouseCursor::NResize => CursorIcon::NResize,
        MouseCursor::NeResize => CursorIcon::NeResize,
        MouseCursor::NwResize => CursorIcon::NwResize,
        MouseCursor::SResize => CursorIcon::SResize,
        MouseCursor::SeResize => CursorIcon::SeResize,
        MouseCursor::SwResize => CursorIcon::SwResize,
        MouseCursor::WResize => CursorIcon::WResize,
        MouseCursor::EwResize => CursorIcon::EwResize,
        MouseCursor::NsResize => CursorIcon::NsResize,
        MouseCursor::NeswResize => CursorIcon::NeswResize,
        MouseCursor::NwseResize => CursorIcon::NwseResize,
        MouseCursor::ColResize => CursorIcon::ColResize,
        MouseCursor::RowResize => CursorIcon::RowResize,
        MouseCursor::Custom(_) => CursorIcon::Arrow,
    }
}

pub fn convert_window_event(event: &WindowEvent) -> Option<Input> {
    // FIXME: We should not hardcode scale factor. When we convert physical to logical, this
    // should be depending on the current window, because the event can be propagated to more than
    // one window.

    // The window size in points.
    let scale_factor: f64 = 2.0;

    match event {
        WindowEvent::Resized(physical_size) => {
            let LogicalSize { width, height } = physical_size.to_logical(scale_factor);
            Some(Input::Resize(width, height))
        }
        WindowEvent::ReceivedCharacter(ch) => {
            let string = match ch {
                // Ignore control characters and return ascii for Text event (like sdl2).
                '\u{7f}' | // Delete
                '\u{1b}' | // Escape
                '\u{8}'  | // Backspace
                '\r' | '\n' | '\t' => "".to_string(),
                _ => ch.to_string()
            };
            Some(Input::Text(string))
        }
        WindowEvent::Focused(focused) => {
            Some(Input::Focus(*focused))
        }
        WindowEvent::KeyboardInput { input, .. } => {
            input.virtual_keycode.map(|key| {
                let key = convert_key(key);
                match input.state {
                    ElementState::Pressed => Input::Press(Button::Keyboard(key)),
                    ElementState::Released => Input::Release(Button::Keyboard(key)),
                }
            })
        }
        WindowEvent::Touch(WinitTouch { phase, location, id, .. }) => {
            let LogicalPosition { x, y } = location.to_logical::<f64>(scale_factor);

            let phase = match phase {
                WinitTouchPhase::Started => TouchPhase::Start,
                WinitTouchPhase::Moved => TouchPhase::Move,
                WinitTouchPhase::Cancelled => TouchPhase::Cancel,
                WinitTouchPhase::Ended => TouchPhase::End,
            };

            let id = TouchId::new(id.clone());

            let touch = Touch {
                phase,
                id,
                position: Position::new(x, y)
            };

            Some(Input::Touch(touch))
        }
        WindowEvent::CursorMoved { position, .. } => {
            let LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor);

            Some(Input::Motion(Motion::MouseCursor { x, y }))
        }
        WindowEvent::MouseWheel { delta, .. } => {
            match delta {
                MouseScrollDelta::PixelDelta(delta) => {
                    let LogicalPosition { x, y } = delta.to_logical::<f64>(scale_factor);
                    let x = x;
                    let y = -y;

                    Some(Input::Motion(Motion::Scroll { x, y }))
                }
                MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * *x as carbide_core::draw::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -*y as carbide_core::draw::Scalar;

                    Some(Input::Motion(Motion::Scroll { x, y }))
                }
            }
        }
        WindowEvent::SmartMagnify { .. } => {
            Some(Input::Gesture(Gesture::SmartScale))
        }
        WindowEvent::TouchpadRotate { delta, phase, .. } => {
            let phase = match phase {
                WinitTouchPhase::Started => TouchPhase::Start,
                WinitTouchPhase::Moved => TouchPhase::Move,
                WinitTouchPhase::Cancelled => TouchPhase::Cancel,
                WinitTouchPhase::Ended => TouchPhase::End,
            };

            Some(Input::Gesture(Gesture::Rotate(
                *delta as f64 / scale_factor,
                phase
            )))
        }
        WindowEvent::TouchpadMagnify {delta, phase, .. } => {
            let phase = match phase {
                WinitTouchPhase::Started => TouchPhase::Start,
                WinitTouchPhase::Moved => TouchPhase::Move,
                WinitTouchPhase::Cancelled => TouchPhase::Cancel,
                WinitTouchPhase::Ended => TouchPhase::End,
            };

            Some(Input::Gesture(Gesture::Scale(
                *delta / scale_factor,
                phase
            )))
        }
        WindowEvent::MouseInput { state, button, .. } => {

            let mouse_button = convert_mouse_button(*button);
            match state {
                ElementState::Pressed => Some(Input::Press(Button::Mouse(mouse_button))),
                ElementState::Released => Some(Input::Release(Button::Mouse(mouse_button))),
            }
        }
        WindowEvent::CloseRequested => {
            Some(Input::CloseRequested)
        }
        _ => None,
    }
}