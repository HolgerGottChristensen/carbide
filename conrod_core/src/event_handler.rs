use event::input::Input;
use ::{Ui, Scalar};
use event::press::PressEvent;
use event::button::ButtonEvent;
use event::ui::UiEvent;
use input::{Source, ModifierKey, MouseButton, Key, Motion};
use event::release::Release;
use event::click::Click;
use event::double_click::DoubleClick;
use ::{utils, graph};
use event::drag::Drag;
use event::scroll::Scroll;
use event::text::Text;
use input::touch::Phase;
use event::tap::Tap;
use widget::scroll::State;
use input::state::touch::{Touch, Start};
use Point;
use input::{Button};
use std::collections::HashMap;
use instant::Instant;
use std::time::Duration;
use event_handler::WidgetEvent::Keyboard;
use position::Dimensions;

#[derive(Debug)]
pub struct EventHandler {
    pressed_keys: HashMap<Key, KeyboardEvent>,
    pressed_buttons: HashMap<MouseButton, MouseEvent>,
    modifiers: ModifierKey,
    last_click: Option<(Instant, MouseEvent)>,
    mouse_position: Point,
    events: Vec<WidgetEvent>
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
    DoubleClick(MouseButton, Point, ModifierKey),
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
            MouseEvent::DoubleClick(_, n, _) => {*n}
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

