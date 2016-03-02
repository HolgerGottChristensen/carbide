//! Everything related to storing the state of user input. This includes the state of any
//! buttons on either the keyboard or the mouse, as well as the position of the mouse.
//! It also includes which widgets, if any, are capturing the keyboard and mouse.
//! This module exists mostly to support the `events::InputProvider` trait.

use events::UiEvent;
use input::keyboard::{NO_MODIFIER, ModifierKey, Key};
use position::Point;
use widget::Index;

use self::mouse::Mouse;


/// Holds the current state of user input.
///
/// This includes the state of all buttons on the keyboard and mouse, as well as the position of
/// the mouse.
///
/// It also includes which widgets, if any, are capturing keyboard and mouse input.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InputState {
    /// Mouse position and button state.
    pub mouse: Mouse,
    /// Which widget, if any, is currently capturing the keyboard
    pub widget_capturing_keyboard: Option<Index>,
    /// Which widget, if any, is currently capturing the mouse
    pub widget_capturing_mouse: Option<Index>,
    /// The widget that is currently under the mouse cursor.
    ///
    /// If the mouse is currently over multiple widgets, this index will represent the top-most,
    /// non-graphic-child widget.
    pub widget_under_mouse: Option<Index>,
    /// Which modifier keys are being held down.
    pub modifiers: ModifierKey,
}

impl InputState {

    /// Returns a fresh new input state
    pub fn new() -> InputState {
        InputState{
            mouse: Mouse::new(),
            widget_capturing_keyboard: None,
            widget_capturing_mouse: None,
            widget_under_mouse: None,
            modifiers: NO_MODIFIER,
        }
    }

    /// Updates the input state based on an event.
    pub fn update(&mut self, event: &UiEvent) {
        use input::{Button, Motion, Input};

        match *event {
            UiEvent::Raw(Input::Press(Button::Mouse(mouse_button))) => {
                self.mouse.buttons.press(mouse_button, self.mouse.xy);
            },
            UiEvent::Raw(Input::Release(Button::Mouse(mouse_button))) => {
                self.mouse.buttons.release(mouse_button);
            },
            UiEvent::Raw(Input::Move(Motion::MouseRelative(x, y))) => {
                self.mouse.xy = [x, y];
            },
            UiEvent::Raw(Input::Press(Button::Keyboard(key))) => {
                get_modifier(key).map(|modifier| self.modifiers.insert(modifier));
            },
            UiEvent::Raw(Input::Release(Button::Keyboard(key))) => {
                get_modifier(key).map(|modifier| self.modifiers.remove(modifier));
            },
            UiEvent::WidgetCapturesKeyboard(idx) => {
                self.widget_capturing_keyboard = Some(idx);
            },
            UiEvent::WidgetUncapturesKeyboard(_) => {
                self.widget_capturing_keyboard = None;
            },
            UiEvent::WidgetCapturesMouse(idx) => {
                self.widget_capturing_mouse = Some(idx);
            },
            UiEvent::WidgetUncapturesMouse(_) =>  {
                self.widget_capturing_mouse = None;
            },
            _ => {}
        }
    }

    /// Returns a copy of the InputState relative to the given `position::Point`
    pub fn relative_to(mut self, xy: Point) -> InputState {
        self.mouse.xy = ::vecmath::vec2_sub(self.mouse.xy, xy);
        self.mouse.buttons = self.mouse.buttons.relative_to(xy);
        self
    }

}

fn get_modifier(key: Key) -> Option<ModifierKey> {
    use input::keyboard::{CTRL, SHIFT, ALT, GUI};
    match key {
        Key::LCtrl | Key::RCtrl => Some(CTRL),
        Key::LShift | Key::RShift => Some(SHIFT),
        Key::LAlt | Key::RAlt => Some(ALT),
        Key::LGui | Key::RGui => Some(GUI),
        _ => None
    }
}


/// Mouse specific state.
pub mod mouse {
    use position::Point;
    #[doc(inline)]
    pub use input::MouseButton as Button;

    /// The max total number of buttons on a mouse.
    pub const NUM_BUTTONS: usize = 9;

