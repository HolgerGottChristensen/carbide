use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use accesskit::{NodeId, TreeUpdate};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::Key;
#[cfg(not(target_arch = "wasm32"))]
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::WindowId;
use carbide_core::accessibility::AccessibilityContext;
use carbide_core::asynchronous::{AsyncContext, check_tasks};
use carbide_core::draw::{Dimension, ImageContext, Position, Scalar};
use carbide_core::environment::{Environment};
use carbide_core::event::{AccessibilityEvent, AccessibilityEventContext, EventId, KeyboardEvent, KeyboardEventContext, ModifierKey, MouseEvent, MouseEventContext, OtherEvent, OtherEventContext, OtherEventHandler, WindowEventContext};
use carbide_core::focus::{FocusContext, FocusManager, Refocus};
use carbide_core::render::{NoopRenderContext, RenderContext};
use carbide_core::scene::AnyScene;
use carbide_core::text::TextContext;
use carbide_core::time::*;
use carbide_core::widget::managers::{ShortcutManager, ShortcutPressed, ShortcutReleased};
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

#[derive(Copy, Clone, Debug)]
pub enum RequestRedraw {
    False,
    True,
    IfAnimationsRequested,
}

impl Add<RequestRedraw> for RequestRedraw {
    type Output = RequestRedraw;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RequestRedraw::True, _) => RequestRedraw::True,
            (_, RequestRedraw::True) => RequestRedraw::True,
            (_, RequestRedraw::IfAnimationsRequested) => RequestRedraw::IfAnimationsRequested,
            (RequestRedraw::IfAnimationsRequested, _) => RequestRedraw::IfAnimationsRequested,
            (_, _) => RequestRedraw::False,
        }
    }
}

impl AddAssign for RequestRedraw {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
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

    pub fn mouse_position(&self) -> Position {
        self.mouse_position
    }