    fn add_event(&mut self, event: WidgetEvent) {
        println!("{:?}", &event);
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
    /// This method *drives* the `Ui` forward, and is what allows for using conrod's `Ui` with any
    /// window event stream.
    ///
    /// The given `event` must implement the **ToRawEvent** trait so that it can be converted to a
    /// `RawEvent` that can be used by the `Ui`.
    pub fn handle_event(&mut self, event: Input, window_dimensions: Dimensions) -> Option<WindowEvent>{


        // A function for filtering `ModifierKey`s.


        // Here we handle all user input given to conrod.
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

                    /*let press = PressEvent {
                        button: ButtonEvent::Mouse(mouse_button, mouse_xy),
                        modifiers: ui.global_input.current.modifiers,
                    };
                    let widget = ui.global_input.current.widget_capturing_mouse;
                    let press_event = UiEvent::Press(widget, press).into();
                    ui.global_input.push_event(press_event);
                     */
                    /*if let MouseButton::Left = mouse_button {
                    // Check to see if we need to uncapture the keyboard.
                    if let Some(idx) = ui.global_input.current.widget_capturing_keyboard {
                        if Some(idx) != ui.global_input.current.widget_under_mouse {
                            let source = Source::Keyboard;
                            let event = UiEvent::WidgetUncapturesInputSource(idx, source);
                            ui.global_input.push_event(event.into());
                            ui.global_input.current.widget_capturing_keyboard = None;
                        }
                    }

                    // Check to see if we need to capture the keyboard.
                    if let Some(idx) = ui.global_input.current.widget_under_mouse {
                        let source = Source::Keyboard;
                        let event = UiEvent::WidgetCapturesInputSource(idx, source);
                        ui.global_input.push_event(event.into());
                        ui.global_input.current.widget_capturing_keyboard = Some(idx);
                    }
                }*/

                    // Keep track of pressed buttons in the current input::State.
                    //let xy = ui.global_input.current.mouse.xy;
                    //let widget = ui.global_input.current.widget_under_mouse;
                    //ui.global_input.current.mouse.buttons.press(mouse_button, xy, widget);
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

                    // Create a keyboard `Press` event.
                    /*let press = PressEvent {
                        button: ButtonEvent::Keyboard(key),
                        modifiers: ui.global_input.current.modifiers,
                    };
                    let widget = ui.global_input.current.widget_capturing_keyboard;
                    let press_event = UiEvent::Press(widget, press).into();
                    ui.global_input.push_event(press_event);

                    // If some modifier key was pressed, add it to the current modifiers.
                    if let Some(modifier) = filter_modifier(key) {
                        ui.global_input.current.modifiers.insert(modifier);
                    }*/

                    // If `Esc` was pressed, check to see if we need to cancel a `Drag` or
                    // uncapture a widget.
                    //if let Key::Escape = key {
                    // TODO:
                    // 1. Cancel `Drag` if currently under way.
                    // 2. If mouse is captured due to pinning widget with left mouse button,
                    //    cancel capturing.
                    //}
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
                    let double_click_threshold = Duration::from_millis(500);

                    // Handle double clicks
                    if let Some((time, MouseEvent::Click(button, location, _))) = self.last_click {
                        if button == mouse_button &&
                            location == mouse_xy &&
                            now.duration_since(time) < double_click_threshold {
                            let double_click_event = MouseEvent::DoubleClick(mouse_button, mouse_xy, modifiers);
                            self.add_event(WidgetEvent::Mouse(double_click_event));
                        }
                    }

                    // Handle click events
                    if let Some(MouseEvent::Press(_, location, _)) = pressed_event {
                        if mouse_xy == location {
                            let click_event = MouseEvent::Click(mouse_button, mouse_xy, modifiers);
                            self.add_event(WidgetEvent::Mouse(click_event.clone()));
                            self.last_click = Some((now, click_event));
                        }
                    };


                    // Create a `Release` event.
                   /* let release = Release {
                        button: ButtonEvent::Mouse(mouse_button, mouse_xy),
                        modifiers: ui.global_input.current.modifiers,
                    };
                    let widget = ui.global_input.current.widget_capturing_mouse;
                    let release_event = UiEvent::Release(widget, release).into();
                    ui.global_input.push_event(release_event);

                    // Check for `Click` and `DoubleClick` events.
                    let down = ui.global_input.current.mouse.buttons[mouse_button].if_down();
                    if let Some((_, widget)) = down {

                        // The widget that's being clicked.
                        let clicked_widget = ui.global_input.current.widget_under_mouse
                            .and_then(|released| widget.and_then(|pressed| {
                                if pressed == released { Some(released) } else { None }
                            }));

                        let click = Click {
                            button: mouse_button,
                            xy: ui.global_input.current.mouse.xy,
                            modifiers: ui.global_input.current.modifiers,
                        };

                        let click_event = UiEvent::Click(clicked_widget, click).into();
                        ui.global_input.push_event(click_event);

                        let now = instant::Instant::now();
                        let double_click = ui.global_input.last_click
                            .and_then(|(last_time, last_click)| {

                                // If the button of this click is different to the button
                                // of last click, don't create a `DoubleClick`.
                                if click.button != last_click.button {
                                    return None;
                                }

                                // If the mouse has moved since the last click, don't
                                // create a `DoubleClick`.
                                if click.xy != last_click.xy {
                                    return None;
                                }

                                // If the duration since the last click is longer than the
                                // double_click_threshold, don't create a `DoubleClick`.
                                let duration = now.duration_since(last_time);
                                // TODO: Work out how to get this threshold from the user's
                                // system preferences.
                                let threshold = ui.theme.double_click_threshold;
                                if duration >= threshold {
                                    return None;
                                }

                                Some(DoubleClick {
                                    button: click.button,
                                    xy: click.xy,
                                    modifiers: click.modifiers,
                                })
                            });

                        if let Some(double_click) = double_click {
                            // Reset the `last_click` to `None`, as to not register another
                            // `DoubleClick` on the next consecutive `Click`.
                            ui.global_input.last_click = None;
                            let double_click_event =
                                UiEvent::DoubleClick(clicked_widget, double_click).into();
                            ui.global_input.push_event(double_click_event);
                        } else {
                            // Set the `Click` that we just stored as the `last_click`.
                            ui.global_input.last_click = Some((now, click));
                        }
                    }

                    // Uncapture widget capturing mouse if MouseButton::Left is down and
                    // widget_under_mouse != capturing widget.
                    /*if let MouseButton::Left = mouse_button {
                    if let Some(idx) = ui.global_input.current.widget_capturing_mouse {
                        if Some(idx) != ui.global_input.current.widget_under_mouse {
                            let source = Source::Mouse;
                            let event = UiEvent::WidgetUncapturesInputSource(idx, source);
                            ui.global_input.push_event(event.into());
                            ui.global_input.current.widget_capturing_mouse = None;
                        }
                    }
                }*/

                    // Release the given mouse_button from the input::State.
                    ui.global_input.current.mouse.buttons.release(mouse_button);*/
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

                    // Create a `Release` event.
                    /* let release = Release {
                        button: ButtonEvent::Keyboard(key),
                        modifiers: ui.global_input.current.modifiers,
                    };
                    let widget = ui.global_input.current.widget_capturing_keyboard;
                    let release_event = UiEvent::Release(widget, release).into();
                    ui.global_input.push_event(release_event);
                     */
                    // If a modifier key was released, remove it from the current set.
                    /*if let Some(modifier) = filter_modifier(key) {
                        ui.global_input.current.modifiers.remove(modifier);
                    }*/
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


                //let window_resized = UiEvent::WindowResized([w, h]).into();
                //ui.global_input.push_event(window_resized);

                //ui.win_w = w;
                //ui.win_h = h;
                //ui.needs_redraw();
                //ui.track_widget_under_mouse_and_update_capturing();
                Some(WindowEvent::Resize([w, h]))
            },

            // The mouse cursor was moved to a new position.
            //
            // Checks for events in the following order:
            // 1. `Drag`
            // 2. `WidgetUncapturesMouse`
            // 3. `WidgetCapturesMouse`
            Input::Motion(motion) => {

                // Create a `Motion` event.
                /*let move_ = crate::event::motion::Motion {
                    motion,
                    modifiers: ui.global_input.current.modifiers,
                };*/
                //let widget = ui.global_input.current.widget_capturing_mouse;
                //let move_event = UiEvent::Motion(widget, move_).into();
                //ui.global_input.push_event(move_event);



                match motion {
                    Motion::MouseCursor { x, y } => {
                        let last_mouse_xy = self.mouse_position;
                        let mouse_xy = [x + window_dimensions[0] / 2.0, window_dimensions[1] - (y + window_dimensions[1] / 2.0)];
                        let delta_xy = utils::vec2_sub(mouse_xy, last_mouse_xy);

                        let move_event = MouseEvent::Move{
                            from: last_mouse_xy,
                            to: mouse_xy,
                            delta_xy,
                            modifiers
                        };
                        // Todo: Re-add when we need mouse move events
                        //self.add_event(WidgetEvent::Mouse(move_event));

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


                            // For each button that is down, trigger a drag event.
                           /* let buttons = ui.global_input.current.mouse.buttons.clone();
                            for (btn, btn_xy, widget) in buttons.pressed() {
                                let total_delta_xy = utils::vec2_sub(mouse_xy, btn_xy);
                                let event = UiEvent::Drag(widget, Drag {
                                    button: btn,
                                    origin: btn_xy,
                                    from: last_mouse_xy,
                                    to: mouse_xy,
                                    delta_xy: delta_xy,
                                    total_delta_xy: total_delta_xy,
                                    modifiers: ui.global_input.current.modifiers,
                                }).into();
                                ui.global_input.push_event(event);
                            }*/
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





                        /*let mut scrollable_widgets = {
                            let depth_order = &ui.depth_order.indices;
                            let mouse_xy = ui.global_input.current.mouse.xy;
                            graph::algo::pick_scrollable_widgets(depth_order, mouse_xy)
                        };

                        // Iterate through the scrollable widgets from top to bottom.
                        //
                        // A scroll event will be created for the first scrollable widget
                        // that hasn't already reached the bound of the scroll event's
                        // direction.
                        while let Some(idx) =
                        scrollable_widgets.next(&ui.widget_graph,
                                                &ui.depth_order.indices,
                                                &ui.theme)
                        {
                            let (kid_area, maybe_x_scroll, maybe_y_scroll) =
                                match ui.widget_graph.widget(idx) {
                                    Some(widget) => {
                                        (widget.kid_area,
                                         widget.maybe_x_scroll_state,
                                         widget.maybe_y_scroll_state)
                                    },
                                    None => continue,
                                };

                            fn offset_is_at_bound<A>(scroll: &State<A>,
                                                     additional_offset: Scalar) -> bool {
                                fn approx_eq(a: Scalar, b: Scalar) -> bool {
                                    (a - b).abs() < 0.000001
                                }

                                if additional_offset.is_sign_positive() {
                                    let max = utils::partial_max(scroll.offset_bounds.start,
                                                                 scroll.offset_bounds.end);
                                    approx_eq(scroll.offset, max)
                                } else {
                                    let min = utils::partial_min(scroll.offset_bounds.start,
                                                                 scroll.offset_bounds.end);
                                    approx_eq(scroll.offset, min)
                                }
                            }

                            let mut scroll_x = false;
                            let mut scroll_y = false;

                            // Check whether the x axis is scrollable.
                            if x != 0.0 {
                                let new_scroll =
                                    State::update(ui, idx, &kid_area,
                                                  maybe_x_scroll, x);
                                if let Some(prev_scroll) = maybe_x_scroll {
                                    let (prev_is_at_bound, new_is_at_bound) =
                                        (offset_is_at_bound(&prev_scroll, x),
                                         offset_is_at_bound(&new_scroll, x));
                                    scroll_x = !prev_is_at_bound || !new_is_at_bound;
                                }
                            }

                            // Check whether the y axis is scrollable.
                            if y != 0.0 {
                                let new_scroll =
                                    State::update(ui, idx, &kid_area,
                                                  maybe_y_scroll, y);
                                if let Some(prev_scroll) = maybe_y_scroll {
                                    let (prev_is_at_bound, new_is_at_bound) =
                                        (offset_is_at_bound(&prev_scroll, y),
                                         offset_is_at_bound(&new_scroll, y));
                                    scroll_y = !prev_is_at_bound || !new_is_at_bound;
                                }
                            }

                            // Create a `Scroll` event if either axis is scrollable.
                            if scroll_x || scroll_y {
                                let event = UiEvent::Scroll(Some(idx), Scroll {
                                    x,
                                    y,
                                    modifiers: ui.global_input.current.modifiers,
                                }).into();
                                ui.global_input.push_event(event);

                                // Now that we've scrolled the top, scrollable widget,
                                // we're done with the loop.
                                break;
                            }
                        }

                        // If no scrollable widgets could be scrolled, emit the event to
                        // the widget that currently captures the mouse.
                        if x != 0.0 || y != 0.0 {
                            let widget = ui.global_input.current.widget_capturing_mouse;
                            if let Some(idx) = widget {
                                if let Some(widget) = ui.widget_graph.widget(idx) {
                                    // Only create the event if the widget is not
                                    // scrollable, as the event would have already been
                                    // created within the above loop.
                                    if widget.maybe_x_scroll_state.is_none()
                                        && widget.maybe_y_scroll_state.is_none() {
                                        let scroll = Scroll {
                                            x,
                                            y,
                                            modifiers: ui.global_input.current.modifiers,
                                        };
                                        let event = UiEvent::Scroll(Some(idx), scroll);
                                        ui.global_input.push_event(event.into());
                                    }
                                }
                            }
                        }

                        // Now that there might be a different widget under the mouse, we
                        // must update the capturing state.
                        ui.track_widget_under_mouse_and_update_capturing();*/
                    },

                    _ => (),
                }
                None
            },

            Input::Text(string) => {
                let event = KeyboardEvent::Text(string, modifiers);
                self.add_event(WidgetEvent::Keyboard(event));

                // Create a `Text` event.
                /*let text = Text {
                    string: string,
                    modifiers: ui.global_input.current.modifiers,
                };
                let widget = ui.global_input.current.widget_capturing_keyboard;
                let text_event = UiEvent::Text(widget, text).into();
                ui.global_input.push_event(text_event);*/
                None
            },

            Input::Touch(touch) => match touch.phase {
                /*Phase::Start => {
                    // Find the widget under the touch.
                    let widget_under_touch =
                        graph::algo::pick_widgets(&ui.depth_order.indices, touch.xy)
                            .next(&ui.widget_graph, &ui.depth_order.indices, &ui.theme);

                    // The start of the touch interaction state to be stored.
                    let start = Start {
                        time: instant::Instant::now(),
                        xy: touch.xy,
                        widget: widget_under_touch,
                    };

                    // The touch interaction state to be stored in the map.
                    let state = Touch {
                        start,
                        xy: touch.xy,
                        widget: widget_under_touch,
                    };

                    // Insert the touch state into the map.
                    ui.global_input.current.touch.insert(touch.id, state);

                    // Push touch event.
                    let event = UiEvent::Touch(widget_under_touch, touch);
                    ui.global_input.push_event(event.into());

                    // Push capture event.
                    if let Some(widget) = widget_under_touch {
                        let source = Source::Touch(touch.id);
                        let event = UiEvent::WidgetCapturesInputSource(widget, source);
                        ui.global_input.push_event(event.into());
                    }
                },

                Phase::Move => {

                    // Update the widget under the touch and return the widget capturing the touch.
                    let widget = match ui.global_input.current.touch.get_mut(&touch.id) {
                        Some(touch_state) => {
                            touch_state.widget =
                                graph::algo::pick_widgets(&ui.depth_order.indices, touch.xy)
                                    .next(&ui.widget_graph,
                                          &ui.depth_order.indices,
                                          &ui.theme);
                            touch_state.xy = touch.xy;
                            touch_state.start.widget
                        },
                        None => None,
                    };
                    let event = UiEvent::Touch(widget, touch);
                    ui.global_input.push_event(event.into());
                },

                Phase::Cancel => {
                    let widget = ui.global_input.current.touch.remove(&touch.id).and_then(|t| t.start.widget);
                    let event = UiEvent::Touch(widget, touch);
                    ui.global_input.push_event(event.into());

                    // Generate an "uncaptures" event if necessary.
                    if let Some(widget) = widget {
                        let source = Source::Touch(touch.id);
                        let event = UiEvent::WidgetUncapturesInputSource(widget, source);
                        ui.global_input.push_event(event.into());
                    }
                },

                Phase::End => {
                    let old_touch = ui.global_input.current.touch.remove(&touch.id).map(|touch| touch);
                    let widget_capturing = old_touch.as_ref().and_then(|touch| touch.start.widget);
                    let event = UiEvent::Touch(widget_capturing, touch);
                    ui.global_input.push_event(event.into());

                    // Create a `Tap` event.
                    //
                    // If the widget at the end of the touch is the same as the widget at the start
                    // of the touch, that widget receives the `Tap`.
                    let tapped_widget =
                        graph::algo::pick_widgets(&ui.depth_order.indices, touch.xy)
                            .next(&ui.widget_graph, &ui.depth_order.indices, &ui.theme)
                            .and_then(|widget| match Some(widget) == widget_capturing {
                                true => Some(widget),
                                false => None,
                            });
                    let tap = Tap { id: touch.id, xy: touch.xy };
                    let event = UiEvent::Tap(tapped_widget, tap);
                    ui.global_input.push_event(event.into());

                    // Generate an "uncaptures" event if necessary.
                    if let Some(widget) = widget_capturing {
                        let source = Source::Touch(touch.id);
                        let event = UiEvent::WidgetUncapturesInputSource(widget, source);
                        ui.global_input.push_event(event.into());
                    }
                },*/
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