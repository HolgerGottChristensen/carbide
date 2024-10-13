use std::collections::HashMap;
use std::time::{Duration, Instant};
use accesskit::{NodeId, TreeUpdate};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::Key;
use winit::window::{Theme, WindowId};
use carbide_core::accessibility::AccessibilityContext;
use carbide_core::asynchronous::{AsyncContext, check_tasks};
use carbide_core::cursor::MouseCursor;
use carbide_core::draw::{Dimension, InnerImageContext, Position, Scalar};
use carbide_core::environment::Environment;
use carbide_core::event::{AccessibilityEvent, AccessibilityEventContext, EventId, KeyboardEvent, KeyboardEventContext, ModifierKey, MouseEvent, MouseEventContext, OtherEventContext, WindowEventContext};
use carbide_core::event::Event::CoreEvent;
use carbide_core::focus::{FocusContext, Refocus};
use carbide_core::render::{NoopRenderContext, RenderContext};
use carbide_core::Scene;
use carbide_core::text::InnerTextContext;
use carbide_core::widget::WidgetId;
use crate::{convert_key, convert_mouse_button, convert_touch_phase};
use crate::custom_event::CustomEvent;

const N_CLICK_THRESHOLD: Duration = Duration::from_millis(500);
const MOUSE_CLICK_MAX_DISTANCE: f64 = 3.0;
const ARBITRARY_POINTS_PER_LINE_FACTOR: f64 = 10.0;

static SCALE_FACTORS: Lazy<DashMap<WindowId, Scalar>> = Lazy::new(|| DashMap::new());

pub fn update_scale_factor(window_id: WindowId, factor: Scalar) {
    SCALE_FACTORS.insert(window_id, factor);
}

pub fn scale_factor(window_id: WindowId) -> Scalar {
    *SCALE_FACTORS.get(&window_id).unwrap()
}

pub struct NewEventHandler {
    pressed_buttons: HashMap<MouseButton, (MouseEvent, Instant)>,
    modifiers: ModifierKey,
    last_click: Option<(Instant, MouseEvent)>,
    mouse_position: Position,
    event_id: u32,
}

impl NewEventHandler {
    pub fn new() -> NewEventHandler {
        NewEventHandler {
            pressed_buttons: Default::default(),
            modifiers: Default::default(),
            last_click: None,
            mouse_position: Default::default(),
            event_id: 0,
        }
    }

    pub fn next_id(&mut self) -> EventId {
        self.event_id += 1;
        EventId::new(self.event_id)
    }

    pub fn handle_refocus(target: &mut impl Scene, env: &mut Environment) {
        if let Some(request) = env.focus_request.clone() {
            match request {
                Refocus::FocusRequest => {
                    //println!("Process focus request");
                    target.process_focus_request(&mut FocusContext {
                        env,
                        focus_count: &mut 0,
                        available: &mut false,
                    });
                }
                Refocus::FocusNext => {
                    let mut count = 0;

                    //println!("Focus next");
                    target.process_focus_next(&mut FocusContext {
                        env,
                        focus_count: &mut count,
                        available: &mut false,
                    });

                    if count == 0 {
                        //println!("Focus next back to first");
                        target.process_focus_next(&mut FocusContext {
                            env,
                            focus_count: &mut 0,
                            available: &mut true,
                        });
                    }
                }
                Refocus::FocusPrevious => {
                    let mut count = 0;

                    //println!("Focus prev");
                    target.process_focus_previous(&mut FocusContext {
                        env,
                        focus_count: &mut count,
                        available: &mut false,
                    });

                    if count == 0 {
                        //println!("Focus prev forward to last");
                        target.process_focus_previous(&mut FocusContext {
                            env,
                            focus_count: &mut 0,
                            available: &mut true,
                        });
                    }
                }
            }
            env.focus_request = None;
        }
    }