    /// The state of the `Mouse`, including it's position and button states.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Mouse {
        /// A map that stores the up/down state of each button.
        ///
        /// If the button is down, then it stores the position of the mouse when the button was first
        /// pressed.
        pub buttons: ButtonMap,
        /// The current position of the mouse.
        pub xy: Point,
    }

    /// Whether the button is up or down.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum ButtonPosition {
        /// The button is up (i.e. pressed).
        Up,
        /// The button is down and was originally pressed down at the given `Point`.
        Down(Point),
    }

    /// Stores the state of all mouse buttons.
    ///
    /// If the mouse button is down, it stores the position of the mouse when the button was pressed
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct ButtonMap {
        buttons: [ButtonPosition; NUM_BUTTONS]
    }

    /// An iterator yielding all pressed buttons.
    #[derive(Clone)]
    pub struct PressedButtons<'a> {
        buttons: ::std::iter::Enumerate<::std::slice::Iter<'a, ButtonPosition>>,
    }

    impl Mouse {
        /// Construct a new default `Mouse`.
        pub fn new() -> Self {
            Mouse {
                buttons: ButtonMap::new(),
                xy: [0.0, 0.0],
            }
        }
    }

    impl ButtonPosition {

        /// If the mouse button is down, return a new one with position relative to the given `xy`.
        pub fn relative_to(self, xy: Point) -> Self {
            match self {
                ButtonPosition::Down(pos) => ButtonPosition::Down([pos[0] - xy[0], pos[1] - xy[1]]),
                button_pos => button_pos,
            }
        }

        /// Is the `ButtonPosition` down.
        pub fn is_down(&self) -> bool {
            match *self {
                ButtonPosition::Down(_) => true,
                _ => false,
            }
        }

        /// Is the `ButtonPosition` up.
        pub fn is_up(&self) -> bool {
            match *self {
                ButtonPosition::Up => true,
                _ => false,
            }
        }

        /// Returns the position at which the button was pressed if it's down.
        pub fn xy_if_down(&self) -> Option<Point> {
            match *self {
                ButtonPosition::Down(xy) => Some(xy),
                _ => None,
            }
        }

    }

    impl ButtonMap {

        /// Returns a new button map with all states set to `None`
        pub fn new() -> Self {
            ButtonMap{
                buttons: [ButtonPosition::Up; NUM_BUTTONS]
            }
        }

        /// Returns a copy of the ButtonMap relative to the given `Point`
        pub fn relative_to(self, xy: Point) -> Self {
            self.buttons.iter().enumerate().fold(ButtonMap::new(), |mut map, (idx, button_pos)| {
                map.buttons[idx] = button_pos.relative_to(xy);
                map
            })
        }

        /// The state of the left mouse button.
        pub fn left(&self) -> &ButtonPosition {
            &self[Button::Left]
        }

        /// The state of the middle mouse button.
        pub fn middle(&self) -> &ButtonPosition {
            &self[Button::Middle]
        }

        /// The state of the right mouse button.
        pub fn right(&self) -> &ButtonPosition {
            &self[Button::Right]
        }

        /// Sets the `Button` in the `Down` position.
        pub fn press(&mut self, button: Button, xy: Point) {
            self.buttons[button_to_idx(button)] = ButtonPosition::Down(xy);
        }

        /// Set's the `Button` in the `Up` position.
        pub fn release(&mut self, button: Button) {
            self.buttons[button_to_idx(button)] = ButtonPosition::Up;
        }

        /// An iterator yielding all pressed mouse buttons along with the location at which they
        /// were originally pressed.
        pub fn pressed(&self) -> PressedButtons {
            PressedButtons { buttons: self.buttons.iter().enumerate() }
        }

    }

    /// Converts a `Button` to its respective index within the `ButtonMap`.
    fn button_to_idx(button: Button) -> usize {
        let idx: u32 = button.into();
        idx as usize
    }

    /// Converts a `ButtonMap` index to its respective `Button`.
    fn idx_to_button(idx: usize) -> Button {
        (idx as u32).into()
    }

    impl ::std::ops::Index<Button> for ButtonMap {
        type Output = ButtonPosition;
        fn index(&self, button: Button) -> &Self::Output {
            &self.buttons[button_to_idx(button)]
        }
    }

    impl<'a> Iterator for PressedButtons<'a> {
        type Item = (Button, Point);
        fn next(&mut self) -> Option<Self::Item> {
            while let Some((idx, button_pos)) = self.buttons.next() {
                if let ButtonPosition::Down(xy) = *button_pos {
                    return Some((idx_to_button(idx), xy));
                }
            }
            None
        }
    }

}



#[test]
fn pressed_next_returns_none_if_no_buttons_are_pressed() {
    let map = mouse::ButtonMap::new();
    let pressed = map.pressed().next();
    assert!(pressed.is_none());
}

#[test]
fn pressed_next_should_return_first_pressed_button() {
    let mut map = mouse::ButtonMap::new();

    map.press(mouse::Button::Right, [3.0, 3.0]);
    map.press(mouse::Button::X1, [5.4, 4.5]);

    let pressed = map.pressed().next();
    assert_eq!(Some((mouse::Button::Right, [3.0, 3.0])), pressed);
}

#[test]
fn button_down_should_store_the_point() {
    let mut map = mouse::ButtonMap::new();
    let xy = [2.0, 5.0];
    map.press(mouse::Button::Left, xy);

    assert_eq!(mouse::ButtonPosition::Down(xy), map[mouse::Button::Left]);
}

#[test]
fn input_state_should_be_made_relative_to_a_given_point() {
    let mut state = InputState::new();
    state.mouse.xy = [50.0, -10.0];
    state.mouse.buttons.press(mouse::Button::Middle, [-20.0, -10.0]);

    let relative_state = state.relative_to([20.0, 20.0]);
    assert_eq!([30.0, -30.0], relative_state.mouse.xy);
    assert_eq!(Some([-40.0, -30.0]), relative_state.mouse.buttons[mouse::Button::Middle].xy_if_down());
}
