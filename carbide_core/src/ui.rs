use std;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;

use crate::color::Color;
use crate::cursor;
use crate::event::event::Event;
use crate::event_handler::{EventHandler, WidgetEvent, WindowEvent, KeyboardEvent};
use crate::position::Dimensions;
use crate::render::cprimitives::CPrimitives;
use crate::state::environment::Environment;
use crate::text;
use crate::widget::Rectangle;
use crate::widget::primitive::Widget;
use crate::state::global_state::GlobalState;
use crate::event::input::Input;
use instant::Instant;
use crate::focus::{Refocus, Focusable};
use crate::input::{Key, ModifierKey};

/// A constructor type for building a `Ui` instance with a set of optional parameters.
pub struct UiBuilder {
    /// The initial dimensions of the window in which the `Ui` exists.
    pub window_dimensions: Dimensions,
    /// The theme used to set default styling for widgets.
    ///
    /// If this field is `None` when `build` is called, `Theme::default` will be used.
    /// An estimation of the maximum number of widgets that will be used with this `Ui` instance.
    ///
    /// This value is used to determine the size with which various collections should be
    /// reserved. This may make the first cycle of widget instantiations more efficient as the
    /// collections will not be required to grow dynamically. These collections include:
    ///
    /// - the widget graph node and edge `Vec`s
    /// - the `HashSet` used to track updated widgets
    /// - the widget `DepthOrder` (a kind of toposort describing the order of widgets in their
    /// rendering order).
    ///
    /// If this field is `None` when `build` is called, these collections will be initialised with
    /// no pre-reserved size and will instead grow organically as needed.
    pub maybe_widgets_capacity: Option<usize>
}




/// `Ui` is the most important type within carbide and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their widget::Id.
/// * Stores rendering state for each widget until the end of each render cycle.
/// * Contains the theme used for default styling of the widgets.
/// * Maintains the latest user input state (for mouse and keyboard).
/// * Maintains the latest window dimensions.
#[derive(Debug)]
pub struct Ui<S> where S: GlobalState {

    /// Manages all fonts that have been loaded by the user.
    pub fonts: text::font::Map,

    num_redraw_frames: u8,
    /// Whether or not the `Ui` needs to be re-drawn to screen.
    redraw_count: AtomicUsize,
    /// A background color to clear the screen with before drawing if one was given.
    maybe_background_color: Option<Color>,

    /// Mouse cursor
    mouse_cursor: cursor::MouseCursor,

    // TODO: Remove the following fields as they should now be handled by `input::Global`.

    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,


    pub widgets: Box<dyn Widget<S>>,
    event_handler: EventHandler,
    pub environment: Environment<S>,
    any_focus: bool,
}

/// A wrapper around the `Ui` that restricts the user from mutating the `Ui` in certain ways while
/// in the scope of the `Ui::set_widgets` function and within `Widget`s' `update` methods. Using
/// the `UiCell`, users may access the `Ui` immutably (via `Deref`) however they wish, however they
/// may only mutate the `Ui` via the `&mut self` methods provided by the `UiCell`.
///
/// The name came from its likening to a "jail cell for the `Ui`", as it restricts a user's access
/// to it. However, we realise that the name may also cause ambiguity with the std `Cell` and
/// `RefCell` render (which `UiCell` has nothing to do with). Thus, if you have a better name for
/// this type in mind, please let us know at the github repo via an issue or PR sometime before we
/// hit 1.0.0!
#[derive(Debug)]
pub struct UiCell<'a, S: GlobalState> {
    /// A mutable reference to a **Ui**.
    ui: &'a mut Ui<S>,
}


/// Each time carbide is required to redraw the GUI, it must draw for at least the next three frames
/// to ensure that, in the case that graphics buffers are being swapped, we have filled each
/// buffer. Otherwise if we don't draw into each buffer, we will probably be subject to flickering.
pub const SAFE_REDRAW_COUNT: u8 = 3;

impl UiBuilder {

    /// Begin building a new `Ui` instance.
    ///
    /// Give the initial dimensions of the window within which the `Ui` will be instantiated as a
    /// `Scalar` (DPI agnostic) value.
    pub fn new(window_dimensions: Dimensions) -> Self {
        UiBuilder {
            window_dimensions: window_dimensions,
            maybe_widgets_capacity: None
        }
    }

    /// An estimation of the maximum number of widgets that will be used with this `Ui` instance.
    ///
    /// This value is used to determine the size with which various collections should be
    /// reserved. This may make the first cycle of widget instantiations more efficient as the
    /// collections will not be required to grow dynamically. These collections include:
    ///
    /// - the widget graph node and edge `Vec`s
    /// - the `HashSet` used to track updated widgets
    /// - the widget `DepthOrder` (a kind of toposort describing the order of widgets in their
    /// rendering order).
    ///
    /// If this field is `None` when `build` is called, these collections will be initialised with
    /// no pre-reserved size and will instead grow organically as required.
    pub fn widgets_capacity(mut self, value: usize) -> Self {
        self.maybe_widgets_capacity = Some(value);
        self
    }

