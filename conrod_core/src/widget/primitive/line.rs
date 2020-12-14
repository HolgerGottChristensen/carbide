//! A simple, non-interactive widget for drawing a single straight Line.

use {Color, Colorable, Point, Positionable, Rect, Scalar, Sizeable, Theme};
use ::{graph, text};
use utils::{vec2_add, vec2_sub};
use widget::{self, Id};
use position::Dimensions;
use widget::primitive::Widget;
use widget::render::Render;
use render::primitive::Primitive;
use graph::Container;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use daggy::petgraph::graph::node_index;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::layout::Layout;
use text::font::Map;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use draw::shape::line::is_over_widget;


/// A simple, non-interactive widget for drawing a single straight Line.
#[derive(Debug, Clone, WidgetCommon_)]
pub struct Line {
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// The start of the line.
    pub start: Point,
    /// The end of the line.
    pub end: Point,
    /// Unique styling.
    pub style: Style,
    /// Whether or not the line should be automatically centred to the widget position.
    pub should_centre_points: bool,
    position: Point,
    dimension: Dimensions,

    pub children: Vec<Box<dyn Widget>>
}

impl<S> Event<S> for Line {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList) {
        self.sync_state_default(states);
    }
}

impl WidgetExt for Line {}

impl Layout for Line {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], fonts: &Map) -> [f64; 2] {
        unimplemented!()
    }

    fn position_children(&mut self) {
        unimplemented!()
    }
}

impl Render for Line {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        const DEFAULT_CAP: Cap = Cap::Flat;
        let thickness = 2.0;
        let points = std::iter::once(self.start).chain(std::iter::once(self.end));
        let triangles = match widget::point_path::triangles(points, DEFAULT_CAP, thickness) {
            None => Vec::new(),
            Some(iter) => {
                iter.collect()
            },
        };
        let kind = PrimitiveKind::TrianglesSingleColor {
            color: Color::random().to_rgb(),
            triangles: triangles.to_vec(),
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(node_index(0), kind, Rect::new(self.position, self.dimension), Rect::new(self.position, self.dimension))];
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

impl CommonWidget for Line {
    fn get_id(&self) -> Uuid {
        unimplemented!()
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter {
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        self.children.iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

/// Unique state for the Line widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// The start of the line.
    pub start: Point,
    /// The end of the line.
    pub end: Point,
}

/// Unique styling for a Line widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// The patter for the line.
    pub maybe_pattern: Option<Pattern>,
    /// Color of the Button's pressable area.
    pub maybe_color: Option<Color>,
    /// The thickness of the line.
    pub maybe_thickness: Option<Scalar>,
    /// The style with which the ends of the line are drawn.
    pub maybe_cap: Option<Cap>,
}

/// The pattern used to draw the line.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pattern {
    /// A single continuous stroke.
    Solid,
    /// A series of line strokes.
    Dashed,
    /// A series of circles.
    Dotted,
}

/// Whether the end of the **Line** should be flat or rounded.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cap {
    /// The line is capped with a flat edge.
    Flat,
    /// The line is capped with a semi-circle.
    Round,
}


impl Line {

    pub fn new(start: Point, end: Point, children: Vec<Box<dyn Widget>>) -> Box<Line> {
        Box::new(Line {
            start,
            end,
            common: widget::CommonBuilder::default(),
            style: Style::new(),
            should_centre_points: false,
            position: [10.0,10.0],
            dimension: [10.0,10.0],
            children
        })
    }

    /// Build a new **Line** widget with the given style.
    pub fn styled(start: Point, end: Point, style: Style) -> Self {
        Line {
            start: start,
            end: end,
            common: widget::CommonBuilder::default(),
            style: style,
            should_centre_points: false,
            position: [10.0,10.0],
            dimension: [10.0,10.0],
            children: vec![]
        }
    }
    /// Build a new **Line** whose bounding box is fit to the absolute co-ordinates of the line
    /// points.
    ///
    /// If you would rather centre the start and end to the middle of the bounding box, use
    /// [**Line::centred**](./struct.Line#method.centred) instead.
    pub fn abs(start: Point, end: Point) -> Self {
        Line::abs_styled(start, end, Style::new())
    }

    /// The same as [**Line::abs**](./struct.Line#method.abs) but with the given style.
    pub fn abs_styled(start: Point, end: Point, style: Style) -> Self {
        let (xy, dim) = Rect::from_corners(start, end).xy_dim();
        Line::styled(start, end, style).wh(dim).xy(xy)
    }

    /// Build a new **Line** and shift the location of the start and end points so that the centre
    /// of their bounding rectangle lies at the position determined by the layout for the **Line**
    /// widget.
    ///
    /// This is useful if your points simply describe the line's angle and magnitude, and you want
    /// to position them using conrod's auto-layout or `Positionable` methods.
    ///
    /// If you would rather centre the bounding box to the points, use
    /// [**Line::abs**](./struct.Line#method.abs) instead.
    pub fn centred(start: Point, end: Point) -> Self {
        Line::centred_styled(start, end, Style::new())
    }

