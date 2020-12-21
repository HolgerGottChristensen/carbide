use color::Color;
use ::{event, Positionable};
use graph::{self, Graph};
use input;
use position::{self, Align, Direction, Dimension, Padding, Point, Position, Range, Rect, Scalar, Dimensions};
use render;
use std;
use std::sync::atomic::{self, AtomicUsize};
use fnv;
use text;
use theme::Theme;
use utils;
use widget::{self, Rectangle};
use cursor;
use render::primitives::Primitives;
use render::cprimitives::CPrimitives;
use widget::primitive::Widget;
use crate::event::input::Input;
use input::Source;
use event::tap::Tap;
use event::ui::UiEvent;
use event::scroll::Scroll;
use event::press::PressEvent;
use event::button::ButtonEvent;
use event::release::Release;
use event::click::Click;
use event::text::Text;
use event::drag::Drag;
use event::double_click::DoubleClick;
use event_handler::{EventHandler, WindowEvent, WidgetEvent, MouseEvent, KeyboardEvent};
use event::event::Event;
use widget::render::Render;
use widget::common_widget::CommonWidget;
use std::fmt::{Debug, Formatter};
use std::fmt;
use state::environment::Environment;

/// A constructor type for building a `Ui` instance with a set of optional parameters.
pub struct UiBuilder {
    /// The initial dimensions of the window in which the `Ui` exists.
    pub window_dimensions: Dimensions,
    /// The theme used to set default styling for widgets.
    ///
    /// If this field is `None` when `build` is called, `Theme::default` will be used.
    pub maybe_theme: Option<Theme>,
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




/// `Ui` is the most important type within Conrod and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their widget::Id.
/// * Stores rendering state for each widget until the end of each render cycle.
/// * Contains the theme used for default styling of the widgets.
/// * Maintains the latest user input state (for mouse and keyboard).
/// * Maintains the latest window dimensions.
#[derive(Debug)]
pub struct Ui<S> {
    /// The theme used to set default styling for widgets.
    pub theme: Theme,
    /// An index into the root widget of the graph, representing the entire window.
    pub window: widget::Id,
    /// Handles aggregation of events and providing them to Widgets
    pub(crate) global_input: input::Global,
    /// Manages all fonts that have been loaded by the user.
    pub fonts: text::font::Map,
    /// The Widget cache, storing state for all widgets.
    pub(crate) widget_graph: Graph,


    /// The widget::Id of the widget that was last updated/set.
    maybe_prev_widget_id: Option<widget::Id>,
    /// The widget::Id of the last widget used as a parent for another widget.
    maybe_current_parent_id: Option<widget::Id>,
    /// The number of frames that will be used for the `redraw_count` when `need_redraw` is
    /// triggered.
    num_redraw_frames: u8,
    /// Whether or not the `Ui` needs to be re-drawn to screen.
    redraw_count: AtomicUsize,
    /// A background color to clear the screen with before drawing if one was given.
    maybe_background_color: Option<Color>,
    /// The order in which widgets from the `widget_graph` are drawn.
    pub(crate) depth_order: graph::DepthOrder,
    /// The set of widgets that have been updated since the beginning of the `set_widgets` stage.
    updated_widgets: fnv::FnvHashSet<widget::Id>,
    /// The `updated_widgets` for the previous `set_widgets` stage.
    ///
    /// We use this to compare against the newly generated `updated_widgets` to see whether or not
    /// we require re-drawing.
    prev_updated_widgets: fnv::FnvHashSet<widget::Id>,
    /// Scroll events that have been emitted during a call to `Ui::set_widgets`. These are usually
    /// emitted by some widget like the `Scrollbar`.
    ///
    /// These events will be drained and pushed onto the end of the `global_input` event buffer at
    /// the end of the `Ui::set_widgets` method. This ensures that the events are received by the
    /// target widgets during the next call to `Ui::set_widgets`.
    pending_scroll_events: Vec<UiEvent>,
    /// Mouse cursor
    mouse_cursor: cursor::MouseCursor,

    // TODO: Remove the following fields as they should now be handled by `input::Global`.

    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,