    /// Build **Ui** from the given builder
    pub fn build<S: GlobalState>(self) -> Ui<S> {
        Ui::new(self)
    }

}

impl<S: GlobalState> Ui<S> {

    /// A new, empty **Ui**.
    fn new(builder: UiBuilder) -> Self {

        let UiBuilder {
            window_dimensions,
            ..
        } = builder;


        Ui {
            fonts: text::font::Map::new(),
            widgets: Rectangle::initialize(vec![]),
            win_w: window_dimensions[0],
            win_h: window_dimensions[1],
            num_redraw_frames: SAFE_REDRAW_COUNT,
            redraw_count: AtomicUsize::new(SAFE_REDRAW_COUNT as usize),
            maybe_background_color: None,
            mouse_cursor: cursor::MouseCursor::Arrow,
            event_handler: EventHandler::new(),
            environment: Environment::new(),
            any_focus: false,
        }
    }


    pub fn handle_event(&mut self, event: Input, global_state: &mut S) {
        let window_event = self.event_handler.handle_event(event, [self.win_w, self.win_h]);

        let mut _needs_redraw = self.delegate_events(global_state);

        match window_event {
            None => (),
            Some(event) => {
                match event {
                    WindowEvent::Resize(dimensions) => {
                        self.win_w = dimensions[0];
                        self.win_h = dimensions[1];
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

    fn delegate_events(&mut self, global_state: &mut S) -> bool {
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
                        let focus_first = self.widgets.process_focus_next(event, &request,false, &mut self.environment, global_state);
                        if focus_first {
                            self.widgets.process_focus_next(event, &request,true, &mut self.environment, global_state);
                        }
                    }
                    Refocus::FocusPrevious => {
                        let focus_last = self.widgets.process_focus_previous(event,&request, false, &mut self.environment, global_state);
                        if focus_last {
                            self.widgets.process_focus_previous(event, &request,true, &mut self.environment, global_state);
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
                                self.any_focus = !self.widgets.process_focus_previous(event, &Refocus::FocusPrevious,true, &mut self.environment, global_state);
                            } else if modifier == &carbide_core::input::ModifierKey::NO_MODIFIER {
                                self.any_focus = !self.widgets.process_focus_next(event, &Refocus::FocusNext,true, &mut self.environment, global_state);
                            }
                        }
                    }
                    _ => {}
                }
            }

        }

        self.environment.clear();

        self.widgets.sync_state(&mut self.environment, global_state);

        self.environment.clear();
        self.event_handler.clear_events();

        if now.elapsed().as_millis() > 16 {
            println!("Frame took: {}", now.elapsed().as_secs_f32());
        }

        // Todo: Determine if an redraw is needed after events are processed
        return true
    }

    /// Draw the `Ui` in it's current state.
    ///
    /// NOTE: If you don't need to redraw your carbide GUI every frame, it is recommended to use the
    /// `Ui::draw_if_changed` method instead.
    pub fn draw(&mut self) -> CPrimitives {
        let Ui {
            ref mut widgets,
            win_w, win_h,
            ref mut environment,
            ..
        } = *self;

        CPrimitives::new([win_w, win_h], widgets, environment)
    }

    /// Get mouse cursor state.
    pub fn mouse_cursor(&self) -> cursor::MouseCursor {
        self.mouse_cursor
    }
}


impl<'a, S: GlobalState> UiCell<'a, S> {

    /// A convenience method for borrowing the `Font` for the given `Id` if it exists.
    pub fn font(&self, id: text::font::Id) -> Option<&text::Font> {
        self.ui.fonts.get(id)
    }

    /// Returns the dimensions of the window
    pub fn window_dim(&self) -> Dimensions {
        [self.ui.win_w, self.ui.win_h]
    }

    /// Sets the mouse cursor
    pub fn set_mouse_cursor(&mut self, cursor: cursor::MouseCursor) {
        self.ui.mouse_cursor = cursor;
    }
}

impl<'a, S: GlobalState> ::std::ops::Deref for UiCell<'a, S> {
    type Target = Ui<S>;
    fn deref(&self) -> &Ui<S> {
        self.ui
    }
}

impl<'a, S: GlobalState> AsRef<Ui<S>> for UiCell<'a, S> {
    fn as_ref(&self) -> &Ui<S> {
        &self.ui
    }
}
