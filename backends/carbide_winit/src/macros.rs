//! A collection of macros for generating code related to winit+carbide interop.
//!
//! The reason we provide macros and don't implement functions using the `winit` crate directly is
//! that carbide has many backends that use `winit`, often with differing versions. By providing
//! macros, we allow these backends to generate code necessary for whatever version of winit they
//! are currently using. This means we don't have to wait for all of the backend winit dependencies
//! to synchronise before we can publish new carbide releases.

/// Maps winit's key to a carbide `Key`.
///
/// Expects a `winit::VirtualKeyCode` as input and returns a `carbide_core::input::keyboard::Key`.
///
/// Requires that both the `winit` and `carbide_core` crates exist within the crate root.
#[macro_export]
macro_rules! convert_key {
    ($keycode:expr) => {{
        match $keycode {
            winit::VirtualKeyCode::Key0 => carbide_core::input::keyboard::Key::D0,
            winit::VirtualKeyCode::Key1 => carbide_core::input::keyboard::Key::D1,
            winit::VirtualKeyCode::Key2 => carbide_core::input::keyboard::Key::D2,
            winit::VirtualKeyCode::Key3 => carbide_core::input::keyboard::Key::D3,
            winit::VirtualKeyCode::Key4 => carbide_core::input::keyboard::Key::D4,
            winit::VirtualKeyCode::Key5 => carbide_core::input::keyboard::Key::D5,
            winit::VirtualKeyCode::Key6 => carbide_core::input::keyboard::Key::D6,
            winit::VirtualKeyCode::Key7 => carbide_core::input::keyboard::Key::D7,
            winit::VirtualKeyCode::Key8 => carbide_core::input::keyboard::Key::D8,
            winit::VirtualKeyCode::Key9 => carbide_core::input::keyboard::Key::D9,
            winit::VirtualKeyCode::A => carbide_core::input::keyboard::Key::A,
            winit::VirtualKeyCode::B => carbide_core::input::keyboard::Key::B,
            winit::VirtualKeyCode::C => carbide_core::input::keyboard::Key::C,
            winit::VirtualKeyCode::D => carbide_core::input::keyboard::Key::D,
            winit::VirtualKeyCode::E => carbide_core::input::keyboard::Key::E,
            winit::VirtualKeyCode::F => carbide_core::input::keyboard::Key::F,
            winit::VirtualKeyCode::G => carbide_core::input::keyboard::Key::G,
            winit::VirtualKeyCode::H => carbide_core::input::keyboard::Key::H,
            winit::VirtualKeyCode::I => carbide_core::input::keyboard::Key::I,
            winit::VirtualKeyCode::J => carbide_core::input::keyboard::Key::J,
            winit::VirtualKeyCode::K => carbide_core::input::keyboard::Key::K,
            winit::VirtualKeyCode::L => carbide_core::input::keyboard::Key::L,
            winit::VirtualKeyCode::M => carbide_core::input::keyboard::Key::M,
            winit::VirtualKeyCode::N => carbide_core::input::keyboard::Key::N,
            winit::VirtualKeyCode::O => carbide_core::input::keyboard::Key::O,
            winit::VirtualKeyCode::P => carbide_core::input::keyboard::Key::P,
            winit::VirtualKeyCode::Q => carbide_core::input::keyboard::Key::Q,
            winit::VirtualKeyCode::R => carbide_core::input::keyboard::Key::R,
            winit::VirtualKeyCode::S => carbide_core::input::keyboard::Key::S,
            winit::VirtualKeyCode::T => carbide_core::input::keyboard::Key::T,
            winit::VirtualKeyCode::U => carbide_core::input::keyboard::Key::U,
            winit::VirtualKeyCode::V => carbide_core::input::keyboard::Key::V,
            winit::VirtualKeyCode::W => carbide_core::input::keyboard::Key::W,
            winit::VirtualKeyCode::X => carbide_core::input::keyboard::Key::X,
            winit::VirtualKeyCode::Y => carbide_core::input::keyboard::Key::Y,
            winit::VirtualKeyCode::Z => carbide_core::input::keyboard::Key::Z,
            winit::VirtualKeyCode::Apostrophe => carbide_core::input::keyboard::Key::Unknown,
            winit::VirtualKeyCode::Backslash => carbide_core::input::keyboard::Key::Backslash,
            winit::VirtualKeyCode::Back => carbide_core::input::keyboard::Key::Backspace,
            // K::CapsLock => Key::CapsLock,
            winit::VirtualKeyCode::Delete => carbide_core::input::keyboard::Key::Delete,
            winit::VirtualKeyCode::Comma => carbide_core::input::keyboard::Key::Comma,
            winit::VirtualKeyCode::Down => carbide_core::input::keyboard::Key::Down,
            winit::VirtualKeyCode::End => carbide_core::input::keyboard::Key::End,
            winit::VirtualKeyCode::Return => carbide_core::input::keyboard::Key::Return,
            winit::VirtualKeyCode::Equals => carbide_core::input::keyboard::Key::Equals,
            winit::VirtualKeyCode::Escape => carbide_core::input::keyboard::Key::Escape,
            winit::VirtualKeyCode::F1 => carbide_core::input::keyboard::Key::F1,
            winit::VirtualKeyCode::F2 => carbide_core::input::keyboard::Key::F2,
            winit::VirtualKeyCode::F3 => carbide_core::input::keyboard::Key::F3,
            winit::VirtualKeyCode::F4 => carbide_core::input::keyboard::Key::F4,
            winit::VirtualKeyCode::F5 => carbide_core::input::keyboard::Key::F5,
            winit::VirtualKeyCode::F6 => carbide_core::input::keyboard::Key::F6,
            winit::VirtualKeyCode::F7 => carbide_core::input::keyboard::Key::F7,
            winit::VirtualKeyCode::F8 => carbide_core::input::keyboard::Key::F8,
            winit::VirtualKeyCode::F9 => carbide_core::input::keyboard::Key::F9,
            winit::VirtualKeyCode::F10 => carbide_core::input::keyboard::Key::F10,
            winit::VirtualKeyCode::F11 => carbide_core::input::keyboard::Key::F11,
            winit::VirtualKeyCode::F12 => carbide_core::input::keyboard::Key::F12,
            winit::VirtualKeyCode::F13 => carbide_core::input::keyboard::Key::F13,
            winit::VirtualKeyCode::F14 => carbide_core::input::keyboard::Key::F14,
            winit::VirtualKeyCode::F15 => carbide_core::input::keyboard::Key::F15,
            winit::VirtualKeyCode::Numpad0 => carbide_core::input::keyboard::Key::NumPad0,
            winit::VirtualKeyCode::Numpad1 => carbide_core::input::keyboard::Key::NumPad1,
            winit::VirtualKeyCode::Numpad2 => carbide_core::input::keyboard::Key::NumPad2,
            winit::VirtualKeyCode::Numpad3 => carbide_core::input::keyboard::Key::NumPad3,
            winit::VirtualKeyCode::Numpad4 => carbide_core::input::keyboard::Key::NumPad4,
            winit::VirtualKeyCode::Numpad5 => carbide_core::input::keyboard::Key::NumPad5,
            winit::VirtualKeyCode::Numpad6 => carbide_core::input::keyboard::Key::NumPad6,
            winit::VirtualKeyCode::Numpad7 => carbide_core::input::keyboard::Key::NumPad7,
            winit::VirtualKeyCode::Numpad8 => carbide_core::input::keyboard::Key::NumPad8,
            winit::VirtualKeyCode::Numpad9 => carbide_core::input::keyboard::Key::NumPad9,
            winit::VirtualKeyCode::NumpadComma => carbide_core::input::keyboard::Key::NumPadDecimal,
            winit::VirtualKeyCode::Divide => carbide_core::input::keyboard::Key::NumPadDivide,
            winit::VirtualKeyCode::Multiply => carbide_core::input::keyboard::Key::NumPadMultiply,
            winit::VirtualKeyCode::Subtract => carbide_core::input::keyboard::Key::NumPadMinus,
            winit::VirtualKeyCode::Add => carbide_core::input::keyboard::Key::NumPadPlus,
            winit::VirtualKeyCode::NumpadEnter => carbide_core::input::keyboard::Key::NumPadEnter,
            winit::VirtualKeyCode::NumpadEquals => carbide_core::input::keyboard::Key::NumPadEquals,
            winit::VirtualKeyCode::LShift => carbide_core::input::keyboard::Key::LShift,
            winit::VirtualKeyCode::LControl => carbide_core::input::keyboard::Key::LCtrl,
            winit::VirtualKeyCode::LAlt => carbide_core::input::keyboard::Key::LAlt,
            winit::VirtualKeyCode::RShift => carbide_core::input::keyboard::Key::RShift,
            winit::VirtualKeyCode::RControl => carbide_core::input::keyboard::Key::RCtrl,
            winit::VirtualKeyCode::RAlt => carbide_core::input::keyboard::Key::RAlt,
            winit::VirtualKeyCode::Home => carbide_core::input::keyboard::Key::Home,
            winit::VirtualKeyCode::Insert => carbide_core::input::keyboard::Key::Insert,
            winit::VirtualKeyCode::Left => carbide_core::input::keyboard::Key::Left,
            winit::VirtualKeyCode::LBracket => carbide_core::input::keyboard::Key::LeftBracket,
            winit::VirtualKeyCode::Minus => carbide_core::input::keyboard::Key::Minus,
            winit::VirtualKeyCode::Numlock => carbide_core::input::keyboard::Key::NumLockClear,
            winit::VirtualKeyCode::PageDown => carbide_core::input::keyboard::Key::PageDown,
            winit::VirtualKeyCode::PageUp => carbide_core::input::keyboard::Key::PageUp,
            winit::VirtualKeyCode::Pause => carbide_core::input::keyboard::Key::Pause,
            winit::VirtualKeyCode::Period => carbide_core::input::keyboard::Key::Period,
            winit::VirtualKeyCode::Right => carbide_core::input::keyboard::Key::Right,
            winit::VirtualKeyCode::RBracket => carbide_core::input::keyboard::Key::RightBracket,
            winit::VirtualKeyCode::Semicolon => carbide_core::input::keyboard::Key::Semicolon,
            winit::VirtualKeyCode::Slash => carbide_core::input::keyboard::Key::Slash,
            winit::VirtualKeyCode::Space => carbide_core::input::keyboard::Key::Space,
            winit::VirtualKeyCode::Tab => carbide_core::input::keyboard::Key::Tab,
            winit::VirtualKeyCode::Up => carbide_core::input::keyboard::Key::Up,
            _ => carbide_core::input::keyboard::Key::Unknown,
        }
    }};
}