    pub fn window_event(&mut self, event: WindowEvent, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        match event {
            WindowEvent::Moved(position) => {
                let logical_position = position.to_logical(scale_factor(window_id));

                target.process_window_event(&carbide_core::event::WindowEvent::Moved(Position::new(logical_position.x, logical_position.y)), &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });

                true
            }
            WindowEvent::Resized(new) => {
                let LogicalSize { width, height } = new.to_logical::<f64>(scale_factor(window_id));

                target.process_window_event(&carbide_core::event::WindowEvent::Resize(Dimension::new(width, height)), &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });

                true
            },
            WindowEvent::Focused(focus) => self.focus(focus, window_id, target, text_context, image_context, env),
            WindowEvent::KeyboardInput { event: winit::event::KeyEvent { logical_key, state, .. }, .. } => self.keyboard(logical_key, state, window_id, target, text_context, image_context, env),
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = ModifierKey::from_bits_retain(modifiers.state().bits());
                false
            }
            WindowEvent::Ime(ime) => self.ime(ime, window_id, target, text_context, image_context, env),
            WindowEvent::CursorMoved { position, .. } => self.cursor_moved(position, window_id, target, text_context, image_context, env),
            WindowEvent::MouseWheel { delta, .. } => self.mouse_wheel(delta, window_id, target, text_context, image_context, env),
            WindowEvent::MouseInput { state, button, .. } => self.mouse_input(button, state, window_id, target, text_context, image_context, env),
            WindowEvent::RedrawRequested => {
                target.render(&mut RenderContext {
                    render: &mut NoopRenderContext,
                    text: text_context,
                    image: image_context,
                    env,
                });

                // Set cursor to default for next frame
                env.set_cursor(MouseCursor::Default);

                // Check if there are any animations or requested animation frames
                let redraw = env.has_animations();

                // Clear a single animation frame
                env.clear_animation_frame();

                redraw
            }
            WindowEvent::CloseRequested => {
                target.process_window_event(&carbide_core::event::WindowEvent::CloseRequested, &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });

                false
            },
            WindowEvent::ActivationTokenDone { .. } => false,
            WindowEvent::Destroyed => false,
            WindowEvent::DroppedFile(_) => false,
            WindowEvent::HoveredFile(_) => false,
            WindowEvent::HoveredFileCancelled => false,
            WindowEvent::Occluded(_) => false,
            WindowEvent::AxisMotion { .. } => false,
            WindowEvent::Touch(_) => false,
            WindowEvent::TouchpadPressure { .. } => false,

            WindowEvent::CursorEntered { .. } => {
                let mut consumed = false;
                target.process_mouse_event(&MouseEvent::Entered, &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });
                true
            },
            WindowEvent::CursorLeft { .. } => {
                let mut consumed = false;
                target.process_mouse_event(&MouseEvent::Left, &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });
                true
            },
            WindowEvent::ThemeChanged(theme) => {
                target.process_window_event(&carbide_core::event::WindowEvent::ThemeChanged(match theme {
                    Theme::Light => carbide_core::event::Theme::Light,
                    Theme::Dark => carbide_core::event::Theme::Dark,
                }), &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });

                false
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                println!("ScaleFactorChanged!");

                update_scale_factor(window_id, scale_factor);

                target.process_window_event(&carbide_core::event::WindowEvent::ScaleFactorChanged(scale_factor), &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });

                false
            }

            WindowEvent::DoubleTapGesture { .. } => {
                let mut consumed = false;
                target.process_mouse_event(&MouseEvent::SmartScale(self.mouse_position), &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });
                true
            },
            WindowEvent::PinchGesture { phase, delta, .. } => {
                let mut consumed = false;
                target.process_mouse_event(&MouseEvent::Scale(delta, self.mouse_position, convert_touch_phase(phase)), &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });
                true
            },
            WindowEvent::RotationGesture { phase, delta, .. } => {
                let mut consumed = false;
                target.process_mouse_event(&MouseEvent::Rotation(delta as f64, self.mouse_position, convert_touch_phase(phase)), &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });
                true
            },
            WindowEvent::PanGesture { .. } => {
                false
            }
        }
    }

    pub fn mouse_input(&mut self, button: MouseButton, state: ElementState, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        match state {
            ElementState::Pressed => {
                let id = self.next_id();
                let now = Instant::now();

                let event = MouseEvent::Press {
                    id,
                    button: convert_mouse_button(button),
                    position: self.mouse_position,
                    modifiers: self.modifiers
                };

                let mut consumed = false;
                target.process_mouse_event(&event, &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });

                self.pressed_buttons.insert(button, (event, now));
            }
            ElementState::Released => {
                // The button is no longer pressed, so remove it from currently pressed buttons.
                let (pressed_event, pressed_time) = if let Some(event) = self.pressed_buttons.remove(&button) {
                    event
                } else {
                    println!("Mouse button release without mouse press??");
                    return false;
                };

                // Add the event that the user released the mouse button at the specified location holding the current modifiers.
                let event = MouseEvent::Release {
                    id: self.next_id(),
                    button: convert_mouse_button(button),
                    position: self.mouse_position,
                    modifiers: self.modifiers,
                    press_id: pressed_event.id(),
                    duration: Instant::now().duration_since(pressed_time),
                };
                let mut consumed = false;
                target.process_mouse_event(&event, &mut MouseEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    consumed: &mut consumed,
                });

                // A click should be emitted if within a threshold distance of the press.
                let is_click = self.mouse_position.dist(&pressed_event.get_current_mouse_position()) < MOUSE_CLICK_MAX_DISTANCE;

                // A click will become a double click, if and only if it is within the
                // double click threshold time based on the latest click of the same button.
                let click_number = match self.last_click {
                    // Too long since last click, so we emit a click and not a double click
                    Some((time, _)) if Instant::now().duration_since(time) > N_CLICK_THRESHOLD => 1,

                    // Our previous click was a normal click of the same button within the
                    // same location as the previous click. The time is checked in a previous case.
                    Some((_, MouseEvent::Click(b, location, _))) if convert_mouse_button(button) == b && self.mouse_position.dist(&location) < MOUSE_CLICK_MAX_DISTANCE => 2,

                    // Our previous click was a double click within time and with the same button
                    // and within range of the previous click.
                    Some((_, MouseEvent::NClick(b, location, _, n))) if convert_mouse_button(button) == b && self.mouse_position.dist(&location) < MOUSE_CLICK_MAX_DISTANCE => n + 1,

                    // Either the previous click was not in range or not with the same button
                    Some((_, _)) => 1,

                    // No previous click, so we emit a click and not a double click
                    None => 1,
                };

                if is_click {
                    if click_number == 1 {
                        let event = MouseEvent::Click(convert_mouse_button(button), self.mouse_position, self.modifiers);

                        let mut consumed = false;
                        target.process_mouse_event(&event, &mut MouseEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            window_id: &window_id.into(),
                            consumed: &mut consumed,
                        });
                        self.last_click = Some((Instant::now(), event));
                    } else {
                        let event = MouseEvent::NClick(convert_mouse_button(button), self.mouse_position, self.modifiers, click_number);

                        let mut consumed = false;
                        target.process_mouse_event(&event, &mut MouseEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            window_id: &window_id.into(),
                            consumed: &mut consumed,
                        });
                        self.last_click = Some((Instant::now(), event));
                    }
                } else {
                    self.last_click = None;
                }
            }
        }

        true
    }

    pub fn user_event(&mut self, event: CustomEvent, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        match event {
            CustomEvent::Core(core_event) => {
                check_tasks(&mut AsyncContext {
                    text: text_context,
                    image: image_context,
                    env,
                });

                target.process_other_event(&CoreEvent(core_event), &mut OtherEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                });
            }
            CustomEvent::Accessibility(accesskit_winit::Event { window_id, window_event}) => {
                println!("Accessibility Event: {:#?}", window_event);
                match window_event {
                    accesskit_winit::WindowEvent::InitialTreeRequested => {
                        env.full_accessibility_update = true;

                        target.process_accessibility(&mut AccessibilityContext {
                            env,
                            tree: &mut TreeUpdate {
                                nodes: vec![],
                                tree: None,
                                focus: NodeId(0),
                            },
                            parent_id: Default::default(),
                            children: &mut Default::default(),
                            hidden: false,
                        });

                        env.full_accessibility_update = false;
                    }
                    accesskit_winit::WindowEvent::ActionRequested(request) => {
                        target.process_accessibility_event(&AccessibilityEvent {
                            action: request.action,
                            target: WidgetId(request.target.0 as u32),
                            data: request.data,
                        }, &mut AccessibilityEventContext {
                            env,
                            window_id: &window_id.into(),
                            is_current: &false,
                        });
                    }
                    accesskit_winit::WindowEvent::AccessibilityDeactivated => {}
                }
            }
        }

        true
    }

    pub fn mouse_wheel(&mut self, delta: MouseScrollDelta, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        let (x, y) = match delta {
            MouseScrollDelta::PixelDelta(delta) => {
                let LogicalPosition { x, y } = delta.to_logical::<f64>(scale_factor(window_id));
                let x = x;
                let y = -y;

                (x, y)
            }
            MouseScrollDelta::LineDelta(x, y) => {
                // This should be configurable (we should provide a LineDelta event to allow for this).
                let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as carbide_core::draw::Scalar;
                let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as carbide_core::draw::Scalar;

                (x, y)
            }
        };

        let event = MouseEvent::Scroll {
            x,
            y,
            mouse_position: self.mouse_position,
            modifiers: self.modifiers,
        };

        target.process_mouse_event(&event, &mut MouseEventContext {
            text: text_context,
            image: image_context,
            env,
            is_current: &false,
            window_id: &window_id.into(),
            consumed: &mut false,
        });

        true
    }

    pub fn focus(&mut self, focus: bool, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        if focus {
            target.process_window_event(&carbide_core::event::WindowEvent::Focus, &mut WindowEventContext {
                text: text_context,
                image: image_context,
                env,
                is_current: &false,
                window_id: &window_id.into(),
            });
        } else {
            target.process_window_event(&carbide_core::event::WindowEvent::UnFocus, &mut WindowEventContext {
                text: text_context,
                image: image_context,
                env,
                is_current: &false,
                window_id: &window_id.into(),
            });
        }

        true
    }

    pub fn keyboard(&mut self, logical_key: Key, state: ElementState, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        let key = convert_key(&logical_key);
        let mut prevent_default = false;
        match state {
            ElementState::Pressed => {
                target.process_keyboard_event(&KeyboardEvent::Press(key.clone(), self.modifiers), &mut KeyboardEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    prevent_default: &mut prevent_default,
                });

                if !prevent_default {
                    if key == carbide_core::event::Key::Tab {
                        if self.modifiers.shift_key() {
                            //self.set_focus(Focus::FocusReleased);
                            env.request_focus(Refocus::FocusPrevious);
                        } else if self.modifiers.is_empty() {
                            //self.set_focus(Focus::FocusReleased);
                            env.request_focus(Refocus::FocusNext);
                        }
                    }
                }
            },
            ElementState::Released => {
                target.process_keyboard_event(&KeyboardEvent::Release(key, self.modifiers), &mut KeyboardEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    prevent_default: &mut prevent_default,
                });
            },
        };


        true
    }

    pub fn ime(&mut self, ime: Ime, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        match ime {
            Ime::Enabled => false,
            Ime::Preedit(s, cursor) => {
                target.process_keyboard_event(&KeyboardEvent::Ime(
                    carbide_core::event::Ime::PreEdit(s.to_string(), cursor.clone())
                ), &mut KeyboardEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    prevent_default: &mut false,
                });
                true
            },
            Ime::Commit(s) => {
                target.process_keyboard_event(&KeyboardEvent::Ime(
                    carbide_core::event::Ime::Commit(s.to_string())
                ), &mut KeyboardEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                    prevent_default: &mut false,
                });
                true
            },
            Ime::Disabled => false,
        }
    }

    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>, window_id: WindowId, target: &mut impl Scene, text_context: &mut impl InnerTextContext, image_context: &mut impl InnerImageContext, env: &mut Environment) -> bool {
        let last_mouse_xy = self.mouse_position;

        let LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor(window_id));
        self.mouse_position = Position::new(x, y);

        if last_mouse_xy == self.mouse_position {
            return false;
        }

        env.set_mouse_position(self.mouse_position);

        let delta_xy = self.mouse_position - last_mouse_xy;

        let move_event = MouseEvent::Move {
            from: last_mouse_xy,
            to: self.mouse_position,
            delta_xy,
            modifiers: self.modifiers,
        };

        let mut consumed = false;
        target.process_mouse_event(&move_event, &mut MouseEventContext {
            text: text_context,
            image: image_context,
            env,
            is_current: &false,
            window_id: &window_id.into(),
            consumed: &mut consumed,
        });

        // Check for drag events.

        let distance = (delta_xy.x + delta_xy.y).abs().sqrt();
        let drag_threshold = 0.0;

        if distance > drag_threshold {
            for (button, (evt, _)) in self.pressed_buttons.iter() {
                match evt {
                    MouseEvent::Press { position: location, .. } => {
                        let total_delta_xy = self.mouse_position - *location;
                        let drag_event = MouseEvent::Drag {
                            button: convert_mouse_button(*button),
                            origin: *location,
                            from: last_mouse_xy,
                            to: self.mouse_position,
                            delta_xy,
                            total_delta_xy,
                            modifiers: self.modifiers,
                        };

                        let mut consumed = false;
                        target.process_mouse_event(&drag_event, &mut MouseEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            window_id: &window_id.into(),
                            consumed: &mut consumed,
                        });
                    }
                    _ => {}
                }
            }
        }

        true
    }
}