    pub fn handle_refocus(target: &mut [Box<dyn AnyScene>], focus_manager: &mut FocusManager, env: &mut Environment) {
        if let Some(focus) = focus_manager.requested_focus() {
            for scene in target {
                if !scene.has_application_focus() {
                    continue;
                }

                match focus {
                    Refocus::FocusRequest => {
                        //println!("Process focus request");
                        scene.process_focus_request(&mut FocusContext {
                            env,
                            focus_count: &mut 0,
                            available: &mut false,
                        });
                    }
                    Refocus::FocusNext => {
                        let mut count = 0;

                        //println!("Focus next");
                        scene.process_focus_next(&mut FocusContext {
                            env,
                            focus_count: &mut count,
                            available: &mut false,
                        });

                        if count == 0 {
                            //println!("Focus next back to first");
                            scene.process_focus_next(&mut FocusContext {
                                env,
                                focus_count: &mut 0,
                                available: &mut true,
                            });
                        }
                    }
                    Refocus::FocusPrevious => {
                        let mut count = 0;

                        //println!("Focus prev");
                        scene.process_focus_previous(&mut FocusContext {
                            env,
                            focus_count: &mut count,
                            available: &mut false,
                        });

                        if count == 0 {
                            //println!("Focus prev forward to last");
                            scene.process_focus_previous(&mut FocusContext {
                                env,
                                focus_count: &mut 0,
                                available: &mut true,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn window_event<'a: 'b, 'b, 'c: 'a>(&'a mut self, event: &WindowEvent, window_id: WindowId, scenes: &'b mut [Box<dyn AnyScene>], text_context: &'a mut impl TextContext, image_context: &'a mut impl ImageContext, env: &mut Environment, id: WidgetId) -> RequestRedraw {
        match event {
            WindowEvent::Moved(position) => {
                let logical_position = position.to_logical(scale_factor(window_id));

                for scene in scenes.iter_mut() {
                    scene.process_window_event(&carbide_core::event::WindowEvent::Moved(Position::new(logical_position.x, logical_position.y)), &mut WindowEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                    });
                }

                RequestRedraw::True
            }
            WindowEvent::Resized(new) => {
                let LogicalSize { width, height } = new.to_logical::<f64>(scale_factor(window_id));

                for scene in scenes.iter_mut() {
                    scene.process_window_event(&carbide_core::event::WindowEvent::Resize(Dimension::new(width, height)), &mut WindowEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                    });
                }

                RequestRedraw::True
            },
            WindowEvent::Focused(focus) => self.focus(*focus, window_id, scenes, text_context, image_context, env),
            WindowEvent::KeyboardInput { event, .. } => {
                let winit::event::KeyEvent { logical_key, state, .. } = event;

                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.keyboard(logical_key.clone(), event.key_without_modifiers(), *state, window_id, scenes, text_context, image_context, env)
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.keyboard(logical_key.clone(), logical_key.clone(), *state, window_id, scenes, text_context, image_context, env)

                }
            },
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = ModifierKey::from_bits_retain(modifiers.state().bits());
                RequestRedraw::False
            }
            WindowEvent::Ime(ime) => self.ime(ime.clone(), window_id, scenes, text_context, image_context, env),
            WindowEvent::CursorMoved { position, .. } => self.cursor_moved(*position, window_id, scenes, text_context, image_context, env),
            WindowEvent::MouseWheel { delta, .. } => self.mouse_wheel(*delta, window_id, scenes, text_context, image_context, env),
            WindowEvent::MouseInput { state, button, .. } => self.mouse_input(*button, *state, window_id, scenes, text_context, image_context, env),
            WindowEvent::RedrawRequested => {
                for scene in scenes.iter_mut() {
                    scene.process_accessibility(&mut AccessibilityContext {
                        env,
                        nodes: &mut TreeUpdate {
                            nodes: vec![],
                            tree: None,
                            focus: NodeId(id.as_u32() as u64),
                        },
                        parent_id: None,
                        children: &mut Default::default(),
                        hidden: false,
                        inherited_label: None,
                        inherited_hint: None,
                        inherited_value: None,
                        inherited_enabled: None,
                    });

                    scene.render(&mut RenderContext {
                        render: &mut NoopRenderContext,
                        text: text_context,
                        image: image_context,
                        env,
                    });
                }


                // Check if there are any animations or requested animation frames
                RequestRedraw::IfAnimationsRequested
            }
            WindowEvent::CloseRequested => {
                for scene in scenes.iter_mut() {
                    scene.process_window_event(&carbide_core::event::WindowEvent::CloseRequested, &mut WindowEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                    });
                }

                RequestRedraw::False
            },
            WindowEvent::ActivationTokenDone { .. } => RequestRedraw::False,
            WindowEvent::Destroyed => RequestRedraw::False,
            WindowEvent::DroppedFile(_) => RequestRedraw::False,
            WindowEvent::HoveredFile(_) => RequestRedraw::False,
            WindowEvent::HoveredFileCancelled => RequestRedraw::False,
            WindowEvent::Occluded(_) => RequestRedraw::False,
            WindowEvent::AxisMotion { .. } => RequestRedraw::False,
            WindowEvent::Touch(_) => RequestRedraw::False,
            WindowEvent::TouchpadPressure { .. } => RequestRedraw::False,

            WindowEvent::CursorEntered { .. } => {
                let mut consumed = false;
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&MouseEvent::Entered, &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }
                RequestRedraw::True
            },
            WindowEvent::CursorLeft { .. } => {
                let mut consumed = false;
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&MouseEvent::Left, &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }
                RequestRedraw::True
            },
            WindowEvent::ThemeChanged(_theme) => {
                println!("Theme changed!");

                for scene in scenes.iter_mut() {
                    scene.process_window_event(&carbide_core::event::WindowEvent::ThemeChanged, &mut WindowEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                    });
                }

                RequestRedraw::True
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                println!("ScaleFactorChanged!");

                update_scale_factor(window_id, *scale_factor);

                for scene in scenes.iter_mut() {
                    scene.process_window_event(&carbide_core::event::WindowEvent::ScaleFactorChanged(*scale_factor), &mut WindowEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                    });
                }

                RequestRedraw::False
            }