/// Maps winit's mouse button to carbide's mouse button.
///
/// Expects a `winit::MouseButton` as input and returns a `carbide_core::input::MouseButton` as
/// output.
///
/// Requires that both the `carbide_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! convert_mouse_button {
    ($mouse_button:expr) => {{
        match $mouse_button {
            winit::MouseButton::Left => carbide_core::input::MouseButton::Left,
            winit::MouseButton::Right => carbide_core::input::MouseButton::Right,
            winit::MouseButton::Middle => carbide_core::input::MouseButton::Middle,
            winit::MouseButton::Other(0) => carbide_core::input::MouseButton::X1,
            winit::MouseButton::Other(1) => carbide_core::input::MouseButton::X2,
            winit::MouseButton::Other(2) => carbide_core::input::MouseButton::Button6,
            winit::MouseButton::Other(3) => carbide_core::input::MouseButton::Button7,
            winit::MouseButton::Other(4) => carbide_core::input::MouseButton::Button8,
            _ => carbide_core::input::MouseButton::Unknown,
        }
    }};
}

/// A macro for converting a `winit::WindowEvent` to a `Option<carbide_core::event::Input>`.
///
/// Expects a `winit::WindowEvent` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<carbide_core::event::Input>`.
#[macro_export]
macro_rules! convert_window_event {
    ($event:expr, $window:expr) => {{
        // The window size in points.
        let (win_w, win_h) = match $window.get_inner_size() {
            Some((w, h)) => (w as carbide_core::Scalar, h as carbide_core::Scalar),
            None => return None,
        };

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        let tx = |x: carbide_core::Scalar| x - win_w / 2.0;
        let ty = |y: carbide_core::Scalar| -(y - win_h / 2.0);

        // Functions for converting keys and mouse buttons.
        let map_key = |key| $crate::convert_key!(key);
        let map_mouse = |button| $crate::convert_mouse_button!(button);

        match $event {
            winit::WindowEvent::Resized(winit::dpi::LogicalSize { width, height }) => {
                Some(carbide_core::event::input::Input::Resize(width as _, height as _).into())
            },

            winit::WindowEvent::ReceivedCharacter(ch) => {
                let string = match ch {
                    // Ignore control characters and return ascii for Text event (like sdl2).
                    '\u{7f}' | // Delete
                    '\u{1b}' | // Escape
                    '\u{8}'  | // Backspace
                    '\r' | '\n' | '\t' => "".to_string(),
                    _ => ch.to_string()
                };
                Some(carbide_core::event::input::Input::Text(string).into())
            },

            winit::WindowEvent::Focused(focused) =>
                Some(carbide_core::event::input::Input::Focus(focused).into()),

            winit::WindowEvent::KeyboardInput { input, .. } => {
                input.virtual_keycode.map(|key| {
                    match input.state {
                        winit::ElementState::Pressed =>
                            carbide_core::event::input::Input::Press(carbide_core::input::Button::Keyboard(map_key(key))).into(),
                        winit::ElementState::Released =>
                            carbide_core::event::input::Input::Release(carbide_core::input::Button::Keyboard(map_key(key))).into(),
                    }
                })
            },

            winit::WindowEvent::Touch(winit::Touch { phase, location, id, .. }) => {
                let winit::dpi::LogicalPosition { x, y } = location;
                let phase = match phase {
                    winit::TouchPhase::Started => carbide_core::input::touch::Phase::Start,
                    winit::TouchPhase::Moved => carbide_core::input::touch::Phase::Move,
                    winit::TouchPhase::Cancelled => carbide_core::input::touch::Phase::Cancel,
                    winit::TouchPhase::Ended => carbide_core::input::touch::Phase::End,
                };
                let xy = [tx(x), ty(y)];
                let id = carbide_core::input::touch::Id::new(id);
                let touch = carbide_core::input::Touch { phase: phase, id: id, xy: xy };
                Some(carbide_core::event::input::Input::Touch(touch).into())
            }

            winit::WindowEvent::CursorMoved { position, .. } => {
                let winit::dpi::LogicalPosition { x, y } = position;
                let x = tx(x as carbide_core::Scalar);
                let y = ty(y as carbide_core::Scalar);
                let motion = carbide_core::input::Motion::MouseCursor { x: x, y: y };
                Some(carbide_core::event::input::Input::Motion(motion).into())
            },

            winit::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::MouseScrollDelta::PixelDelta(winit::dpi::LogicalPosition { x, y }) => {
                    let x = x as carbide_core::Scalar;
                    let y = -y as carbide_core::Scalar;
                    let motion = carbide_core::input::Motion::Scroll { x: x, y: y };
                    Some(carbide_core::event::input::Input::Motion(motion).into())
                },

                winit::MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    const ARBITRARY_POINTS_PER_LINE_FACTOR: carbide_core::Scalar = 10.0;
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as carbide_core::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as carbide_core::Scalar;
                    Some(carbide_core::event::input::Input::Motion(carbide_core::input::Motion::Scroll { x: x, y: y }).into())
                },
            },

            winit::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::ElementState::Pressed =>
                    Some(carbide_core::event::input::Input::Press(carbide_core::input::Button::Mouse(map_mouse(button))).into()),
                winit::ElementState::Released =>
                    Some(carbide_core::event::input::Input::Release(carbide_core::input::Button::Mouse(map_mouse(button))).into()),
            },

            winit::WindowEvent::Refresh => {
                Some(carbide_core::event::input::Input::Redraw)
            },

            _ => None,
        }
    }};
}

