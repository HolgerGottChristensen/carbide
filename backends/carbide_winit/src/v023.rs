
#[macro_export]
macro_rules! v023_convert_key {
    ($keycode:expr) => {{
        match $keycode {
            winit::event::VirtualKeyCode::Key0 => carbide_core::input::keyboard::Key::D0,
            winit::event::VirtualKeyCode::Key1 => carbide_core::input::keyboard::Key::D1,
            winit::event::VirtualKeyCode::Key2 => carbide_core::input::keyboard::Key::D2,
            winit::event::VirtualKeyCode::Key3 => carbide_core::input::keyboard::Key::D3,
            winit::event::VirtualKeyCode::Key4 => carbide_core::input::keyboard::Key::D4,
            winit::event::VirtualKeyCode::Key5 => carbide_core::input::keyboard::Key::D5,
            winit::event::VirtualKeyCode::Key6 => carbide_core::input::keyboard::Key::D6,
            winit::event::VirtualKeyCode::Key7 => carbide_core::input::keyboard::Key::D7,
            winit::event::VirtualKeyCode::Key8 => carbide_core::input::keyboard::Key::D8,
            winit::event::VirtualKeyCode::Key9 => carbide_core::input::keyboard::Key::D9,
            winit::event::VirtualKeyCode::A => carbide_core::input::keyboard::Key::A,
            winit::event::VirtualKeyCode::B => carbide_core::input::keyboard::Key::B,
            winit::event::VirtualKeyCode::C => carbide_core::input::keyboard::Key::C,
            winit::event::VirtualKeyCode::D => carbide_core::input::keyboard::Key::D,
            winit::event::VirtualKeyCode::E => carbide_core::input::keyboard::Key::E,
            winit::event::VirtualKeyCode::F => carbide_core::input::keyboard::Key::F,
            winit::event::VirtualKeyCode::G => carbide_core::input::keyboard::Key::G,
            winit::event::VirtualKeyCode::H => carbide_core::input::keyboard::Key::H,
            winit::event::VirtualKeyCode::I => carbide_core::input::keyboard::Key::I,
            winit::event::VirtualKeyCode::J => carbide_core::input::keyboard::Key::J,
            winit::event::VirtualKeyCode::K => carbide_core::input::keyboard::Key::K,
            winit::event::VirtualKeyCode::L => carbide_core::input::keyboard::Key::L,
            winit::event::VirtualKeyCode::M => carbide_core::input::keyboard::Key::M,
            winit::event::VirtualKeyCode::N => carbide_core::input::keyboard::Key::N,
            winit::event::VirtualKeyCode::O => carbide_core::input::keyboard::Key::O,
            winit::event::VirtualKeyCode::P => carbide_core::input::keyboard::Key::P,
            winit::event::VirtualKeyCode::Q => carbide_core::input::keyboard::Key::Q,
            winit::event::VirtualKeyCode::R => carbide_core::input::keyboard::Key::R,
            winit::event::VirtualKeyCode::S => carbide_core::input::keyboard::Key::S,
            winit::event::VirtualKeyCode::T => carbide_core::input::keyboard::Key::T,
            winit::event::VirtualKeyCode::U => carbide_core::input::keyboard::Key::U,
            winit::event::VirtualKeyCode::V => carbide_core::input::keyboard::Key::V,
            winit::event::VirtualKeyCode::W => carbide_core::input::keyboard::Key::W,
            winit::event::VirtualKeyCode::X => carbide_core::input::keyboard::Key::X,
            winit::event::VirtualKeyCode::Y => carbide_core::input::keyboard::Key::Y,
            winit::event::VirtualKeyCode::Z => carbide_core::input::keyboard::Key::Z,
            winit::event::VirtualKeyCode::Apostrophe => carbide_core::input::keyboard::Key::Unknown,
            winit::event::VirtualKeyCode::Backslash => carbide_core::input::keyboard::Key::Backslash,
            winit::event::VirtualKeyCode::Back => carbide_core::input::keyboard::Key::Backspace,
            // K::CapsLock => Key::CapsLock,
            winit::event::VirtualKeyCode::Delete => carbide_core::input::keyboard::Key::Delete,
            winit::event::VirtualKeyCode::Comma => carbide_core::input::keyboard::Key::Comma,
            winit::event::VirtualKeyCode::Down => carbide_core::input::keyboard::Key::Down,
            winit::event::VirtualKeyCode::End => carbide_core::input::keyboard::Key::End,
            winit::event::VirtualKeyCode::Return => carbide_core::input::keyboard::Key::Return,
            winit::event::VirtualKeyCode::Equals => carbide_core::input::keyboard::Key::Equals,
            winit::event::VirtualKeyCode::Escape => carbide_core::input::keyboard::Key::Escape,
            winit::event::VirtualKeyCode::F1 => carbide_core::input::keyboard::Key::F1,
            winit::event::VirtualKeyCode::F2 => carbide_core::input::keyboard::Key::F2,
            winit::event::VirtualKeyCode::F3 => carbide_core::input::keyboard::Key::F3,
            winit::event::VirtualKeyCode::F4 => carbide_core::input::keyboard::Key::F4,
            winit::event::VirtualKeyCode::F5 => carbide_core::input::keyboard::Key::F5,
            winit::event::VirtualKeyCode::F6 => carbide_core::input::keyboard::Key::F6,
            winit::event::VirtualKeyCode::F7 => carbide_core::input::keyboard::Key::F7,
            winit::event::VirtualKeyCode::F8 => carbide_core::input::keyboard::Key::F8,
            winit::event::VirtualKeyCode::F9 => carbide_core::input::keyboard::Key::F9,
            winit::event::VirtualKeyCode::F10 => carbide_core::input::keyboard::Key::F10,
            winit::event::VirtualKeyCode::F11 => carbide_core::input::keyboard::Key::F11,
            winit::event::VirtualKeyCode::F12 => carbide_core::input::keyboard::Key::F12,
            winit::event::VirtualKeyCode::F13 => carbide_core::input::keyboard::Key::F13,
            winit::event::VirtualKeyCode::F14 => carbide_core::input::keyboard::Key::F14,
            winit::event::VirtualKeyCode::F15 => carbide_core::input::keyboard::Key::F15,
            winit::event::VirtualKeyCode::Numpad0 => carbide_core::input::keyboard::Key::NumPad0,
            winit::event::VirtualKeyCode::Numpad1 => carbide_core::input::keyboard::Key::NumPad1,
            winit::event::VirtualKeyCode::Numpad2 => carbide_core::input::keyboard::Key::NumPad2,
            winit::event::VirtualKeyCode::Numpad3 => carbide_core::input::keyboard::Key::NumPad3,
            winit::event::VirtualKeyCode::Numpad4 => carbide_core::input::keyboard::Key::NumPad4,
            winit::event::VirtualKeyCode::Numpad5 => carbide_core::input::keyboard::Key::NumPad5,
            winit::event::VirtualKeyCode::Numpad6 => carbide_core::input::keyboard::Key::NumPad6,
            winit::event::VirtualKeyCode::Numpad7 => carbide_core::input::keyboard::Key::NumPad7,
            winit::event::VirtualKeyCode::Numpad8 => carbide_core::input::keyboard::Key::NumPad8,
            winit::event::VirtualKeyCode::Numpad9 => carbide_core::input::keyboard::Key::NumPad9,
            winit::event::VirtualKeyCode::NumpadComma
            | winit::event::VirtualKeyCode::NumpadDecimal => {
                carbide_core::input::keyboard::Key::NumPadDecimal
            }
            winit::event::VirtualKeyCode::NumpadDivide => {
                carbide_core::input::keyboard::Key::NumPadDivide
            }
            winit::event::VirtualKeyCode::NumpadMultiply => {
                carbide_core::input::keyboard::Key::NumPadMultiply
            }
            winit::event::VirtualKeyCode::NumpadSubtract => {
                carbide_core::input::keyboard::Key::NumPadMinus
            }
            winit::event::VirtualKeyCode::NumpadAdd => {
                carbide_core::input::keyboard::Key::NumPadPlus
            }
            winit::event::VirtualKeyCode::NumpadEnter => {
                carbide_core::input::keyboard::Key::NumPadEnter
            }
            winit::event::VirtualKeyCode::NumpadEquals => {
                carbide_core::input::keyboard::Key::NumPadEquals
            }
            winit::event::VirtualKeyCode::LShift => carbide_core::input::keyboard::Key::LShift,
            winit::event::VirtualKeyCode::LControl => carbide_core::input::keyboard::Key::LCtrl,
            winit::event::VirtualKeyCode::LAlt => carbide_core::input::keyboard::Key::LAlt,
            winit::event::VirtualKeyCode::RShift => carbide_core::input::keyboard::Key::RShift,
            winit::event::VirtualKeyCode::RControl => carbide_core::input::keyboard::Key::RCtrl,
            winit::event::VirtualKeyCode::RAlt => carbide_core::input::keyboard::Key::RAlt,
            winit::event::VirtualKeyCode::Home => carbide_core::input::keyboard::Key::Home,
            winit::event::VirtualKeyCode::Insert => carbide_core::input::keyboard::Key::Insert,
            winit::event::VirtualKeyCode::Left => carbide_core::input::keyboard::Key::Left,

            // On mac this is the command (cmd) key
            winit::event::VirtualKeyCode::LWin => carbide_core::input::keyboard::Key::LGui,
            winit::event::VirtualKeyCode::RWin => carbide_core::input::keyboard::Key::RGui,

            winit::event::VirtualKeyCode::LBracket => {
                carbide_core::input::keyboard::Key::LeftBracket
            }
            winit::event::VirtualKeyCode::Minus => carbide_core::input::keyboard::Key::Minus,
            winit::event::VirtualKeyCode::Numlock => {
                carbide_core::input::keyboard::Key::NumLockClear
            }
            winit::event::VirtualKeyCode::PageDown => carbide_core::input::keyboard::Key::PageDown,
            winit::event::VirtualKeyCode::PageUp => carbide_core::input::keyboard::Key::PageUp,
            winit::event::VirtualKeyCode::Pause => carbide_core::input::keyboard::Key::Pause,
            winit::event::VirtualKeyCode::Period => carbide_core::input::keyboard::Key::Period,
            winit::event::VirtualKeyCode::Right => carbide_core::input::keyboard::Key::Right,
            winit::event::VirtualKeyCode::RBracket => {
                carbide_core::input::keyboard::Key::RightBracket
            }
            winit::event::VirtualKeyCode::Semicolon => carbide_core::input::keyboard::Key::Semicolon,
            winit::event::VirtualKeyCode::Slash => carbide_core::input::keyboard::Key::Slash,
            winit::event::VirtualKeyCode::Space => carbide_core::input::keyboard::Key::Space,
            winit::event::VirtualKeyCode::Tab => carbide_core::input::keyboard::Key::Tab,
            winit::event::VirtualKeyCode::Up => carbide_core::input::keyboard::Key::Up,
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
macro_rules! v023_convert_mouse_button {
    ($mouse_button:expr) => {{
        $crate::v021_convert_mouse_button!($mouse_button)
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
                Some(carbide_core::event::input::Input::Resize(width, height).into())
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
                Some(carbide_core::event::input::Input::Text(string).into())
            },

            winit::event::WindowEvent::Focused(focused) =>
                Some(carbide_core::event::input::Input::Focus(focused.clone()).into()),

            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                input.virtual_keycode.map(|key| {
                    match input.state {
                        winit::event::ElementState::Pressed =>
                            carbide_core::event::input::Input::Press(carbide_core::input::Button::Keyboard(map_key(key))).into(),
                        winit::event::ElementState::Released =>
                            carbide_core::event::input::Input::Release(carbide_core::input::Button::Keyboard(map_key(key))).into(),
                    }
                })
            },

            winit::event::WindowEvent::Touch(winit::event::Touch { phase, location, id, .. }) => {
                let winit::dpi::LogicalPosition { x, y } = location.to_logical::<f64>(scale_factor);
                let phase = match phase {
                    winit::event::TouchPhase::Started => carbide_core::input::touch::Phase::Start,
                    winit::event::TouchPhase::Moved => carbide_core::input::touch::Phase::Move,
                    winit::event::TouchPhase::Cancelled => carbide_core::input::touch::Phase::Cancel,
                    winit::event::TouchPhase::Ended => carbide_core::input::touch::Phase::End,
                };
                let xy = [tx(x), ty(y)];
                let id = carbide_core::input::touch::Id::new(id.clone());
                let touch = carbide_core::input::Touch { phase: phase, id: id, xy: xy };
                Some(carbide_core::event::input::Input::Touch(touch).into())
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let winit::dpi::LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor);
                let x = tx(x as carbide_core::Scalar);
                let y = ty(y as carbide_core::Scalar);
                let motion = carbide_core::input::Motion::MouseCursor { x: x, y: y };
                Some(carbide_core::event::input::Input::Motion(motion).into())
            },

            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                    let winit::dpi::LogicalPosition { x, y } = delta.to_logical::<f64>(scale_factor);
                    let x = x as carbide_core::Scalar;
                    let y = -y as carbide_core::Scalar;
                    let motion = carbide_core::input::Motion::Scroll { x: x, y: y };
                    Some(carbide_core::event::input::Input::Motion(motion).into())
                },

                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    const ARBITRARY_POINTS_PER_LINE_FACTOR: carbide_core::Scalar = 10.0;
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x.clone() as carbide_core::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y.clone() as carbide_core::Scalar;
                    Some(carbide_core::event::input::Input::Motion(carbide_core::input::Motion::Scroll { x: x, y: y }).into())
                },
            },

            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed =>
                    Some(carbide_core::event::input::Input::Press(carbide_core::input::Button::Mouse(map_mouse(button.clone()))).into()),
                winit::event::ElementState::Released =>
                    Some(carbide_core::event::input::Input::Release(carbide_core::input::Button::Mouse(map_mouse(button.clone()))).into()),
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
        $crate::v021_convert_mouse_cursor!($cursor)
    }};
}