    pub widgets: Box<dyn Widget<S>>,
    event_handler: EventHandler,
    pub environment: Environment
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
pub struct UiCell<'a, S: 'static + Clone> {
    /// A mutable reference to a **Ui**.
    ui: &'a mut Ui<S>,
}


/// Each time conrod is required to redraw the GUI, it must draw for at least the next three frames
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
            maybe_theme: None,
            maybe_widgets_capacity: None
        }
    }

    /// The theme used to set default styling for widgets.
    ///
    /// If this field is `None` when `build` is called, `Theme::default` will be used.
    pub fn theme(mut self, value: Theme) -> Self {
        self.maybe_theme = Some(value);
        self
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
    pub fn build<S: 'static + Clone>(self) -> Ui<S> {
        Ui::new(self)
    }

}

impl<S: 'static + Clone> Ui<S> {

    /// A new, empty **Ui**.
    fn new(builder: UiBuilder) -> Self {

        let UiBuilder {
            window_dimensions,
            maybe_widgets_capacity,
            maybe_theme,
        } = builder;

        let (mut widget_graph, depth_order, updated_widgets) =
            maybe_widgets_capacity.map_or_else(
                || (Graph::new(),
                   graph::DepthOrder::new(),
                   fnv::FnvHashSet::default()),
                |n| (Graph::with_node_capacity(n),
                     graph::DepthOrder::with_node_capacity(n),
                     std::collections::HashSet::with_capacity_and_hasher(n,
                        fnv::FnvBuildHasher::default())));

        let window = widget_graph.add_placeholder();
        let prev_updated_widgets = updated_widgets.clone();
        Ui {
            widget_graph,
            theme: maybe_theme.unwrap_or_else(|| Theme::default()),
            fonts: text::font::Map::new(),
            window,
            widgets: Rectangle::initialize(vec![]),
            win_w: window_dimensions[0],
            win_h: window_dimensions[1],
            maybe_prev_widget_id: None,
            maybe_current_parent_id: None,
            num_redraw_frames: SAFE_REDRAW_COUNT,
            redraw_count: AtomicUsize::new(SAFE_REDRAW_COUNT as usize),
            maybe_background_color: None,
            depth_order,
            updated_widgets,
            prev_updated_widgets,
            global_input: input::Global::new(),
            pending_scroll_events: Vec::new(),
            mouse_cursor: cursor::MouseCursor::Arrow,
            event_handler: EventHandler::new(),
            environment: Environment::new()
        }
    }

    /// Returns a `input::Widget` for the given widget
    pub fn widget_input(&self, widget: widget::Id) -> input::Widget {
        // If there's no rectangle for a given widget, then we use one with zero area.
        // This means that the resulting `input::Widget` will not include any mouse events
        // unless it has captured the mouse, since none will have occured over that area.
        let rect = self.rect_of(widget).unwrap_or_else(|| {
            let right_edge = self.win_w / 2.0;
            let bottom_edge = self.win_h / 2.0;
            Rect::from_xy_dim([right_edge, bottom_edge], [0.0, 0.0])
        });
        input::Widget::for_widget(widget, rect, &self.global_input)
    }

    /// The **Rect** for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn rect_of(&self, id: widget::Id) -> Option<Rect> {
        self.widget_graph.widget(id).map(|widget| widget.rect)
    }

    /// The absolute width of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn w_of(&self, id: widget::Id) -> Option<Scalar> {
        self.rect_of(id).map(|rect| rect.w())
    }

    /// The absolute height of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn h_of(&self, id: widget::Id) -> Option<Scalar> {
        self.rect_of(id).map(|rect| rect.h())
    }

    /// The absolute dimensions for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn wh_of(&self, id: widget::Id) -> Option<Dimensions> {
        self.rect_of(id).map(|rect| rect.dim())
    }

    /// The coordinates for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn xy_of(&self, id: widget::Id) -> Option<Point> {
        self.rect_of(id).map(|rect| rect.xy())
    }

    /// The `kid_area` of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn kid_area_of(&self, id: widget::Id) -> Option<Rect> {
        self.widget_graph.widget(id).map(|widget| {
            widget.kid_area.rect.padding(widget.kid_area.pad)
        })
    }

    /// An index to the previously updated widget if there is one.
    pub fn maybe_prev_widget(&self) -> Option<widget::Id> {
        self.maybe_prev_widget_id
    }

    /// Borrow the **Ui**'s `widget_graph`.
    pub fn widget_graph(&self) -> &Graph {
        &self.widget_graph
    }

    /// Borrow the **Ui**'s set of updated widgets.
    ///
    /// This set indicates which widgets have been instantiated since the beginning of the most
    /// recent `Ui::set_widgets` call.
    pub fn updated_widgets(&self) -> &fnv::FnvHashSet<widget::Id> {
        &self.updated_widgets
    }

    /// Borrow the **Ui**'s set of updated widgets.
    ///
    /// This set indicates which widgets have were instantiated during the previous call to
    /// `Ui::set_widgets`.
    pub fn prev_updated_widgets(&self) -> &fnv::FnvHashSet<widget::Id> {
        &self.prev_updated_widgets
    }

    /// Produces a type that may be used to generate new unique `widget::Id`s.
    ///
    /// See the [**widget::id::Generator**](../widget/id/struct.Generator.html) docs for details on
    /// how to use this correctly.
    pub fn widget_id_generator(&mut self) -> widget::old::id::Generator {
        widget::old::id::Generator::new(&mut self.widget_graph)
    }

    /// Scroll the widget at the given index by the given offset amount.
    ///
    /// The produced `Scroll` event will be applied upon the next call to `Ui::set_widgets`.
    pub fn scroll_widget(&mut self, widget_id: widget::Id, offset: [Scalar; 2]) {
        let (x, y) = (offset[0], offset[1]);

        if x != 0.0 || y != 0.0 {
            let event = UiEvent::Scroll(Some(widget_id), Scroll {
                x: x,
                y: y,
                modifiers: self.global_input.current.modifiers,
            }).into();
            self.global_input.push_event(event);
        }
    }

    /// Determines which widget is currently under the mouse and sets it within the `Ui`'s
    /// `input::Global`'s `input::State`.
    ///
    /// If the `widget_under_mouse` has changed, this function will also update the
    /// `widget_capturing_mouse`.
    ///
    /// If the left mouse button is up, we assume that the widget directly under the
    /// mouse cursor captures all input from the mouse.
    ///
    /// If the left mouse button is down, we assume that the widget that was clicked
    /// remains "pinned" and will continue to capture the mouse until it is
    /// released.
    ///
    /// Note: This function expects that `ui.global_input.current.mouse.xy` is up-to-date.
    pub(crate) fn track_widget_under_mouse_and_update_capturing(&mut self) {
        self.global_input.current.widget_under_mouse =
            graph::algo::pick_widgets(&self.depth_order.indices,
                                      self.global_input.current.mouse.xy)
                                      .next(&self.widget_graph,
                                            &self.depth_order.indices,
                                            &self.theme);

        // If MouseButton::Left is up and `widget_under_mouse` has changed, capture new widget
        // under mouse.
        if self.global_input.current.mouse.buttons.left().is_up() {
            let widget_under_mouse = self.global_input.current.widget_under_mouse;

            // Check to see if we need to uncapture a widget.
            if let Some(idx) = self.global_input.current.widget_capturing_mouse {
                if widget_under_mouse != Some(idx) {
                    let source = input::Source::Mouse;
                    let event = UiEvent::WidgetUncapturesInputSource(idx, source).into();
                    self.global_input.push_event(event);
                    self.global_input.current.widget_capturing_mouse = None;
                }
            }

            // Check to see if there is a new widget capturing the mouse.
            if self.global_input.current.widget_capturing_mouse.is_none() {
                if let Some(idx) = widget_under_mouse {
                    let source = input::Source::Mouse;
                    let event = UiEvent::WidgetCapturesInputSource(idx, source).into();
                    self.global_input.push_event(event);
                    self.global_input.current.widget_capturing_mouse = Some(idx);
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: Input, global_state: &mut S) {
        let window_event = self.event_handler.handle_event(event, [self.win_w, self.win_h]);

        let mut needs_redraw = self.delegate_events(global_state);

        match  window_event {
            None => (),
            Some(event) => {
                match event {
                    WindowEvent::Resize(dimensions) => {
                        self.win_w = dimensions[0];
                        self.win_h = dimensions[1];
                        needs_redraw = true;
                    }
                    WindowEvent::Focus => needs_redraw = true,
                    WindowEvent::UnFocus => (),
                    WindowEvent::Redraw => needs_redraw = true,
                }
            }
        }

        if needs_redraw {
            //self.needs_redraw()
        }
    }

    fn delegate_events(&mut self, global_state: &mut S) -> bool {
        let events = self.event_handler.get_events();

        for event in events {
            match event {
                WidgetEvent::Mouse(mouse_event) => {
                    let consumed = false;
                    self.widgets.process_mouse_event(mouse_event, &consumed, Vec::new(), global_state);
                }
                WidgetEvent::Keyboard(keyboard_event) => {
                    self.widgets.process_keyboard_event(keyboard_event, Vec::new(), global_state);
                }
                WidgetEvent::Window(window_event) => {

                }
                WidgetEvent::Touch(_) => {}
            }

        }

        self.widgets.sync_state(Vec::new(), global_state);
        self.event_handler.clear_events();

        // Todo: Determine if an redraw is needed after events are processed
        return true
    }


    /// Get an immutable reference to global input. Handles aggregation of events and providing them to Widgets
    ///
    /// Can be used to access the current input state, e.g. which widgets are currently capturing inputs.
    pub fn global_input(&self) -> &input::Global {
        &self.global_input
    }

    /// Set keyboard capturing widget
    pub fn keyboard_capture(&mut self, idx: widget::Id) {
        let source = input::Source::Keyboard;

        if self.global_input.current.widget_capturing_keyboard.is_some() {
            let event = UiEvent::WidgetUncapturesInputSource(idx, source);
            self.global_input.push_event(event.into());
            self.global_input.current.widget_capturing_keyboard = None;
        }

        let event = UiEvent::WidgetCapturesInputSource(idx, source).into();
        self.global_input.push_event(event);
        self.global_input.current.widget_capturing_keyboard = Some(idx);
    }

    /*/// Get the centred xy coords for some given `Dimension`s, `Position` and alignment.
   ///
   /// If getting the xy for a specific widget, its `widget::Id` should be specified so that we
   /// can also consider the scroll offset of the scrollable parent widgets.
   ///
   /// The `place_on_kid_area` argument specifies whether or not **Place** **Position** variants
   /// should target a **Widget**'s `kid_area`, or simply the **Widget**'s total area.
   pub fn calc_xy(&self,
                  maybe_id: Option<widget::Id>,
                  maybe_parent_id: Option<widget::Id>,
                  x_position: Position,
                  y_position: Position,
                  dim: Dimensions,
                  place_on_kid_area: bool) -> Point
   {
       use utils::vec2_add;

       // Retrieves the absolute **Scalar** position from the given position for a single axis.
       //
       // The axis used is specified by the given range_from_rect function which, given some
       // **Rect**, returns the relevant **Range**.
       /*fn abs_from_position<R, P, S>(ui: &Ui<S>,
                                  maybe_parent_id: Option<widget::Id>,
                                  position: Position,
                                  dim: Scalar,
                                  place_on_kid_area: bool,
                                  range_from_rect: R,
                                  start_and_end_pad: P) -> Scalar
           where R: FnOnce(Rect) -> Range,
                 P: FnOnce(Padding) -> Range,
       {
           let (relative, maybe_id) = match position {
               Position::Absolute(abs) => return abs,
               Position::Relative(relative, maybe_id) => (relative, maybe_id),
           };

           match relative {

               position::Relative::Scalar(scalar) =>
                   maybe_id.or(ui.maybe_prev_widget_id).or(Some(ui.window.into()))
                       .and_then(|idx| ui.rect_of(idx).map(range_from_rect))
                       .map(|other_range| other_range.middle() + scalar)
                       .unwrap_or(scalar),

               position::Relative::Direction(direction, amt) =>
                   maybe_id.or(ui.maybe_prev_widget_id)
                       .and_then(|idx| ui.rect_of(idx).map(range_from_rect))
                       .map(|other_range| {
                           let range = Range::from_pos_and_len(0.0, dim);
                           match direction {
                               Direction::Forwards => range.align_after(other_range).middle() + amt,
                               Direction::Backwards => range.align_before(other_range).middle() - amt,
                           }
                       })
                       .unwrap_or_else(|| match direction {
                           Direction::Forwards => amt,
                           Direction::Backwards => -amt,
                       }),

               position::Relative::Align(align) =>
                   maybe_id.or(ui.maybe_prev_widget_id).or(Some(ui.window.into()))
                       .and_then(|idx| ui.rect_of(idx).map(range_from_rect))
                       .map(|other_range| {
                           let range = Range::from_pos_and_len(0.0, dim);
                           match align {
                               Align::Start => range.align_start_of(other_range).middle(),
                               Align::Middle => other_range.middle(),
                               Align::End => range.align_end_of(other_range).middle(),
                           }
                       })
                       .unwrap_or(0.0),

               position::Relative::Place(place) => {
                   let parent_id = maybe_id
                       .or(maybe_parent_id)
                       .or(ui.maybe_current_parent_id)
                       .unwrap_or(ui.window.into());
                   let maybe_area = match place_on_kid_area {
                       true => ui.widget_graph.widget(parent_id)
                           .map(|w| w.kid_area)
                           .map(|k| (range_from_rect(k.rect), start_and_end_pad(k.pad))),
                       false => ui.rect_of(parent_id)
                           .map(|rect| (range_from_rect(rect), Range::new(0.0, 0.0))),
                   };
                   maybe_area
                       .map(|(parent_range, pad)| {
                           let range = Range::from_pos_and_len(0.0, dim);
                           let parent_range = parent_range.pad_start(pad.start).pad_end(pad.end);
                           match place {
                               position::Place::Start(maybe_mgn) =>
                                   range.align_start_of(parent_range).middle() + maybe_mgn.unwrap_or(0.0),
                               position::Place::Middle =>
                                   parent_range.middle(),
                               position::Place::End(maybe_mgn) =>
                                   range.align_end_of(parent_range).middle() - maybe_mgn.unwrap_or(0.0),
                           }
                       })
                       .unwrap_or(0.0)
               },
           }
       }*/

       fn x_range(rect: Rect) -> Range { rect.x }
       fn y_range(rect: Rect) -> Range { rect.y }
       fn x_pad(pad: Padding) -> Range { pad.x }
       fn y_pad(pad: Padding) -> Range { pad.y }
       let x = abs_from_position(self, maybe_parent_id, x_position, dim[0], place_on_kid_area, x_range, x_pad);
       let y = abs_from_position(self, maybe_parent_id, y_position, dim[1], place_on_kid_area, y_range, y_pad);
       let xy = [x, y];

       // Add the widget's parents' total combined scroll offset to the given xy.
       maybe_id
           .map(|idx| vec2_add(xy, graph::algo::scroll_offset(&self.widget_graph, idx)))
           .unwrap_or(xy)
   }

*/
    /// A function within which all widgets are instantiated by the user, normally situated within
    /// the "update" stage of an event loop.
    pub fn set_widgets(&mut self) -> UiCell<S> {
        self.maybe_prev_widget_id = None;
        self.maybe_current_parent_id = None;

        // Move the previous `updated_widgets` to `prev_updated_widgets` and clear
        // `updated_widgets` so that we're ready to store the newly updated widgets.
        {
            let Ui { ref mut updated_widgets, ref mut prev_updated_widgets, .. } = *self;
            std::mem::swap(updated_widgets, prev_updated_widgets);
            updated_widgets.clear();
        }

        let mut ui_cell = UiCell { ui: self };

        // Instantiate the root `Window` `Widget`.
        //
        // This widget acts as the parent-most widget and root node for the Ui's `widget_graph`,
        // upon which all other widgets are placed.
        /*{
            use {color, Colorable, Borderable, Positionable};
            type Window = widget::BorderedRectangle;
            Window::new([ui_cell.win_w, ui_cell.win_h])
                .border(0.0)
                .border_color(color::BLACK.alpha(0.0))
                .color(ui_cell.maybe_background_color.unwrap_or(color::BLACK.alpha(0.0)))
                .set(ui_cell.window, &mut ui_cell);
        }*/

        ui_cell.ui.maybe_current_parent_id = Some(ui_cell.window.into());

        ui_cell.set_mouse_cursor(cursor::MouseCursor::Arrow);

        ui_cell
    }


    /// Set the number of frames that the `Ui` should draw in the case that `needs_redraw` is
    /// called. The default is `3` (see the SAFE_REDRAW_COUNT docs for details).
    pub fn set_num_redraw_frames(&mut self, num_frames: u8) {
        self.num_redraw_frames = num_frames;
    }


    /// Tells the `Ui` that it needs to re-draw everything. It does this by setting the redraw
    /// count to `num_redraw_frames`. See the docs for `set_num_redraw_frames`, SAFE_REDRAW_COUNT
    /// or `draw_if_changed` for more info on how/why the redraw count is used.
    pub fn needs_redraw(&self) {
        self.redraw_count.store(self.num_redraw_frames as usize, atomic::Ordering::Relaxed);
    }

    /// The first of the `Primitives` yielded by `Ui::draw` or `Ui::draw_if_changed` will always
    /// be a `Rectangle` the size of the window in which conrod is hosted.
    ///
    /// This method sets the colour with which this `Rectangle` is drawn (the default being
    /// `conrod::color::TRANSPARENT`.
    pub fn clear_with(&mut self, color: Color) {
        self.maybe_background_color = Some(color);
    }

    /// Draw the `Ui` in it's current state.
    ///
    /// NOTE: If you don't need to redraw your conrod GUI every frame, it is recommended to use the
    /// `Ui::draw_if_changed` method instead.
    pub fn draw(&mut self) -> (Primitives, CPrimitives) {
        let Ui {
            ref redraw_count,
            ref widget_graph,
            ref mut widgets,
            ref depth_order,
            ref theme,
            ref fonts,
            win_w, win_h,
            ref mut environment,
            ..
        } = *self;

        // Use the depth_order indices as the order for drawing.
        let indices = &depth_order.indices;

        // We're about to draw everything, so take one from the redraw count.
        let remaining_redraws = redraw_count.load(atomic::Ordering::Relaxed);
        if remaining_redraws > 0 {
            redraw_count.store(remaining_redraws - 1, atomic::Ordering::Relaxed);
        }

        (
            Primitives::new(widget_graph, indices, theme, fonts, [win_w, win_h]),
            CPrimitives::new([win_w, win_h], widgets, environment)
        )
    }


    /// Same as the `Ui::draw` method, but *only* draws if the `redraw_count` is greater than 0.
    ///
    /// The `redraw_count` is set to `SAFE_REDRAW_COUNT` whenever a `Widget` indicates that it
    /// needs to be re-drawn.
    ///
    /// It can also be triggered manually by the user using the `Ui::needs_redraw` method.
    ///
    /// This method is generally preferred over `Ui::draw` as it requires far less CPU usage, only
    /// redrawing to the screen if necessary.
    ///
    /// Note that when `Ui::needs_redraw` is triggered, it sets the `redraw_count` to 3 by default.
    /// This ensures that conrod is drawn to each buffer in the case that there is buffer swapping
    /// happening. Let us know if you need finer control over this and we'll expose a way for you
    /// to set the redraw count manually.
    pub fn draw_if_changed(&mut self) -> Option<(Primitives, CPrimitives)> {
        if self.has_changed() {
            return Some(self.draw())
        }

        None
    }

    /// Returns if the redraw_count is greater than 0 and thus draw_if_changed would draw
    /// See `Ui::draw_if_changed` for when this is triggered
    pub fn has_changed(&self) -> bool {
        self.redraw_count.load(atomic::Ordering::Relaxed) > 0
    }


    /// The **Rect** that bounds the kids of the widget with the given index.
    pub fn kids_bounding_box(&self, id: widget::Id) -> Option<Rect> {
        graph::algo::kids_bounding_box(&self.widget_graph, &self.prev_updated_widgets, id)
    }


    /// The **Rect** that represents the maximum fully visible area for the widget with the given
    /// index, including consideration of cropped scroll area.
    ///
    /// Otherwise, return None if the widget is not visible.
    pub fn visible_area(&self, id: widget::Id) -> Option<Rect> {
        graph::algo::cropped_area_of_widget(&self.widget_graph, id)
    }

    /// Get mouse cursor state.
    pub fn mouse_cursor(&self) -> cursor::MouseCursor {
        self.mouse_cursor
    }
}


impl<'a, S: Clone> UiCell<'a, S> {

    /// A reference to the `Theme` that is currently active within the `Ui`.
    pub fn theme(&self) -> &Theme { &self.ui.theme }

    /// A convenience method for borrowing the `Font` for the given `Id` if it exists.
    pub fn font(&self, id: text::font::Id) -> Option<&text::Font> {
        self.ui.fonts.get(id)
    }

    /// Returns the dimensions of the window
    pub fn window_dim(&self) -> Dimensions {
        [self.ui.win_w, self.ui.win_h]
    }

    /// Returns an immutable reference to the `input::Global` of the `Ui`.
    ///
    /// All coordinates here will be relative to the center of the window.
    pub fn global_input(&self) -> &input::Global {
        &self.ui.global_input
    }

    /*/// Returns a `input::Widget` with input events for the widget.
    ///
    /// All coordinates in the `input::Widget` will be relative to the widget at the given index.
    pub fn widget_input(&self, id: widget::Id) -> input::Widget {
        self.ui.widget_input(id)
    }*/

    /*/// Produces a type that may be used to generate new unique `widget::Id`s.
    ///
    /// See the [**widget::id::Generator**](../widget/id/struct.Generator.html) docs for details on
    /// how to use this correctly.
    pub fn widget_id_generator(&mut self) -> widget::id::Generator {
        self.ui.widget_id_generator()
    }

    /// The **Rect** that bounds the kids of the widget with the given index.
    ///
    /// Returns `None` if the widget has no children or if there's is no widget for the given index.
    pub fn kids_bounding_box(&self, id: widget::Id) -> Option<Rect> {
        self.ui.kids_bounding_box(id)
    }*/

    /// Scroll the widget at the given index by the given offset amount.
    ///
    /// The produced `Scroll` event will be pushed to the `pending_scroll_events` and will be
    /// applied to the widget during the next call to `Ui::set_widgets`.
    pub fn scroll_widget(&mut self, id: widget::Id, offset: [Scalar; 2]) {
        let (x, y) = (offset[0], offset[1]);

        if x != 0.0 || y != 0.0 {
            let event = UiEvent::Scroll(Some(id), Scroll {
                x,
                y,
                modifiers: self.ui.global_input.current.modifiers,
            });
            self.ui.pending_scroll_events.push(event);
        }
    }

    /// Sets the mouse cursor
    pub fn set_mouse_cursor(&mut self, cursor: cursor::MouseCursor) {
        self.ui.mouse_cursor = cursor;
    }
}

impl<'a, S: 'static + Clone> Drop for UiCell<'a, S> {
    fn drop(&mut self) {
        // We'll need to re-draw if we have gained or lost widgets.
        let changed = self.ui.updated_widgets != self.ui.prev_updated_widgets;
        if changed {
            self.ui.needs_redraw();
        }

        // Update the **DepthOrder** so that it reflects the **Graph**'s current state.
        {
            let Ui {
                ref widget_graph,
                ref mut depth_order,
                window,
                ref updated_widgets,
                ..
            } = *self.ui;

            depth_order.update(widget_graph, window, updated_widgets);
        }

        // Reset the global input state. Note that this is the **only** time this should be called.
        self.ui.global_input.clear_events_and_update_start_state();

        // Update which widget is under the cursor.
        if changed {
            self.ui.track_widget_under_mouse_and_update_capturing();
        }

        // Move all pending `Scroll` events that have been produced since the start of this method
        // into the `global_input` event buffer.
        for scroll_event in self.ui.pending_scroll_events.drain(0..) {
            self.ui.global_input.push_event(scroll_event.into());
        }
    }
}