/// A macro for converting a `winit::Event` to a `carbide_core::event::Input`.
///
/// Expects a `winit::Event` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<carbide_core::event::Input>`.
///
/// Invocations of this macro require that a version of the `winit` and `carbide_core` crates are
/// available in the crate root.
#[macro_export]
macro_rules! convert_event {
    ($event:expr, $window:expr) => {{
        match $event {
            winit::Event::WindowEvent { event, .. } => $crate::convert_window_event!(event, $window),
            _ => None,
        }
    }};
}

/// Convert a given carbide mouse cursor to the corresponding winit cursor type.
///
/// Expects a `carbide_core::cursor::MouseCursor`, returns a `winit::MouseCursor`.
///
/// Requires that both the `carbide_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! convert_mouse_cursor {
    ($cursor:expr) => {{
        match $cursor {
            carbide_core::cursor::MouseCursor::Text => winit::MouseCursor::Text,
            carbide_core::cursor::MouseCursor::VerticalText => winit::MouseCursor::VerticalText,
            carbide_core::cursor::MouseCursor::Hand => winit::MouseCursor::Hand,
            carbide_core::cursor::MouseCursor::Grab => winit::MouseCursor::Grab,
            carbide_core::cursor::MouseCursor::Grabbing => winit::MouseCursor::Grabbing,
            carbide_core::cursor::MouseCursor::ResizeVertical => winit::MouseCursor::NsResize,
            carbide_core::cursor::MouseCursor::ResizeHorizontal => winit::MouseCursor::EwResize,
            carbide_core::cursor::MouseCursor::ResizeTopLeftBottomRight => winit::MouseCursor::NwseResize,
            carbide_core::cursor::MouseCursor::ResizeTopRightBottomLeft => winit::MouseCursor::NeswResize,
            _ => winit::MouseCursor::Arrow,
        }
    }};
}