#[macro_export]
macro_rules! v023_conversion_fns {
    () => {
        /// Generate a set of conversion functions for converting between types of the crate's versions of
        /// `winit` and `carbide_core`.
        /// Maps winit's key to a carbide `Key`.
        ///
        /// Expects a `winit::VirtualKeyCode` as input and returns a `carbide_core::input::keyboard::Key`.
        ///
        /// Requires that both the `winit` and `carbide_core` crates exist within the crate root.
        pub fn convert_key(
            keycode: winit::event::VirtualKeyCode,
        ) -> carbide_core::input::keyboard::Key {
            $crate::v023_convert_key!(keycode)
        }

        /// Convert a `winit::MouseButton` to a `carbide_core::input::MouseButton`.
        pub fn convert_mouse_button(
            mouse_button: winit::event::MouseButton,
        ) -> carbide_core::input::MouseButton {
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
        ) -> Option<carbide_core::event::input::Input> {
            $crate::v023_convert_window_event!(event, window)
        }

        /// A function for converting a `winit::Event` to a `carbide_core::event::Input`.
        pub fn convert_event<T>(
            event: &winit::event::Event<T>,
            window: &winit::window::Window,
        ) -> Option<carbide_core::event::input::Input> {
            $crate::v023_convert_event!(event, window)
        }
    };
}