impl<'a, S: Clone> ::std::ops::Deref for UiCell<'a, S> {
    type Target = Ui<S>;
    fn deref(&self) -> &Ui<S> {
        self.ui
    }
}

impl<'a, S: Clone> AsRef<Ui<S>> for UiCell<'a, S> {
    fn as_ref(&self) -> &Ui<S> {
        &self.ui
    }
}

/// A function for retrieving the `&mut Ui<B>` from a `UiCell<B>`.
///
/// This function is only for internal use to allow for some `Ui` type acrobatics in order to
/// provide a nice *safe* API for the user.
pub fn ref_mut_from_ui_cell<'a, 'b: 'a, S: Clone>(ui_cell: &'a mut UiCell<'b, S>) -> &'a mut Ui<S> {
    ui_cell.ui
}

/// A mutable reference to the given `Ui`'s widget `Graph`.
pub fn widget_graph_mut<S>(ui: &mut Ui<S>) -> &mut Graph {
    &mut ui.widget_graph
}


/// Infer a widget's `Depth` parent by examining it's *x* and *y* `Position`s.
///
/// When a different parent may be inferred from either `Position`, the *x* `Position` is favoured.
pub fn infer_parent_from_position<S>(ui: &Ui<S>, x: Position, y: Position) -> Option<widget::Id> {
    use Position::Relative;
    use position::Relative::{Align, Direction, Place, Scalar};
    match (x, y) {
        (Relative(Place(_), maybe_parent_id), _) | (_, Relative(Place(_), maybe_parent_id)) =>
            maybe_parent_id,
        (Relative(Direction(_, _), maybe_id), _) | (_, Relative(Direction(_, _), maybe_id)) |
        (Relative(Align(_), maybe_id), _)        | (_, Relative(Align(_), maybe_id))        |
        (Relative(Scalar(_), maybe_id), _)       | (_, Relative(Scalar(_), maybe_id))       =>
            maybe_id.or(ui.maybe_prev_widget_id)
                .and_then(|idx| ui.widget_graph.depth_parent(idx)),
        _ => None,
    }
}