            WindowEvent::DoubleTapGesture { .. } => {
                let mut consumed = false;
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&MouseEvent::SmartScale(self.mouse_position), &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }
                RequestRedraw::True
            },
            WindowEvent::PinchGesture { phase, delta, .. } => {
                let mut consumed = false;
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&MouseEvent::Scale(*delta, self.mouse_position, convert_touch_phase(*phase)), &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }
                RequestRedraw::True
            },
            WindowEvent::RotationGesture { phase, delta, .. } => {
                let mut consumed = false;
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&MouseEvent::Rotation(*delta as f64, self.mouse_position, convert_touch_phase(*phase)), &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }
                RequestRedraw::True
            },
            WindowEvent::PanGesture { .. } => {
                RequestRedraw::False
            }
        }
    }

    pub fn mouse_input(&mut self, button: MouseButton, state: ElementState, window_id: WindowId, scenes: &mut [Box<dyn AnyScene>], text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment) -> RequestRedraw {
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
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&event, &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }

                self.pressed_buttons.insert(button, (event, now));
            }
            ElementState::Released => {
                // The button is no longer pressed, so remove it from currently pressed buttons.
                let (pressed_event, pressed_time) = if let Some(event) = self.pressed_buttons.remove(&button) {
                    event
                } else {
                    println!("Mouse button release without mouse press??");
                    return RequestRedraw::False;
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
                for scene in scenes.iter_mut() {
                    scene.process_mouse_event(&event, &mut MouseEventContext {
                        text: text_context,
                        image: image_context,
                        is_current: &false,
                        window_id: &window_id.into(),
                        consumed: &mut consumed,
                        env,
                    });
                }

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
                        for scene in scenes.iter_mut() {
                            scene.process_mouse_event(&event, &mut MouseEventContext {
                                text: text_context,
                                image: image_context,
                                is_current: &false,
                                window_id: &window_id.into(),
                                consumed: &mut consumed,
                                env,
                            });
                        }
                        self.last_click = Some((Instant::now(), event));
                    } else {
                        let event = MouseEvent::NClick(convert_mouse_button(button), self.mouse_position, self.modifiers, click_number);

                        let mut consumed = false;
                        for scene in scenes.iter_mut() {
                            scene.process_mouse_event(&event, &mut MouseEventContext {
                                text: text_context,
                                image: image_context,
                                is_current: &false,
                                window_id: &window_id.into(),
                                consumed: &mut consumed,
                                env,
                            });
                        }
                        self.last_click = Some((Instant::now(), event));
                    }
                } else {
                    self.last_click = None;
                }
            }
        }

        RequestRedraw::True
    }

    pub fn user_event(&mut self, event: &CustomEvent, target: &mut impl AnyScene, text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment, id: WidgetId) -> RequestRedraw {
        match event {
            CustomEvent::Core(core_event) => {
                check_tasks(&mut AsyncContext {
                    text: text_context,
                    image: image_context,
                    env,
                });

                target.process_other_event(&OtherEvent::CoreEvent(*core_event), &mut OtherEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    is_consumed: &mut false,
                });
            }
            CustomEvent::Accessibility(accesskit_winit::Event { window_event, .. }) => {
                println!("Accessibility Event: {:#?}", window_event);
                match window_event {
                    accesskit_winit::WindowEvent::InitialTreeRequested => {
                        target.process_accessibility(&mut AccessibilityContext {
                            env,
                            nodes: &mut TreeUpdate {
                                nodes: vec![],
                                tree: None,
                                focus: NodeId(id.as_u32() as u64),
                            },
                            parent_id: None,
                            children: &mut Default::default(),
                            hidden: false,
                            inherited_label: None,
                            inherited_hint: None,
                            inherited_value: None,
                            inherited_enabled: None,
                        });
                    }
                    accesskit_winit::WindowEvent::ActionRequested(request) => {
                        target.process_accessibility_event(&AccessibilityEvent {
                            action: request.action,
                            target: WidgetId::from_u32(request.target.0 as u32),
                            data: &request.data,
                        }, &mut AccessibilityEventContext {
                            env,
                        });
                    }
                    accesskit_winit::WindowEvent::AccessibilityDeactivated => {}
                }
            }
        }

        RequestRedraw::True
    }

    pub fn mouse_wheel(&mut self, delta: MouseScrollDelta, window_id: WindowId, scenes: &mut [Box<dyn AnyScene>], text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment) -> RequestRedraw {
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

        for scene in scenes.iter_mut() {
            scene.process_mouse_event(&event, &mut MouseEventContext {
                text: text_context,
                image: image_context,
                is_current: &false,
                window_id: &window_id.into(),
                consumed: &mut false,
                env,
            });
        }


        RequestRedraw::True
    }

    pub fn focus(&mut self, focus: bool, window_id: WindowId, scenes: &mut [Box<dyn AnyScene>], text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment) -> RequestRedraw {
        if focus {
            for scene in scenes.iter_mut() {
                scene.process_window_event(&carbide_core::event::WindowEvent::Focus, &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });
            }
        } else {
            for scene in scenes.iter_mut() {
                scene.process_window_event(&carbide_core::event::WindowEvent::UnFocus, &mut WindowEventContext {
                    text: text_context,
                    image: image_context,
                    env,
                    is_current: &false,
                    window_id: &window_id.into(),
                });
            }
        }

        RequestRedraw::True
    }

    pub fn keyboard(&mut self, logical_key: Key, no_modifier_key: Key, state: ElementState, window_id: WindowId, scenes: &mut [Box<dyn AnyScene>], text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment) -> RequestRedraw {
        let key = convert_key(&logical_key);
        let no_modifier_key = convert_key(&no_modifier_key);

        let mut prevent_default = false;

        let mut shortcut_manager = ShortcutManager::new();

        match state {
            ElementState::Pressed => {
                env.with_mut::<ShortcutManager>(&mut shortcut_manager, |env| {
                    for scene in scenes.iter_mut() {
                        scene.process_keyboard_event(&KeyboardEvent::Press {
                            key: key.clone(),
                            modifiers: self.modifiers,
                            no_modifier_key: no_modifier_key.clone()
                        }, &mut KeyboardEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            window_id: &window_id.into(),
                            prevent_default: &mut prevent_default,
                        });
                    }
                });

                if let Some(shortcut) = shortcut_manager.has_shortcut() {
                    let event = OtherEvent::new(ShortcutPressed(shortcut));

                    for scene in scenes.iter_mut() {
                        scene.process_other_event(&event, &mut OtherEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            is_consumed: &mut false,
                        });
                    }

                    return RequestRedraw::True;
                }

                if !prevent_default {
                    if key == carbide_core::event::Key::Tab {
                        if self.modifiers.shift_key() {
                            //self.set_focus(Focus::FocusReleased);
                            FocusManager::get(env, |manager| {
                                manager.request_focus(Refocus::FocusPrevious)
                            });
                        } else if self.modifiers.is_empty() {
                            //self.set_focus(Focus::FocusReleased);
                            FocusManager::get(env, |manager| {
                                manager.request_focus(Refocus::FocusNext)
                            });
                        }
                    }
                }
            },
            ElementState::Released => {
                env.with_mut::<ShortcutManager>(&mut shortcut_manager, |env| {
                    for scene in scenes.iter_mut() {
                        scene.process_keyboard_event(&KeyboardEvent::Release {
                            key: key.clone(),
                            modifiers: self.modifiers,
                            no_modifier_key: no_modifier_key.clone()
                        }, &mut KeyboardEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            window_id: &window_id.into(),
                            prevent_default: &mut prevent_default,
                        });
                    }
                });

                if let Some(shortcut) = shortcut_manager.has_shortcut() {
                    let event = OtherEvent::new(ShortcutReleased(shortcut));

                    for scene in scenes.iter_mut() {
                        scene.process_other_event(&event, &mut OtherEventContext {
                            text: text_context,
                            image: image_context,
                            env,
                            is_current: &false,
                            is_consumed: &mut false,
                        });
                    }

                    return RequestRedraw::True;
                }
            },
        };


        RequestRedraw::True
    }

    pub fn ime(&mut self, ime: Ime, window_id: WindowId, scenes: &mut [Box<dyn AnyScene>], text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment) -> RequestRedraw {
        match ime {
            Ime::Enabled => RequestRedraw::False,
            Ime::Preedit(s, cursor) => {
                for scene in scenes.iter_mut() {
                    scene.process_keyboard_event(&KeyboardEvent::Ime(
                        carbide_core::event::Ime::PreEdit(s.to_string(), cursor.clone())
                    ), &mut KeyboardEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                        prevent_default: &mut false,
                    });
                }
                RequestRedraw::True
            },
            Ime::Commit(s) => {
                for scene in scenes.iter_mut() {
                    scene.process_keyboard_event(&KeyboardEvent::Ime(
                        carbide_core::event::Ime::Commit(s.to_string())
                    ), &mut KeyboardEventContext {
                        text: text_context,
                        image: image_context,
                        env,
                        is_current: &false,
                        window_id: &window_id.into(),
                        prevent_default: &mut false,
                    });
                }
                RequestRedraw::True
            },
            Ime::Disabled => RequestRedraw::False,
        }
    }

    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>, window_id: WindowId, scenes: &mut [Box<dyn AnyScene>], text_context: &mut impl TextContext, image_context: &mut impl ImageContext, env: &mut Environment) -> RequestRedraw {
        let last_mouse_xy = self.mouse_position;

        let LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor(window_id));
        self.mouse_position = Position::new(x, y);

        if last_mouse_xy == self.mouse_position {
            return RequestRedraw::False;
        }

        /*if let Some(position) = env.get_mut::<MousePositionKey>() {
            *position = self.mouse_position;
        }*/

        let delta_xy = self.mouse_position - last_mouse_xy;

        let move_event = MouseEvent::Move {
            from: last_mouse_xy,
            to: self.mouse_position,
            delta_xy,
            modifiers: self.modifiers,
        };

        let mut consumed = false;
        for scene in scenes.iter_mut() {
            scene.process_mouse_event(&move_event, &mut MouseEventContext {
                text: text_context,
                image: image_context,
                is_current: &false,
                window_id: &window_id.into(),
                consumed: &mut consumed,
                env,
            });
        }

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
                        for scene in scenes.iter_mut() {
                            scene.process_mouse_event(&drag_event, &mut MouseEventContext {
                                text: text_context,
                                image: image_context,
                                is_current: &false,
                                window_id: &window_id.into(),
                                consumed: &mut consumed,
                                env,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        RequestRedraw::True
    }
}