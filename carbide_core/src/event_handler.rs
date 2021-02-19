use std::collections::HashMap;
use std::time::Duration;

use instant::Instant;

use crate::event::input::Input;
use crate::Scalar;
use crate::utils;
use crate::piston_input::{Key, MouseButton, Button};
use crate::piston_input::keyboard::ModifierKey;
use crate::Point;
use crate::position::Dimensions;
use crate::event::Motion;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug)]
pub struct EventHandler {
    pressed_keys: HashMap<Key, KeyboardEvent>,
    pressed_buttons: HashMap<MouseButton, MouseEvent>,
    modifiers: ModifierKey,
    last_click: Option<(Instant, MouseEvent)>,
    mouse_position: Point,
    events: Vec<WidgetEvent>,
}

impl EventHandler {
    pub fn get_events(&self) -> &Vec<WidgetEvent> {
        &self.events
    }

    pub fn clear_events(&mut self) {
        self.events.clear()
    }
}

#[derive(Clone, Debug)]
pub enum WidgetEvent {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Window(WindowEvent),
    Touch(TouchEvent)
}

#[derive(Clone, Debug)]
pub enum MouseEvent {
    Press(MouseButton, Point, ModifierKey),
    Release(MouseButton, Point, ModifierKey),
    Click(MouseButton, Point, ModifierKey),
    Move{
        from: Point,
        to: Point,
        delta_xy: Point,
        modifiers: ModifierKey
    },
    NClick(MouseButton, Point, ModifierKey, u32),
    Scroll {x: Scalar, y: Scalar, mouse_position: Point, modifiers: ModifierKey},
    Drag {
        button: MouseButton,
        origin: Point,
        from: Point,
        to: Point,
        delta_xy: Point,
        total_delta_xy: Point,
        modifiers: ModifierKey
    },
}

impl MouseEvent {
    pub fn get_current_mouse_position(&self) -> Point {
        match self {
            MouseEvent::Press(_, n, _) => {*n}
            MouseEvent::Release(_, n, _) => {*n}
            MouseEvent::Click(_, n, _) => {*n}
            MouseEvent::Move {to, .. } => {*to}
            MouseEvent::NClick(_, n, _, _) => {*n}
            MouseEvent::Scroll { mouse_position, .. } => {*mouse_position}
            MouseEvent::Drag {to, .. } => {*to}
        }
    }
}

#[derive(Clone, Debug)]
pub enum KeyboardEvent {
    Press(Key, ModifierKey),
    Release(Key, ModifierKey),
    Click(Key, ModifierKey),
    Text(String, ModifierKey),
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Resize(Dimensions),
    Focus,
    UnFocus,
    Redraw
}

#[derive(Clone, Debug)]
pub enum TouchEvent {
    // Todo: Handle touch events
}


fn filter_modifier(key: Key) -> Option<ModifierKey> {
    match key {
        Key::LCtrl | Key::RCtrl => Some(ModifierKey::CTRL),
        Key::LShift | Key::RShift => Some(ModifierKey::SHIFT),
        Key::LAlt | Key::RAlt => Some(ModifierKey::ALT),
        Key::LGui | Key::RGui => Some(ModifierKey::GUI),
        _ => None
    }
}

impl EventHandler {

    pub fn new() -> Self {
        Self {
            pressed_keys: HashMap::new(),
            pressed_buttons: HashMap::new(),
            modifiers: ModifierKey::default(),
            last_click: None,
            mouse_position: [0.0, 0.0],
            events: vec![]
        }
    }

    #[cfg(not(debug_assertions))]
    fn add_event(&mut self, event: WidgetEvent) {
        self.events.push(event);
    }

    #[cfg(debug_assertions)]
    fn add_event(&mut self, event: WidgetEvent) {
        /*if let WidgetEvent::Mouse(MouseEvent::Move {..}) = event {} else {
            println!("{:?}", &event);
        }*/
        self.events.push(event);
    }