/// Attempts to infer the parent of a widget from its *x*/*y* `Position`s and the current state of
/// the `Ui`.
///
/// If no parent can be inferred via the `Position`s, the `maybe_current_parent_id` will be used.
///
/// If `maybe_current_parent_id` is `None`, the `Ui`'s `window` widget will be used.
///
/// **Note:** This function does not check whether or not using the `window` widget would cause a
/// cycle.
pub fn infer_parent_unchecked<S>(ui: &Ui<S>, x_pos: Position, y_pos: Position) -> widget::Id {
    infer_parent_from_position(ui, x_pos, y_pos)
        .or(ui.maybe_current_parent_id)
        .unwrap_or(ui.window.into())
}


/// Cache some `PreUpdateCache` widget data into the widget graph.
/// Set the widget that is being cached as the new `prev_widget`.
/// Set the widget's parent as the new `current_parent`.
pub fn pre_update_cache<S>(ui: &mut Ui<S>, widget: widget::PreUpdateCache) {
    ui.maybe_prev_widget_id = Some(widget.id);
    ui.maybe_current_parent_id = widget.maybe_parent_id;
    let widget_id = widget.id;
    ui.widget_graph.pre_update_cache(ui.window, widget, ui.updated_widgets.len());

    // Add the widget's `widget::Id` to the set of updated widgets.
    ui.updated_widgets.insert(widget_id);
}

/* /// Cache some `PostUpdateCache` widget data into the widget graph.
/// Set the widget that is being cached as the new `prev_widget`.
/// Set the widget's parent as the new `current_parent`.
pub fn post_update_cache<W, S>(ui: &mut Ui<S>, widget: widget::PostUpdateCache<W>)
    where W: OldWidget,
          W::State: 'static,
          W::Style: 'static,
{
    ui.maybe_prev_widget_id = Some(widget.id);
    ui.maybe_current_parent_id = widget.maybe_parent_id;
    ui.widget_graph.post_update_cache(widget);
}*/
