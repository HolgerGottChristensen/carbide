#[macro_export]
macro_rules! v023_convert_key {
    ($keycode:expr) => {{
        match $keycode {
            winit::event::VirtualKeyCode::Key0 => carbide_core::event::Key::D0,
            winit::event::VirtualKeyCode::Key1 => carbide_core::event::Key::D1,
            winit::event::VirtualKeyCode::Key2 => carbide_core::event::Key::D2,
            winit::event::VirtualKeyCode::Key3 => carbide_core::event::Key::D3,
            winit::event::VirtualKeyCode::Key4 => carbide_core::event::Key::D4,
            winit::event::VirtualKeyCode::Key5 => carbide_core::event::Key::D5,
            winit::event::VirtualKeyCode::Key6 => carbide_core::event::Key::D6,
            winit::event::VirtualKeyCode::Key7 => carbide_core::event::Key::D7,
            winit::event::VirtualKeyCode::Key8 => carbide_core::event::Key::D8,
            winit::event::VirtualKeyCode::Key9 => carbide_core::event::Key::D9,
            winit::event::VirtualKeyCode::A => carbide_core::event::Key::A,
            winit::event::VirtualKeyCode::B => carbide_core::event::Key::B,
            winit::event::VirtualKeyCode::C => carbide_core::event::Key::C,
            winit::event::VirtualKeyCode::D => carbide_core::event::Key::D,
            winit::event::VirtualKeyCode::E => carbide_core::event::Key::E,
            winit::event::VirtualKeyCode::F => carbide_core::event::Key::F,
            winit::event::VirtualKeyCode::G => carbide_core::event::Key::G,
            winit::event::VirtualKeyCode::H => carbide_core::event::Key::H,
            winit::event::VirtualKeyCode::I => carbide_core::event::Key::I,
            winit::event::VirtualKeyCode::J => carbide_core::event::Key::J,
            winit::event::VirtualKeyCode::K => carbide_core::event::Key::K,
            winit::event::VirtualKeyCode::L => carbide_core::event::Key::L,
            winit::event::VirtualKeyCode::M => carbide_core::event::Key::M,
            winit::event::VirtualKeyCode::N => carbide_core::event::Key::N,
            winit::event::VirtualKeyCode::O => carbide_core::event::Key::O,
            winit::event::VirtualKeyCode::P => carbide_core::event::Key::P,
            winit::event::VirtualKeyCode::Q => carbide_core::event::Key::Q,
            winit::event::VirtualKeyCode::R => carbide_core::event::Key::R,
            winit::event::VirtualKeyCode::S => carbide_core::event::Key::S,
            winit::event::VirtualKeyCode::T => carbide_core::event::Key::T,
            winit::event::VirtualKeyCode::U => carbide_core::event::Key::U,
            winit::event::VirtualKeyCode::V => carbide_core::event::Key::V,
            winit::event::VirtualKeyCode::W => carbide_core::event::Key::W,
            winit::event::VirtualKeyCode::X => carbide_core::event::Key::X,
            winit::event::VirtualKeyCode::Y => carbide_core::event::Key::Y,
            winit::event::VirtualKeyCode::Z => carbide_core::event::Key::Z,
            winit::event::VirtualKeyCode::Apostrophe => carbide_core::event::Key::Unknown,
            winit::event::VirtualKeyCode::Backslash => carbide_core::event::Key::Backslash,
            winit::event::VirtualKeyCode::Back => carbide_core::event::Key::Backspace,
            winit::event::VirtualKeyCode::Delete => carbide_core::event::Key::Delete,
            winit::event::VirtualKeyCode::Comma => carbide_core::event::Key::Comma,
            winit::event::VirtualKeyCode::Down => carbide_core::event::Key::Down,
            winit::event::VirtualKeyCode::End => carbide_core::event::Key::End,
            winit::event::VirtualKeyCode::Return => carbide_core::event::Key::Return,
            winit::event::VirtualKeyCode::Equals => carbide_core::event::Key::Equals,
            winit::event::VirtualKeyCode::Escape => carbide_core::event::Key::Escape,
            winit::event::VirtualKeyCode::F1 => carbide_core::event::Key::F1,
            winit::event::VirtualKeyCode::F2 => carbide_core::event::Key::F2,
            winit::event::VirtualKeyCode::F3 => carbide_core::event::Key::F3,
            winit::event::VirtualKeyCode::F4 => carbide_core::event::Key::F4,
            winit::event::VirtualKeyCode::F5 => carbide_core::event::Key::F5,
            winit::event::VirtualKeyCode::F6 => carbide_core::event::Key::F6,
            winit::event::VirtualKeyCode::F7 => carbide_core::event::Key::F7,
            winit::event::VirtualKeyCode::F8 => carbide_core::event::Key::F8,
            winit::event::VirtualKeyCode::F9 => carbide_core::event::Key::F9,
            winit::event::VirtualKeyCode::F10 => carbide_core::event::Key::F10,
            winit::event::VirtualKeyCode::F11 => carbide_core::event::Key::F11,
            winit::event::VirtualKeyCode::F12 => carbide_core::event::Key::F12,
            winit::event::VirtualKeyCode::F13 => carbide_core::event::Key::F13,
            winit::event::VirtualKeyCode::F14 => carbide_core::event::Key::F14,
            winit::event::VirtualKeyCode::F15 => carbide_core::event::Key::F15,
            winit::event::VirtualKeyCode::Numpad0 => carbide_core::event::Key::NumPad0,
            winit::event::VirtualKeyCode::Numpad1 => carbide_core::event::Key::NumPad1,
            winit::event::VirtualKeyCode::Numpad2 => carbide_core::event::Key::NumPad2,
            winit::event::VirtualKeyCode::Numpad3 => carbide_core::event::Key::NumPad3,
            winit::event::VirtualKeyCode::Numpad4 => carbide_core::event::Key::NumPad4,
            winit::event::VirtualKeyCode::Numpad5 => carbide_core::event::Key::NumPad5,
            winit::event::VirtualKeyCode::Numpad6 => carbide_core::event::Key::NumPad6,
            winit::event::VirtualKeyCode::Numpad7 => carbide_core::event::Key::NumPad7,
            winit::event::VirtualKeyCode::Numpad8 => carbide_core::event::Key::NumPad8,
            winit::event::VirtualKeyCode::Numpad9 => carbide_core::event::Key::NumPad9,
            winit::event::VirtualKeyCode::NumpadComma
            | winit::event::VirtualKeyCode::NumpadDecimal => {
                carbide_core::event::Key::NumPadDecimal
            }
            winit::event::VirtualKeyCode::NumpadDivide => carbide_core::event::Key::NumPadDivide,
            winit::event::VirtualKeyCode::NumpadMultiply => {
                carbide_core::event::Key::NumPadMultiply
            }
            winit::event::VirtualKeyCode::NumpadSubtract => carbide_core::event::Key::NumPadMinus,
            winit::event::VirtualKeyCode::NumpadAdd => carbide_core::event::Key::NumPadPlus,
            winit::event::VirtualKeyCode::NumpadEnter => carbide_core::event::Key::NumPadEnter,
            winit::event::VirtualKeyCode::NumpadEquals => carbide_core::event::Key::NumPadEquals,
            winit::event::VirtualKeyCode::LShift => carbide_core::event::Key::LShift,
            winit::event::VirtualKeyCode::LControl => carbide_core::event::Key::LCtrl,
            winit::event::VirtualKeyCode::LAlt => carbide_core::event::Key::LAlt,
            winit::event::VirtualKeyCode::RShift => carbide_core::event::Key::RShift,
            winit::event::VirtualKeyCode::RControl => carbide_core::event::Key::RCtrl,
            winit::event::VirtualKeyCode::RAlt => carbide_core::event::Key::RAlt,
            winit::event::VirtualKeyCode::Home => carbide_core::event::Key::Home,
            winit::event::VirtualKeyCode::Insert => carbide_core::event::Key::Insert,
            winit::event::VirtualKeyCode::Left => carbide_core::event::Key::Left,

            // On mac this is the command (cmd) key
            winit::event::VirtualKeyCode::LWin => carbide_core::event::Key::LGui,
            winit::event::VirtualKeyCode::RWin => carbide_core::event::Key::RGui,

            winit::event::VirtualKeyCode::LBracket => carbide_core::event::Key::LeftBracket,
            winit::event::VirtualKeyCode::Minus => carbide_core::event::Key::Minus,
            winit::event::VirtualKeyCode::Numlock => carbide_core::event::Key::NumLockClear,
            winit::event::VirtualKeyCode::PageDown => carbide_core::event::Key::PageDown,
            winit::event::VirtualKeyCode::PageUp => carbide_core::event::Key::PageUp,
            winit::event::VirtualKeyCode::Pause => carbide_core::event::Key::Pause,
            winit::event::VirtualKeyCode::Period => carbide_core::event::Key::Period,
            winit::event::VirtualKeyCode::Right => carbide_core::event::Key::Right,
            winit::event::VirtualKeyCode::RBracket => carbide_core::event::Key::RightBracket,
            winit::event::VirtualKeyCode::Semicolon => carbide_core::event::Key::Semicolon,
            winit::event::VirtualKeyCode::Slash => carbide_core::event::Key::Slash,
            winit::event::VirtualKeyCode::Space => carbide_core::event::Key::Space,
            winit::event::VirtualKeyCode::Tab => carbide_core::event::Key::Tab,
            winit::event::VirtualKeyCode::Up => carbide_core::event::Key::Up,
            _ => carbide_core::event::Key::Unknown,
        }
    }};
}

