use std;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;

use instant::Instant;

use crate::{color, cursor};
use crate::color::Color;
use crate::event::event::Event;
use crate::event::input::Input;
use crate::event_handler::{EventHandler, KeyboardEvent, WidgetEvent, WindowEvent};
use crate::focus::{Focusable, Refocus};
use crate::input::{Key, ModifierKey};
use crate::position::Dimensions;
use crate::prelude::Environment;
use crate::prelude::EnvironmentColor;
use crate::prelude::EnvironmentFontSize;
use crate::prelude::EnvironmentVariable;
use crate::render::cprimitives::CPrimitives;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::widget::primitive::Widget;
use crate::widget::Rectangle;

/// `Ui` is the most important type within carbide and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their widget::Id.
/// * Stores rendering state for each widget until the end of each render cycle.
/// * Contains the theme used for default styling of the widgets.
/// * Maintains the latest user input state (for mouse and keyboard).
/// * Maintains the latest window dimensions.
#[derive(Debug)]
pub struct Ui<GS> where GS: GlobalStateContract {
    num_redraw_frames: u8,
    /// Whether or not the `Ui` needs to be re-drawn to screen.
    redraw_count: AtomicUsize,
    /// A background color to clear the screen with before drawing if one was given.
    maybe_background_color: Option<Color>,

    /// Mouse cursor
    mouse_cursor: cursor::MouseCursor,

    pub widgets: Box<dyn Widget<GS>>,
    event_handler: EventHandler,
    pub environment: Environment<GS>,
    any_focus: bool,
}


/// Each time carbide is required to redraw the GUI, it must draw for at least the next three frames
/// to ensure that, in the case that graphics buffers are being swapped, we have filled each
/// buffer. Otherwise if we don't draw into each buffer, we will probably be subject to flickering.
pub const SAFE_REDRAW_COUNT: u8 = 3;