    /// The same as [**Line::centred**](./struct.Line#method.centred) but with the given style.
    pub fn centred_styled(start: Point, end: Point, style: Style) -> Self {
        let dim = Rect::from_corners(start, end).dim();
        let mut line = Line::styled(start, end, style);//.wh(dim);
        line.should_centre_points = true;
        line
    }

    /// The thickness or width of the Line.
    ///
    /// Use this instead of `Positionable::width` for the thickness of the `Line`, as `width` and
    /// `height` refer to the dimensions of the bounding rectangle.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.style.set_thickness(thickness);
        self
    }

    /// Make a solid line.
    pub fn solid(mut self) -> Self {
        self.style.set_pattern(Pattern::Solid);
        self
    }

    /// Make a line with a Dashed pattern.
    pub fn dashed(mut self) -> Self {
        self.style.set_pattern(Pattern::Dashed);
        self
    }

    /// Make a line with a Dotted pattern.
    pub fn dotted(mut self) -> Self {
        self.style.set_pattern(Pattern::Dotted);
        self
    }

}


impl Style {

    /// Constructor for a default Line Style.
    pub fn new() -> Self {
        Style {
            maybe_pattern: None,
            maybe_color: None,
            maybe_thickness: None,
            maybe_cap: None,
        }
    }

    /// Make a solid line.
    pub fn solid() -> Self {
        Style::new().pattern(Pattern::Solid)
    }

    /// Make a line with a Dashed pattern.
    pub fn dashed() -> Self {
        Style::new().pattern(Pattern::Dashed)
    }

    /// Make a line with a Dotted pattern.
    pub fn dotted() -> Self {
        Style::new().pattern(Pattern::Dotted)
    }

    /// The style with some given pattern.
    pub fn pattern(mut self, pattern: Pattern) -> Self {
        self.set_pattern(pattern);
        self
    }

    /// The style with some given color.
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// The style with some given thickness.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.set_thickness(thickness);
        self
    }

    /// The style for the ends of the Line.
    pub fn cap(mut self, cap: Cap) -> Self {
        self.set_cap(cap);
        self
    }

    /// Set the pattern for the line.
    pub fn set_pattern(&mut self, pattern: Pattern) {
        self.maybe_pattern = Some(pattern);
    }

    /// Set the color for the line.
    pub fn set_color(&mut self, color: Color) {
        self.maybe_color = Some(color);
    }

    /// Set the thickness for the line.
    pub fn set_thickness(&mut self, thickness: Scalar) {
        self.maybe_thickness = Some(thickness);
    }

    /// Set the **Cap** for the line.
    pub fn set_cap(&mut self, cap: Cap) {
        self.maybe_cap = Some(cap);
    }

    /// The Pattern for the Line.
    pub fn get_pattern(&self, theme: &Theme) -> Pattern {
        const DEFAULT_PATTERN: Pattern = Pattern::Solid;
        self.maybe_pattern.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_pattern.unwrap_or(DEFAULT_PATTERN)
        })).unwrap_or(DEFAULT_PATTERN)
    }

    /// The Color for the Line.
    pub fn get_color(&self, theme: &Theme) -> Color {
        self.maybe_color.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// The width or thickness of the Line.
    pub fn get_thickness(&self, theme: &Theme) -> Scalar {
        const DEFAULT_THICKNESS: Scalar = 1.0;
        self.maybe_thickness.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_thickness.unwrap_or(DEFAULT_THICKNESS)
        })).unwrap_or(DEFAULT_THICKNESS)
    }

    /// The styling for the ends of the Line.
    pub fn get_cap(&self, theme: &Theme) -> Cap {
        const DEFAULT_CAP: Cap = Cap::Flat;
        self.maybe_cap.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_cap.unwrap_or(DEFAULT_CAP)
        })).unwrap_or(DEFAULT_CAP)
    }

}


/*impl<S> OldWidget<S> for Line<S> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            start: [0.0, 0.0],
            end: [0.0, 0.0],
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn is_over(&self) -> widget::IsOverFn {
        is_over_widget
    }

    /// Update the state of the Line.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { rect, state, .. } = args;
        let Line { mut start, mut end, should_centre_points, .. } = self;

        // Check whether or not we need to shift the line to the xy position.
        if should_centre_points {
            let original = Rect::from_corners(start, end).xy();
            let xy = rect.xy();
            let difference = vec2_sub(xy, original);
            start = vec2_add(start, difference);
            end = vec2_add(end, difference);
        }

        if state.start != start {
            state.update(|state| state.start = start);
        }

        if state.end != end {
            state.update(|state| state.end = end);
        }
    }
}
*/

impl Colorable for Line {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

