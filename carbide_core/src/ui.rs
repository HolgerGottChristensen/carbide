use carbide_core::platform::mac::menu::set_application_menu;
use std;
use std::ffi::c_void;
use std::fmt::Debug;
use std::time::Instant;

use crate::cursor::MouseCursor;
use crate::draw::{theme, Dimension};
use crate::event::{
    EventHandler, EventSink, Input, Key, KeyboardEvent, ModifierKey, OtherEventHandler,
    WidgetEvent, WindowEvent,
};
use crate::focus::{Focusable, Refocus};
use crate::prelude::Environment;
use crate::prelude::EnvironmentColor;
use crate::prelude::EnvironmentFontSize;
use crate::prelude::EnvironmentVariable;
use crate::render::Primitives;
use crate::widget::Widget;
use crate::widget::{Menu, Rectangle};
use crate::{color, cursor};

/// `Ui` is the most important type within carbide and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their widget::Id.
/// * Stores rendering state for each widget until the end of each render cycle.
/// * Contains the theme used for default styling of the widgets.
/// * Maintains the latest user input state (for mouse and keyboard).
/// * Maintains the latest window dimensions.
#[derive(Debug)]
pub struct Ui {
    pub widgets: Box<dyn Widget>,
    pub menu: Option<Vec<Menu>>,
    event_handler: EventHandler,
    pub environment: Environment,
    any_focus: bool,
}

impl Ui {
    /// A new, empty **Ui**.
    pub fn new(
        window_pixel_dimensions: Dimension,
        scale_factor: f64,
        window_handle: Option<*mut c_void>,
        event_sink: Box<dyn EventSink>,
    ) -> Self {


        let environment = Environment::new(
            window_pixel_dimensions,
            scale_factor,
            window_handle,
            event_sink,
        );

        Ui {
            widgets: Rectangle::new().fill(EnvironmentColor::SystemBackground),
            menu: None,
            event_handler: EventHandler::new(),
            environment,
            any_focus: false,
        }
    }

    pub fn set_window_width(&mut self, width: f64) {
        self.environment.set_pixel_width(width);
    }

    pub fn set_window_height(&mut self, height: f64) {
        self.environment.set_pixel_height(height);
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.environment.set_scale_factor(scale_factor);
    }

    #[cfg(target_os = "macos")]
    pub fn refresh_application_menu(&self) {
        if let Some(menu) = &self.menu {
            set_application_menu(menu);
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn refresh_application_menu(&self) {}

    pub fn has_animations(&self) -> bool {
        self.environment.has_animations()
    }

    pub fn has_queued_events(&self) -> bool {
        self.event_handler.get_events().len() > 0
    }

    pub fn compound_and_add_event(&mut self, event: Input) {
        let window_event = self
            .event_handler
            .compound_and_add_event(event);

        //let mut _needs_redraw = self.delegate_events(global_state);

        match window_event {
            None => (),
            Some(event) => {
                match event {
                    WindowEvent::Resize(pixel_dimensions) => {
                        self.set_window_width(pixel_dimensions.width);
                        self.set_window_height(pixel_dimensions.height);
                        //_needs_redraw = true;
                    }
                    WindowEvent::Focus => (), //_needs_redraw = true,
                    WindowEvent::UnFocus => (),
                    WindowEvent::Redraw => (), //_needs_redraw = true,
                    WindowEvent::CloseRequested => {}
                }
            }
        }

        //if _needs_redraw {
        //    self.draw()
        //}
    }

    pub fn delegate_events(&mut self) -> bool {
        let now = Instant::now();
        let events = self.event_handler.get_events();

        self.environment.set_cursor(MouseCursor::Arrow);

        for event in events {
            self.environment.capture_time();
            match event {
                WidgetEvent::Mouse(mouse_event) => {
                    let consumed = false;
                    self.widgets
                        .process_mouse_event(mouse_event, &consumed, &mut self.environment);
                }
                WidgetEvent::Keyboard(keyboard_event) => {
                    self.widgets
                        .process_keyboard_event(keyboard_event, &mut self.environment);
                }
                WidgetEvent::Window(_) => {
                    self.widgets
                        .process_other_event(event, &mut self.environment);
                }
                WidgetEvent::Touch(_) => {
                    self.widgets
                        .process_other_event(event, &mut self.environment);
                }
                WidgetEvent::Custom(_) => {}
                WidgetEvent::DoneProcessingEvents => {
                    self.widgets
                        .process_other_event(event, &mut self.environment);
                }
            }

            if let Some(request) = self.environment.focus_request.clone() {
                match request {
                    Refocus::FocusRequest => {
                        println!("Process focus request");
                        self.any_focus = self.widgets.process_focus_request(
                            event,
                            &request,
                            &mut self.environment,
                        );
                    }
                    Refocus::FocusNext => {
                        println!("Focus next");
                        let focus_first = self.widgets.process_focus_next(
                            event,
                            &request,
                            false,
                            &mut self.environment,
                        );
                        if focus_first {
                            println!("Focus next back to first");
                            self.widgets.process_focus_next(
                                event,
                                &request,
                                true,
                                &mut self.environment,
                            );
                        }
                    }
                    Refocus::FocusPrevious => {
                        let focus_last = self.widgets.process_focus_previous(
                            event,
                            &request,
                            false,
                            &mut self.environment,
                        );
                        if focus_last {
                            self.widgets.process_focus_previous(
                                event,
                                &request,
                                true,
                                &mut self.environment,
                            );
                        }
                    }
                }
                self.environment.focus_request = None;
            } else if !self.any_focus {
                match event {
                    WidgetEvent::Keyboard(KeyboardEvent::Press(key, modifier)) => {
                        if key == &Key::Tab {
                            if modifier == &ModifierKey::SHIFT {
                                // If focus is still up for grab we can assume that no element
                                // has been focused. This assumption breaks if there can be multiple
                                // widgets with focus at the same time
                                self.any_focus = !self.widgets.process_focus_previous(
                                    event,
                                    &Refocus::FocusPrevious,
                                    true,
                                    &mut self.environment,
                                );
                            } else if modifier == &ModifierKey::NO_MODIFIER {
                                self.any_focus = !self.widgets.process_focus_next(
                                    event,
                                    &Refocus::FocusNext,
                                    true,
                                    &mut self.environment,
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
        self.widgets
            .process_other_event(&WidgetEvent::DoneProcessingEvents, &mut self.environment);
        self.event_handler.clear_events();

        if now.elapsed().as_millis() > 16 {
            println!("Frame took: {}", now.elapsed().as_secs_f32());
        }

        // Todo: Determine if an redraw is needed after events are processed
        return true;
    }

    pub fn draw(&mut self) -> Primitives {
        let corrected_dimensions = self.environment.get_corrected_dimensions();
        self.environment.capture_time();

        Primitives::new(
            corrected_dimensions,
            &mut self.widgets,
            &mut self.environment,
        )
    }

    /// Get mouse cursor state.
    pub fn mouse_cursor(&self) -> cursor::MouseCursor {
        self.environment.cursor()
    }
}