/// Generate a set of conversion functions for converting between render of the crate's versions of
/// `winit` and `carbide_core`.
#[macro_export]
macro_rules! conversion_fns {
    () => {
        /// Convert a `winit::VirtualKeyCode` to a `carbide_core::input::keyboard::Key`.
        pub fn convert_key(keycode: winit::VirtualKeyCode) -> carbide_core::input::keyboard::Key {
            $crate::convert_key!(keycode)
        }

        /// Convert a `winit::MouseButton` to a `carbide_core::input::MouseButton`.
        pub fn convert_mouse_button(
            mouse_button: winit::MouseButton,
        ) -> carbide_core::input::MouseButton {
            $crate::convert_mouse_button!(mouse_button)
        }

        /// Convert a given carbide mouse cursor to the corresponding winit cursor type.
        pub fn convert_mouse_cursor(
            cursor: carbide_core::cursor::MouseCursor,
        ) -> winit::MouseCursor {
            $crate::convert_mouse_cursor!(cursor)
        }

        /// A function for converting a `winit::WindowEvent` to a `carbide_core::event::Input`.
        pub fn convert_window_event<W>(
            event: winit::WindowEvent,
            window: &W,
        ) -> Option<carbide_core::event::input::Input>
        where
            W: $crate::WinitWindow,
        {
            $crate::convert_window_event!(event, window)
        }

        /// A function for converting a `winit::Event` to a `carbide_core::event::Input`.
        pub fn convert_event<W>(
            event: winit::Event,
            window: &W,
        ) -> Option<carbide_core::event::input::Input>
        where
            W: $crate::WinitWindow,
        {
            $crate::convert_event!(event, window)
        }
    };
}