    /// Handle raw window events and update the `Ui` state accordingly.
    ///
    /// This occurs within several stages:
    ///
    /// 1. Convert the user's given `event` to a `RawEvent` so that the `Ui` may use it.
    /// 2. Interpret the `RawEvent` for higher-level `Event`s such as `DoubleClick`,
    ///    `WidgetCapturesKeyboard`, etc.
    /// 3. Update the `Ui`'s `global_input` `State` accordingly, depending on the `RawEvent`.
    /// 4. Store newly produced `event::Ui`s within the `global_input` so that they may be filtered
    ///    and fed to `Widget`s next time `Ui::set_widget` is called.
    ///
    /// This method *drives* the `Ui` forward, and is what allows for using carbide's `Ui` with any
    /// window event stream.
    ///
    /// The given `event` must implement the **ToRawEvent** trait so that it can be converted to a
    /// `RawEvent` that can be used by the `Ui`.
    pub fn handle_event(&mut self, event: Input, window_dimensions: Dimensions) -> Option<WindowEvent>{


        // A function for filtering `ModifierKey`s.


        // Here we handle all user input given to carbide.
        //
        // Not only do we store the `Input` event as an `Event::Raw`, we also use them to
        // interpret higher level events such as `Click` or `Drag`.
        //
        // Finally, we also ensure that the `current_state` is up-to-date.

        //ui.global_input.push_event(event.clone().into());

        // Get current state
        let modifiers = self.modifiers;
        let mouse_xy = self.mouse_position;

        match event {

            // Some button was pressed, whether keyboard, mouse or some other device.
            Input::Press(button_type) => match button_type {

                // Check to see whether we need to (un)capture the keyboard or mouse.
                Button::Mouse(mouse_button) => {
                    let event = MouseEvent::Press(mouse_button, mouse_xy, modifiers);
                    self.add_event(WidgetEvent::Mouse(event.clone()));
                    self.pressed_buttons.insert(mouse_button, event);

                    None
                },

                Button::Keyboard(key) => {
                    let event = KeyboardEvent::Press(key, modifiers);
                    self.add_event(WidgetEvent::Keyboard(event.clone()));
                    self.pressed_keys.insert(key, event);

                    // If some modifier key was pressed, add it to the current modifiers.
                    if let Some(modifier) = filter_modifier(key) {
                        self.modifiers.insert(modifier);
                    }

                    None
                },

                _ => {
                    None
                }
            },

            // Some button was released.
            //
            // Checks for events in the following order:
            // 1. Click
            // 2. DoubleClick
            // 2. WidgetUncapturesMouse
            Input::Release(button_type) => match button_type {

                Button::Mouse(mouse_button) => {

                    let event = MouseEvent::Release(mouse_button, mouse_xy, modifiers);
                    self.add_event(WidgetEvent::Mouse(event));
                    let pressed_event = self.pressed_buttons.remove(&mouse_button);
                    let now = Instant::now();
                    let n_click_threshold = Duration::from_millis(500);

                    if let Some((time, MouseEvent::NClick(button, location, _, n))) = self.last_click {
                        if button == mouse_button &&
                            location == mouse_xy &&
                            now.duration_since(time) < n_click_threshold {
                            let n_click_event = MouseEvent::NClick(mouse_button, mouse_xy, modifiers, n + 1);
                            self.add_event(WidgetEvent::Mouse(n_click_event.clone()));
                            self.last_click = Some((now, n_click_event));
                        }
                    } else if let Some((time, MouseEvent::Click(button, location, _))) = self.last_click {
                        if button == mouse_button &&
                            location == mouse_xy &&
                            now.duration_since(time) < n_click_threshold {
                            let n_click_event = MouseEvent::NClick(mouse_button, mouse_xy, modifiers, 2);
                            self.add_event(WidgetEvent::Mouse(n_click_event.clone()));
                            self.last_click = Some((now, n_click_event));
                        }
                    }

                    // Handle click events
                    if let Some(MouseEvent::Press(_, location, _)) = pressed_event {
                        if mouse_xy == location {
                            let click_event = MouseEvent::Click(mouse_button, mouse_xy, modifiers);
                            if let Some((time, MouseEvent::NClick(_,_,_,_))) = self.last_click {
                                if now.duration_since(time) >= n_click_threshold {
                                    self.add_event(WidgetEvent::Mouse(click_event.clone()));
                                    self.last_click = Some((now, click_event));
                                }
                            } else {
                                self.add_event(WidgetEvent::Mouse(click_event.clone()));
                                self.last_click = Some((now, click_event));
                            }

                        }
                    };

                    None
                },

                Button::Keyboard(key) => {
                    let event = KeyboardEvent::Release(key, modifiers);
                    self.add_event(WidgetEvent::Keyboard(event));
                    let pressed_event = self.pressed_keys.remove(&key);

                    if let Some(KeyboardEvent::Press(..)) = pressed_event {
                        let click_event = KeyboardEvent::Click(key, modifiers);
                        self.add_event(WidgetEvent::Keyboard(click_event));
                    }

                    if let Some(modifier) = filter_modifier(key) {
                        self.modifiers.remove(modifier);
                    }

                    None
                },

                _ => {
                    None
                },
            },

            // The window was resized.
            Input::Resize(w, h) => {
                // Create a `WindowResized` event.
                let (w, h) = (w as Scalar, h as Scalar);
                let event = WindowEvent::Resize([w, h]);
                self.add_event(WidgetEvent::Window(event));
                Some(WindowEvent::Resize([w, h]))
            },

            // The mouse cursor was moved to a new position.
            //
            // Checks for events in the following order:
            // 1. `Drag`
            // 2. `WidgetUncapturesMouse`
            // 3. `WidgetCapturesMouse`
            Input::Motion(motion) => {

                match motion {
                    Motion::MouseCursor { x, y } => {
                        let last_mouse_xy = self.mouse_position;
                        let mouse_xy = [x + window_dimensions[0] / 2.0, window_dimensions[1] - (y + window_dimensions[1] / 2.0)];
                        let delta_xy = utils::vec2_sub(mouse_xy, last_mouse_xy);

                        let move_event = MouseEvent::Move {
                            from: last_mouse_xy,
                            to: mouse_xy,
                            delta_xy,
                            modifiers,
                        };

                        self.add_event(WidgetEvent::Mouse(move_event));

                        // Check for drag events.

                        let distance = (delta_xy[0] + delta_xy[1]).abs().sqrt();
                        let drag_threshold = 0.0;

                        if distance > drag_threshold {

                            let mut events = vec![];
                            for (button, evt) in self.pressed_buttons.iter() {
                                match evt {
                                    MouseEvent::Press(_, location, _) => {
                                        let total_delta_xy = utils::vec2_sub(mouse_xy, *location);
                                        let drag_event = MouseEvent::Drag {
                                            button: *button,
                                            origin: *location,
                                            from: last_mouse_xy,
                                            to: mouse_xy,
                                            delta_xy,
                                            total_delta_xy,
                                            modifiers
                                        };

                                        events.push(WidgetEvent::Mouse(drag_event));

                                    }
                                    _ => {}
                                }
                            }

                            events.iter().for_each(|evt| self.add_event(evt.clone()))

                        }

                        // Update the position of the mouse within the global_input's
                        // input::State.
                        self.mouse_position = mouse_xy;

                        //ui.track_widget_under_mouse_and_update_capturing();
                    },

                    // Some scrolling occurred (e.g. mouse scroll wheel).
                    Motion::Scroll { x, y } => {
                        let event = MouseEvent::Scroll { x, y, mouse_position: mouse_xy, modifiers };
                        self.add_event(WidgetEvent::Mouse(event));
                    },

                    _ => (),
                }
                None
            },

            Input::Text(string) => {
                let event = KeyboardEvent::Text(string, modifiers);
                self.add_event(WidgetEvent::Keyboard(event));

                None
            },

            Input::Touch(touch) => match touch.phase {
                _ => {None}
            },

            Input::Focus(focused) if focused == true => {
                self.add_event(WidgetEvent::Window(WindowEvent::Focus));
                Some(WindowEvent::Focus)
                //ui.needs_redraw()
            },
            Input::Focus(_focused) => {
                self.add_event(WidgetEvent::Window(WindowEvent::UnFocus));
                None
            },

            Input::Redraw => {
                //ui.needs_redraw();
                Some(WindowEvent::Redraw)
            },
        }
    }
}