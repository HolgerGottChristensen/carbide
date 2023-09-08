use std::collections::HashMap;
use std::time::{Duration, Instant};

use carbide_core::cursor::MouseCursor;
use carbide_core::focus::Refocus;
use carbide_core::widget::Widget;

use crate::draw::{Dimension, Position, Scalar};
use crate::environment::Environment;
use crate::event::{Button, CustomEvent, Input, Key, ModifierKey, Motion, MouseButton};
use crate::window::WindowId;

const N_CLICK_THRESHOLD: Duration = Duration::from_millis(500);
const MOUSE_CLICK_MAX_DISTANCE: f64 = 3.0;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug)]
pub struct EventHandler {
    pressed_keys: HashMap<Key, KeyboardEvent>,
    pressed_buttons: HashMap<MouseButton, MouseEvent>,
    modifiers: ModifierKey,
    last_click: Option<(Instant, MouseEvent)>,
    mouse_position: Position,
    events: Vec<(WidgetEvent, Option<WindowId>)>,
    any_focus: bool,
}

impl EventHandler {
    pub fn delegate_events(&mut self, widgets: &mut impl Widget, env: &mut Environment) -> bool {
        let now = Instant::now();
        let mut any_focus = self.any_focus;
        let events = self.get_events();

        env.set_cursor(MouseCursor::Arrow);

        for (event, window_id) in events {
            let window_id = *window_id;
            env.set_current_event_window_id(Box::new(move |e| {
                window_id.map(|id| id == e).unwrap_or(true)
            }));
            env.capture_time();
            match event {
                WidgetEvent::Mouse(mouse_event) => {
                    let consumed = false;
                    widgets.process_mouse_event(mouse_event, &consumed, env);
                }
                WidgetEvent::Keyboard(keyboard_event) => {
                    widgets.process_keyboard_event(keyboard_event, env);
                }
                WidgetEvent::Window(_) => {
                    widgets.process_other_event(event, env);
                }
                WidgetEvent::Touch(_) => {
                    widgets.process_other_event(event, env);
                }
                WidgetEvent::Custom(_) => {}
                WidgetEvent::DoneProcessingEvents => {
                    widgets.process_other_event(event, env);
                }
            }

            if let Some(request) = env.focus_request.clone() {
                match request {
                    Refocus::FocusRequest => {
                        println!("Process focus request");
                        any_focus = widgets.process_focus_request(
                            event,
                            &request,
                            env,
                        );
                    }
                    Refocus::FocusNext => {
                        println!("Focus next");
                        let focus_first = widgets.process_focus_next(
                            event,
                            &request,
                            false,
                            env,
                        );
                        if focus_first {
                            println!("Focus next back to first");
                            widgets.process_focus_next(
                                event,
                                &request,
                                true,
                                env,
                            );
                        }
                    }
                    Refocus::FocusPrevious => {
                        println!("Focus prev");
                        let focus_last = widgets.process_focus_previous(
                            event,
                            &request,
                            false,
                            env,
                        );
                        if focus_last {
                            println!("Focus prev forward to last");
                            widgets.process_focus_previous(
                                event,
                                &request,
                                true,
                                env,
                            );
                        }
                    }
                }
                env.focus_request = None;
            } else if !any_focus {
                match event {
                    WidgetEvent::Keyboard(KeyboardEvent::Press(key, modifier)) => {
                        if key == &Key::Tab {
                            if modifier == &ModifierKey::SHIFT {
                                // If focus is still up for grab we can assume that no element
                                // has been focused. This assumption breaks if there can be multiple
                                // widgets with focus at the same time
                                any_focus = !widgets.process_focus_previous(
                                    event,
                                    &Refocus::FocusPrevious,
                                    true,
                                    env
                                );
                            } else if modifier == &ModifierKey::NO_MODIFIER {
                                any_focus = !widgets.process_focus_next(
                                    event,
                                    &Refocus::FocusNext,
                                    true,
                                    env
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Todo: Consider being smarter about sending this event. We dont need to send it if no state changed this frame.
        // Currently used by foreach to check if updates has been made to its model.
        widgets.process_other_event(&WidgetEvent::DoneProcessingEvents, env);
        self.clear_events();
        self.any_focus = any_focus;

        if now.elapsed().as_millis() > 16 {
            println!("Frame took: {}", now.elapsed().as_secs_f32());
        }

        // Todo: Determine if an redraw is needed after events are processed
        return true;
    }

    pub fn get_events(&self) -> &Vec<(WidgetEvent, Option<WindowId>)> {
        &self.events
    }

    pub fn has_queued_events(&self) -> bool {
        self.get_events().len() > 0
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
    Touch(TouchEvent),
    Custom(CustomEvent),
    DoneProcessingEvents,
}

#[derive(Clone, Debug)]
pub enum MouseEvent {
    Press(MouseButton, Position, ModifierKey),
    Release(MouseButton, Position, ModifierKey),
    Click(MouseButton, Position, ModifierKey),
    Move {
        from: Position,
        to: Position,
        delta_xy: Position,
        modifiers: ModifierKey,
    },
    NClick(MouseButton, Position, ModifierKey, u32),
    Scroll {
        x: Scalar,
        y: Scalar,
        mouse_position: Position,
        modifiers: ModifierKey,
    },
    Drag {
        button: MouseButton,
        origin: Position,
        from: Position,
        to: Position,
        delta_xy: Position,
        total_delta_xy: Position,
        modifiers: ModifierKey,
    },
}

impl MouseEvent {
    pub fn get_current_mouse_position(&self) -> Position {
        match self {
            MouseEvent::Press(_, n, _) => *n,
            MouseEvent::Release(_, n, _) => *n,
            MouseEvent::Click(_, n, _) => *n,
            MouseEvent::Move { to, .. } => *to,
            MouseEvent::NClick(_, n, _, _) => *n,
            MouseEvent::Scroll { mouse_position, .. } => *mouse_position,
            MouseEvent::Drag { to, .. } => *to,
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
    Resize(Dimension),
    Focus,
    UnFocus,
    Redraw,
    CloseRequested,
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
        _ => None,
    }
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashMap::new(),
            pressed_buttons: HashMap::new(),
            modifiers: ModifierKey::default(),
            last_click: None,
            mouse_position: Position::new(0.0, 0.0),
            events: vec![],
            any_focus: false
        }
    }

    fn add_event(&mut self, event: WidgetEvent, window_id: Option<WindowId>) {
        if !matches!(event, WidgetEvent::Mouse(MouseEvent::Move {..})) {
            println!("Event: {:#?}", event);
        }

        if let WidgetEvent::Mouse(MouseEvent::Move {
            delta_xy,
            to,
            modifiers,
            ..
        }) = event
        {
            // We should only add move events where the mouse have actually moved
            if delta_xy.x != 0.0 || delta_xy.y != 0.0 {
                // If the last event was also a move event we can compress it to a single move event.
                if let Some((WidgetEvent::Mouse(MouseEvent::Move {
                    delta_xy: old_delta_xy,
                    modifiers: old_modifiers,
                    to: old_to,
                    ..
                }), _w_id)) = self.events.last_mut()
                {
                    old_delta_xy.x += delta_xy.x;
                    old_delta_xy.y += delta_xy.y;
                    *old_modifiers = modifiers;
                    *old_to = to;
                } else {
                    self.events.push((event, window_id));
                }
            }
        } else if let WidgetEvent::Mouse(MouseEvent::Scroll {
            x: new_x,
            y: new_y,
            mouse_position: new_mouse_position,
            modifiers: new_modifiers,
        }) = event
        {
            // If the last event was a scroll, we can compress the events into a single scroll event.
            if let Some((WidgetEvent::Mouse(MouseEvent::Scroll {
                x,
                y,
                mouse_position,
                modifiers,
            }), _w_id)) = self.events.last_mut()
            {
                *x += new_x;
                *y += new_y;
                *mouse_position = new_mouse_position;
                *modifiers = new_modifiers;
            } else {
                self.events.push((event, window_id));
            }
        } else {
            self.events.push((event, window_id));
        }
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
    pub fn compound_and_add_event(
        &mut self,
        event: Input,
        window_id: Option<WindowId>
    ) -> Option<WindowEvent> {
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
                    self.add_event(WidgetEvent::Mouse(event.clone()), window_id);
                    self.pressed_buttons.insert(mouse_button, event);

                    None
                }

                Button::Keyboard(key) => {
                    let event = KeyboardEvent::Press(key, modifiers);
                    self.add_event(WidgetEvent::Keyboard(event.clone()), window_id);
                    self.pressed_keys.insert(key, event);

                    // If some modifier key was pressed, add it to the current modifiers.
                    if let Some(modifier) = filter_modifier(key) {
                        self.modifiers.insert(modifier);
                    }

                    None
                }
            }

            // Some button was released.
            //
            // Checks for events in the following order:
            // 1. Click
            // 2. DoubleClick
            Input::Release(button_type) => match button_type {
                Button::Mouse(mouse_button) => {
                    // Add the event that the user released the mouse button at the specified location holding the current modifiers.
                    let event = MouseEvent::Release(mouse_button, mouse_xy, modifiers);
                    self.add_event(WidgetEvent::Mouse(event), window_id);

                    // The button is no longer pressed, so remove it from currently pressed buttons.
                    let pressed_event = self.pressed_buttons.remove(&mouse_button);

                    // A click should be emitted if within a threshold distance of the press.
                    let is_click = if let Some(MouseEvent::Press(_, location, _)) = pressed_event {
                        mouse_xy.dist(&location) < MOUSE_CLICK_MAX_DISTANCE
                    } else {
                        false
                    };

                    // A click will become a double click, if and only if it is within the
                    // double click threshold time based on the latest click of the same button.
                    let click_number = match self.last_click {
                        // Too long since last click, so we emit a click and not a double click
                        Some((time, _)) if Instant::now().duration_since(time) > N_CLICK_THRESHOLD => 1,

                        // Our previous click was a normal click of the same button within the
                        // same location as the previous click. The time is checked in a previous case.
                        Some((_, MouseEvent::Click(button, location, _))) if button == mouse_button && mouse_xy.dist(&location) < MOUSE_CLICK_MAX_DISTANCE => 2,

                        // Our previous click was a double click within time and with the same button
                        // and within range of the previous click.
                        Some((_, MouseEvent::NClick(button, location, _, n))) if button == mouse_button && mouse_xy.dist(&location) < MOUSE_CLICK_MAX_DISTANCE => n + 1,

                        // Either the previous click was not in range or not with the same button
                        Some((_, _)) => 1,

                        // No previous click, so we emit a click and not a double click
                        None => 1,
                    };

                    if is_click {
                        if click_number == 1 {
                            self.add_event(WidgetEvent::Mouse(MouseEvent::Click(mouse_button, mouse_xy, modifiers)), window_id);
                            self.last_click = Some((Instant::now(), MouseEvent::Click(mouse_button, mouse_xy, modifiers)));
                        } else {
                            self.add_event(WidgetEvent::Mouse(MouseEvent::NClick(mouse_button, mouse_xy, modifiers, click_number)), window_id);
                            self.last_click = Some((Instant::now(), MouseEvent::NClick(mouse_button, mouse_xy, modifiers, click_number)));
                        }
                    } else {
                        self.last_click = None;
                    }
                    None
                }

                Button::Keyboard(key) => {
                    let event = KeyboardEvent::Release(key, modifiers);
                    self.add_event(WidgetEvent::Keyboard(event), window_id);
                    let pressed_event = self.pressed_keys.remove(&key);

                    if let Some(KeyboardEvent::Press(..)) = pressed_event {
                        let click_event = KeyboardEvent::Click(key, modifiers);
                        self.add_event(WidgetEvent::Keyboard(click_event), window_id);
                    }

                    if let Some(modifier) = filter_modifier(key) {
                        self.modifiers.remove(modifier);
                    }

                    None
                }
            }

            // The window was resized.
            Input::Resize(w, h) => {
                // Create a `WindowResized` event.
                let (w, h) = (w as Scalar, h as Scalar);
                let event = WindowEvent::Resize(Dimension::new(w, h));
                self.add_event(WidgetEvent::Window(event), window_id);
                Some(WindowEvent::Resize(Dimension::new(w, h)))
            }

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
                        let mouse_xy = Position::new(x, y);

                        let delta_xy = mouse_xy - last_mouse_xy;

                        let move_event = MouseEvent::Move {
                            from: last_mouse_xy,
                            to: mouse_xy,
                            delta_xy,
                            modifiers,
                        };

                        self.add_event(WidgetEvent::Mouse(move_event), window_id);

                        // Check for drag events.

                        let distance = (delta_xy.x + delta_xy.y).abs().sqrt();
                        let drag_threshold = 0.0;

                        if distance > drag_threshold {
                            let mut events = vec![];
                            for (button, evt) in self.pressed_buttons.iter() {
                                match evt {
                                    MouseEvent::Press(_, location, _) => {
                                        let total_delta_xy = mouse_xy - *location;
                                        let drag_event = MouseEvent::Drag {
                                            button: *button,
                                            origin: *location,
                                            from: last_mouse_xy,
                                            to: mouse_xy,
                                            delta_xy,
                                            total_delta_xy,
                                            modifiers,
                                        };

                                        events.push(WidgetEvent::Mouse(drag_event));
                                    }
                                    _ => {}
                                }
                            }

                            events.iter().for_each(|evt| self.add_event(evt.clone(), window_id))
                        }

                        // Update the position of the mouse within the global_input's
                        // input::State.
                        self.mouse_position = mouse_xy;

                        //ui.track_widget_under_mouse_and_update_capturing();
                    }

                    // Some scrolling occurred (e.g. mouse scroll wheel).
                    Motion::Scroll { x, y } => {
                        let event = MouseEvent::Scroll {
                            x,
                            y,
                            mouse_position: mouse_xy,
                            modifiers,
                        };
                        self.add_event(WidgetEvent::Mouse(event), window_id);
                    }

                    _ => (),
                }
                None
            }

            Input::Text(string) => {
                let event = KeyboardEvent::Text(string, modifiers);
                self.add_event(WidgetEvent::Keyboard(event), window_id);

                None
            }

            Input::Touch(touch) => match touch.phase {
                _ => None,
            }

            Input::Focus(focused) if focused == true => {
                self.add_event(WidgetEvent::Window(WindowEvent::Focus), window_id);
                Some(WindowEvent::Focus)
                //ui.needs_redraw()
            }
            Input::Focus(_focused) => {
                self.add_event(WidgetEvent::Window(WindowEvent::UnFocus), window_id);
                None
            }

            Input::Redraw => {
                //ui.needs_redraw();
                Some(WindowEvent::Redraw)
            }
            Input::Custom(event) => {
                self.add_event(WidgetEvent::Custom(event), window_id);
                None
            }
            Input::CloseRequested => {
                self.add_event(WidgetEvent::Window(WindowEvent::CloseRequested), window_id);
                None
            }
        }
    }
}