impl<GS: GlobalStateContract> Ui<GS> {
    /// A new, empty **Ui**.
    pub fn new(window_pixel_dimensions: Dimensions, scale_factor: f64) -> Self {
        let dark_system_colors = vec![
            EnvironmentVariable::Color { key: EnvironmentColor::Blue, value: color::rgba_bytes(10, 132, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Green, value: color::rgba_bytes(48, 209, 88, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Indigo, value: color::rgba_bytes(94, 92, 230, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Orange, value: color::rgba_bytes(255, 149, 10, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Pink, value: color::rgba_bytes(255, 55, 95, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Purple, value: color::rgba_bytes(191, 90, 242, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Red, value: color::rgba_bytes(255, 69, 58, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Teal, value: color::rgba_bytes(100, 210, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Yellow, value: color::rgba_bytes(255, 214, 10, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray, value: color::rgba_bytes(142, 142, 147, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray2, value: color::rgba_bytes(99, 99, 102, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray3, value: color::rgba_bytes(72, 72, 74, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray4, value: color::rgba_bytes(58, 58, 60, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray5, value: color::rgba_bytes(44, 44, 46, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray6, value: color::rgba_bytes(28, 28, 30, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SystemBackground, value: color::rgba_bytes(28, 28, 30, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SecondarySystemBackground, value: color::rgba_bytes(44, 44, 46, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::TertiarySystemBackground, value: color::rgba_bytes(58, 58, 60, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Label, value: color::rgba_bytes(255, 255, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SecondaryLabel, value: color::rgba_bytes(152, 152, 159, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::TertiaryLabel, value: color::rgba_bytes(90, 90, 95, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::QuaternaryLabel, value: color::rgba_bytes(65, 65, 69, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::PlaceholderText, value: color::rgba_bytes(71, 71, 74, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Link, value: color::rgba_bytes(9, 132, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SystemFill, value: color::rgba_bytes(61, 61, 65, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SecondarySystemFill, value: color::rgba_bytes(57, 57, 61, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::TertiarySystemFill, value: color::rgba_bytes(50, 50, 54, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::QuaternarySystemFill, value: color::rgba_bytes(44, 44, 48, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::OpaqueSeparator, value: color::rgba_bytes(61, 61, 65, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Separator, value: color::rgba_bytes(255, 255, 255, 0.15) },
            EnvironmentVariable::Color { key: EnvironmentColor::Accent, value: color::rgba_bytes(10, 132, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::LightText, value: color::rgba_bytes(0, 0, 0, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::DarkText, value: color::rgba_bytes(255, 255, 255, 1.0) },
        ];

        let _light_system_colors = vec![
            EnvironmentVariable::Color { key: EnvironmentColor::Blue, value: color::rgba_bytes(0, 122, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Green, value: color::rgba_bytes(52, 199, 89, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Indigo, value: color::rgba_bytes(88, 86, 214, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Orange, value: color::rgba_bytes(255, 149, 0, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Pink, value: color::rgba_bytes(255, 45, 85, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Purple, value: color::rgba_bytes(175, 82, 222, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Red, value: color::rgba_bytes(255, 59, 48, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Teal, value: color::rgba_bytes(90, 200, 250, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Yellow, value: color::rgba_bytes(255, 204, 0, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray, value: color::rgba_bytes(142, 142, 147, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray2, value: color::rgba_bytes(174, 174, 178, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray3, value: color::rgba_bytes(199, 199, 204, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray4, value: color::rgba_bytes(209, 209, 214, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray5, value: color::rgba_bytes(229, 229, 234, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Gray6, value: color::rgba_bytes(242, 242, 247, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SystemBackground, value: color::rgba_bytes(255, 255, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SecondarySystemBackground, value: color::rgba_bytes(242, 242, 247, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::TertiarySystemBackground, value: color::rgba_bytes(255, 255, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Label, value: color::rgba_bytes(0, 0, 0, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SecondaryLabel, value: color::rgba_bytes(138, 138, 142, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::TertiaryLabel, value: color::rgba_bytes(196, 196, 198, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::QuaternaryLabel, value: color::rgba_bytes(220, 220, 221, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::PlaceholderText, value: color::rgba_bytes(196, 196, 198, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Link, value: color::rgba_bytes(0, 122, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SystemFill, value: color::rgba_bytes(228, 228, 230, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::SecondarySystemFill, value: color::rgba_bytes(233, 233, 235, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::TertiarySystemFill, value: color::rgba_bytes(239, 239, 240, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::QuaternarySystemFill, value: color::rgba_bytes(244, 244, 245, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::OpaqueSeparator, value: color::rgba_bytes(220, 220, 222, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::Separator, value: color::rgba_bytes(0, 0, 0, 0.137) },
            EnvironmentVariable::Color { key: EnvironmentColor::Accent, value: color::rgba_bytes(0, 122, 255, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::LightText, value: color::rgba_bytes(0, 0, 0, 1.0) },
            EnvironmentVariable::Color { key: EnvironmentColor::DarkText, value: color::rgba_bytes(255, 255, 255, 1.0) },
        ];

        let font_sizes_large = vec![
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::LargeTitle, value: 34 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Title, value: 28 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Title2, value: 22 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Title3, value: 20 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Headline, value: 17 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Body, value: 17 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Callout, value: 16 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Subhead, value: 15 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Footnote, value: 13 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Caption, value: 12 },
            EnvironmentVariable::FontSize { key: EnvironmentFontSize::Caption2, value: 11 },
        ];

        let base_environment = dark_system_colors.iter().chain(font_sizes_large.iter()).map(|item| item.clone()).collect::<Vec<_>>();

        let environment = Environment::new(base_environment, window_pixel_dimensions, scale_factor);

        Ui {
            widgets: Rectangle::initialize(vec![]),
            num_redraw_frames: SAFE_REDRAW_COUNT,
            redraw_count: AtomicUsize::new(SAFE_REDRAW_COUNT as usize),
            maybe_background_color: None,
            mouse_cursor: cursor::MouseCursor::Arrow,
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


    pub fn handle_event(&mut self, event: Input, _: &mut GS) {
        let window_event = self.event_handler.handle_event(event, self.environment.get_corrected_dimensions());

        //let mut _needs_redraw = self.delegate_events(global_state);

        match window_event {
            None => (),
            Some(event) => {
                match event {
                    WindowEvent::Resize(pixel_dimensions) => {
                        self.set_window_width(pixel_dimensions[0]);
                        self.set_window_height(pixel_dimensions[1]);
                        //_needs_redraw = true;
                    }
                    WindowEvent::Focus => (),//_needs_redraw = true,
                    WindowEvent::UnFocus => (),
                    WindowEvent::Redraw => (),//_needs_redraw = true,
                }
            }
        }

        //if _needs_redraw {
        //    self.draw()
        //}
    }

    pub fn delegate_events(&mut self, global_state: &GlobalStateContainer<GS>) -> bool {
        let now = Instant::now();
        let events = self.event_handler.get_events();

        for event in events {
            match event {
                WidgetEvent::Mouse(mouse_event) => {
                    let consumed = false;
                    self.widgets.process_mouse_event(mouse_event, &consumed, &mut self.environment, global_state);
                }
                WidgetEvent::Keyboard(keyboard_event) => {
                    self.widgets.process_keyboard_event(keyboard_event, &mut self.environment, global_state);
                }
                WidgetEvent::Window(_) => {
                    self.widgets.process_other_event(event, &mut self.environment, global_state);
                }
                WidgetEvent::Touch(_) => {
                    self.widgets.process_other_event(event, &mut self.environment, global_state);
                }
            }

            if let Some(request) = self.environment.focus_request.clone() {
                match request {
                    Refocus::FocusRequest => {
                        println!("Process focus request");
                        self.any_focus = self.widgets.process_focus_request(event, &request, &mut self.environment, global_state);
                    }
                    Refocus::FocusNext => {
                        println!("Focus next");
                        let focus_first = self.widgets.process_focus_next(event, &request, false, &mut self.environment, global_state);
                        if focus_first {
                            println!("Focus next back to first");
                            self.widgets.process_focus_next(event, &request, true, &mut self.environment, global_state);
                        }
                    }
                    Refocus::FocusPrevious => {
                        let focus_last = self.widgets.process_focus_previous(event, &request, false, &mut self.environment, global_state);
                        if focus_last {
                            self.widgets.process_focus_previous(event, &request, true, &mut self.environment, global_state);
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
                                self.any_focus = !self.widgets.process_focus_previous(event, &Refocus::FocusPrevious, true, &mut self.environment, global_state);
                            } else if modifier == &carbide_core::input::ModifierKey::NO_MODIFIER {
                                self.any_focus = !self.widgets.process_focus_next(event, &Refocus::FocusNext, true, &mut self.environment, global_state);
                            }
                        }
                    }
                    _ => {}
                }
            }


            self.environment.clear();

            // Todo check if this can be removed. It is for the overlay layer to have the same position
            // as the thing below. This will not work if the thing below the overlay layers, position is
            // dependent on some state that has not been synchronized. For a use case look at the pop up
            // button in controls.
            self.widgets.calculate_size(self.environment.get_corrected_dimensions(), &mut self.environment);
            self.widgets.position_children();

            //self.widgets.sync_state(&mut self.environment, global_state);


            self.environment.clear();
        }


        self.event_handler.clear_events();

        if now.elapsed().as_millis() > 16 {
            println!("Frame took: {}", now.elapsed().as_secs_f32());
        }


        // Todo: Determine if an redraw is needed after events are processed
        return true;
    }

    /// Draw the `Ui` in it's current state.
    ///
    /// NOTE: If you don't need to redraw your carbide GUI every frame, it is recommended to use the
    /// `Ui::draw_if_changed` method instead.
    pub fn draw(&mut self, global_state: &GlobalStateContainer<GS>) -> CPrimitives {
        let corrected_dimensions = self.environment.get_corrected_dimensions();

        CPrimitives::new(corrected_dimensions, &mut self.widgets, &mut self.environment, global_state)
    }

    /// Get mouse cursor state.
    pub fn mouse_cursor(&self) -> cursor::MouseCursor {
        self.mouse_cursor
    }
}