/// Maps winit's mouse button to carbide's mouse button.
///
/// Expects a `winit::MouseButton` as input and returns a `carbide_core::event::MouseButton` as
/// output.
///
/// Requires that both the `carbide_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! v023_convert_mouse_button {
    ($mouse_button:expr) => {{
        match $mouse_button {
            winit::event::MouseButton::Left => carbide_core::event::MouseButton::Left,
            winit::event::MouseButton::Right => carbide_core::event::MouseButton::Right,
            winit::event::MouseButton::Middle => carbide_core::event::MouseButton::Middle,
            winit::event::MouseButton::Other(0) => carbide_core::event::MouseButton::X1,
            winit::event::MouseButton::Other(1) => carbide_core::event::MouseButton::X2,
            winit::event::MouseButton::Other(2) => carbide_core::event::MouseButton::Button6,
            winit::event::MouseButton::Other(3) => carbide_core::event::MouseButton::Button7,
            winit::event::MouseButton::Other(4) => carbide_core::event::MouseButton::Button8,
            _ => carbide_core::event::MouseButton::Unknown,
        }
    }};
}

/// A macro for converting a `winit::WindowEvent` to a `Option<carbide_core::event::Input>`.
///
/// Expects a `winit::WindowEvent` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<carbide_core::event::Input>`.
#[macro_export]
macro_rules! v023_convert_window_event {
    ($event:expr, $window:expr) => {{
        // The window size in points.
        let scale_factor: f64 = $window.scale_factor();
        let (win_w, win_h): (f64, f64) = $window.inner_size().to_logical::<f64>(scale_factor).into();

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        let tx = |x: carbide_core::Scalar| x - win_w / 2.0;
        let ty = |y: carbide_core::Scalar| -(y - win_h / 2.0);

        // Functions for converting keys and mouse buttons.
        let map_key = |key: winit::event::VirtualKeyCode| $crate::v023_convert_key!(key);
        let map_mouse = |button: winit::event::MouseButton| $crate::v023_convert_mouse_button!(button);

        match $event {
            winit::event::WindowEvent::Resized(physical_size) => {
                let winit::dpi::LogicalSize { width, height } = physical_size.to_logical(scale_factor);
                Some(carbide_core::event::Input::Resize(width, height).into())
            },

            winit::event::WindowEvent::ReceivedCharacter(ch) => {
                let string = match ch {
                    // Ignore control characters and return ascii for Text event (like sdl2).
                    '\u{7f}' | // Delete
                    '\u{1b}' | // Escape
                    '\u{8}'  | // Backspace
                    '\r' | '\n' | '\t' => "".to_string(),
                    _ => ch.to_string()
                };
                Some(carbide_core::event::Input::Text(string).into())
            },

            winit::event::WindowEvent::Focused(focused) =>
                Some(carbide_core::event::Input::Focus(focused.clone()).into()),

            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                input.virtual_keycode.map(|key| {
                    match input.state {
                        winit::event::ElementState::Pressed =>
                            carbide_core::event::Input::Press(carbide_core::event::Button::Keyboard(map_key(key))).into(),
                        winit::event::ElementState::Released =>
                            carbide_core::event::Input::Release(carbide_core::event::Button::Keyboard(map_key(key))).into(),
                    }
                })
            },

            winit::event::WindowEvent::Touch(winit::event::Touch { phase, location, id, .. }) => {
                let winit::dpi::LogicalPosition { x, y } = location.to_logical::<f64>(scale_factor);
                let phase = match phase {
                    winit::event::TouchPhase::Started => carbide_core::event::TouchPhase::Start,
                    winit::event::TouchPhase::Moved => carbide_core::event::TouchPhase::Move,
                    winit::event::TouchPhase::Cancelled => carbide_core::event::TouchPhase::Cancel,
                    winit::event::TouchPhase::Ended => carbide_core::event::TouchPhase::End,
                };
                let id = carbide_core::event::TouchId::new(id.clone());
                let touch = carbide_core::event::Touch { phase: phase, id: id, position: carbide_core::draw::Position::new(tx(x), ty(y)) };
                Some(carbide_core::event::Input::Touch(touch).into())
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let winit::dpi::LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor);
                let x = tx(x as carbide_core::Scalar);
                let y = ty(y as carbide_core::Scalar);
                let motion = carbide_core::event::Motion::MouseCursor { x: x, y: y };
                Some(carbide_core::event::Input::Motion(motion).into())
            },

            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                    let winit::dpi::LogicalPosition { x, y } = delta.to_logical::<f64>(scale_factor);
                    let x = x as carbide_core::Scalar;
                    let y = -y as carbide_core::Scalar;
                    let motion = carbide_core::event::Motion::Scroll { x: x, y: y };
                    Some(carbide_core::event::Input::Motion(motion).into())
                },

                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    const ARBITRARY_POINTS_PER_LINE_FACTOR: carbide_core::Scalar = 10.0;
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x.clone() as carbide_core::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y.clone() as carbide_core::Scalar;
                    Some(carbide_core::event::Input::Motion(carbide_core::event::Motion::Scroll { x: x, y: y }).into())
                },
            },

            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed =>
                    Some(carbide_core::event::Input::Press(carbide_core::event::Button::Mouse(map_mouse(button.clone()))).into()),
                winit::event::ElementState::Released =>
                    Some(carbide_core::event::Input::Release(carbide_core::event::Button::Mouse(map_mouse(button.clone()))).into()),
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
macro_rules! v023_convert_event {
    ($event:expr, $window:expr) => {{
        match $event {
            winit::event::Event::WindowEvent { event, .. } => {
                $crate::v023_convert_window_event!(event, $window)
            }
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
macro_rules! v023_convert_mouse_cursor {
    ($cursor:expr) => {{
        match $cursor {
            carbide_core::cursor::MouseCursor::Default => winit::window::CursorIcon::Arrow,
            carbide_core::cursor::MouseCursor::Crosshair => winit::window::CursorIcon::Crosshair,
            carbide_core::cursor::MouseCursor::Hand => winit::window::CursorIcon::Hand,
            carbide_core::cursor::MouseCursor::Arrow => winit::window::CursorIcon::Arrow,
            carbide_core::cursor::MouseCursor::Move => winit::window::CursorIcon::Move,
            carbide_core::cursor::MouseCursor::Text => winit::window::CursorIcon::Text,
            carbide_core::cursor::MouseCursor::Wait => winit::window::CursorIcon::Wait,
            carbide_core::cursor::MouseCursor::Help => winit::window::CursorIcon::Help,
            carbide_core::cursor::MouseCursor::Progress => winit::window::CursorIcon::Progress,
            carbide_core::cursor::MouseCursor::NotAllowed => winit::window::CursorIcon::NotAllowed,
            carbide_core::cursor::MouseCursor::ContextMenu => {
                winit::window::CursorIcon::ContextMenu
            }
            carbide_core::cursor::MouseCursor::Cell => winit::window::CursorIcon::Cell,
            carbide_core::cursor::MouseCursor::VerticalText => {
                winit::window::CursorIcon::VerticalText
            }
            carbide_core::cursor::MouseCursor::Alias => winit::window::CursorIcon::Alias,
            carbide_core::cursor::MouseCursor::Copy => winit::window::CursorIcon::Copy,
            carbide_core::cursor::MouseCursor::NoDrop => winit::window::CursorIcon::NoDrop,
            carbide_core::cursor::MouseCursor::Grab => winit::window::CursorIcon::Grab,
            carbide_core::cursor::MouseCursor::Grabbing => winit::window::CursorIcon::Grabbing,
            carbide_core::cursor::MouseCursor::AllScroll => winit::window::CursorIcon::AllScroll,
            carbide_core::cursor::MouseCursor::ZoomIn => winit::window::CursorIcon::ZoomIn,
            carbide_core::cursor::MouseCursor::ZoomOut => winit::window::CursorIcon::ZoomOut,
            carbide_core::cursor::MouseCursor::EResize => winit::window::CursorIcon::EResize,
            carbide_core::cursor::MouseCursor::NResize => winit::window::CursorIcon::NResize,
            carbide_core::cursor::MouseCursor::NeResize => winit::window::CursorIcon::NeResize,
            carbide_core::cursor::MouseCursor::NwResize => winit::window::CursorIcon::NwResize,
            carbide_core::cursor::MouseCursor::SResize => winit::window::CursorIcon::SResize,
            carbide_core::cursor::MouseCursor::SeResize => winit::window::CursorIcon::SeResize,
            carbide_core::cursor::MouseCursor::SwResize => winit::window::CursorIcon::SwResize,
            carbide_core::cursor::MouseCursor::WResize => winit::window::CursorIcon::WResize,
            carbide_core::cursor::MouseCursor::EwResize => winit::window::CursorIcon::EwResize,
            carbide_core::cursor::MouseCursor::NsResize => winit::window::CursorIcon::NsResize,
            carbide_core::cursor::MouseCursor::NeswResize => winit::window::CursorIcon::NeswResize,
            carbide_core::cursor::MouseCursor::NwseResize => winit::window::CursorIcon::NwseResize,
            carbide_core::cursor::MouseCursor::ColResize => winit::window::CursorIcon::ColResize,
            carbide_core::cursor::MouseCursor::RowResize => winit::window::CursorIcon::RowResize,
            _ => winit::window::CursorIcon::Arrow,
        }
    }};
}

#[macro_export]
macro_rules! v023_conversion_fns {
    () => {
        /// Generate a set of conversion functions for converting between types of the crate's versions of
        /// `winit` and `carbide_core`.
        /// Maps winit's key to a carbide `Key`.
        ///
        /// Expects a `winit::VirtualKeyCode` as input and returns a `carbide_core::event::Key`.
        ///
        /// Requires that both the `winit` and `carbide_core` crates exist within the crate root.
        pub fn convert_key(keycode: winit::event::VirtualKeyCode) -> carbide_core::event::Key {
            $crate::v023_convert_key!(keycode)
        }

        /// Convert a `winit::MouseButton` to a `carbide_core::event::MouseButton`.
        pub fn convert_mouse_button(
            mouse_button: winit::event::MouseButton,
        ) -> carbide_core::event::MouseButton {
            $crate::v023_convert_mouse_button!(mouse_button)
        }

        /// Convert a given carbide mouse cursor to the corresponding winit cursor type.
        pub fn convert_mouse_cursor(
            cursor: carbide_core::cursor::MouseCursor,
        ) -> winit::window::CursorIcon {
            $crate::v023_convert_mouse_cursor!(cursor)
        }

        /// A function for converting a `winit::WindowEvent` to a `carbide_core::event::Input`.
        pub fn convert_window_event(
            event: &winit::event::WindowEvent,
            window: &winit::window::Window,
        ) -> Option<carbide_core::event::Input> {
            $crate::v023_convert_window_event!(event, window)
        }

        /// A function for converting a `winit::Event` to a `carbide_core::event::Input`.
        pub fn convert_event<T>(
            event: &winit::event::Event<T>,
            window: &winit::window::Window,
        ) -> Option<carbide_core::event::Input> {
            $crate::v023_convert_event!(event, window)
        }
    };
}
