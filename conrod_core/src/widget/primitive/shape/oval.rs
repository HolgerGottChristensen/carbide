//! A simple, non-interactive widget for drawing a single **Oval**.

use {Color, Colorable, Point, Rect, Scalar, Sizeable, Theme, OldWidget};
use ::{graph, text};
use std;
use super::Style as Style;
use widget;
use widget::triangles::Triangle;
use widget::render::Render;
use graph::Container;
use widget::{Id, Rectangle};
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use position::Dimensions;
use render::owned_primitive::OwnedPrimitive;
use daggy::petgraph::graph::node_index;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::primitive::Widget;
use widget::layout::Layout;
use text::font::Map;
use widget::envelope_editor::EnvelopePoint;
use layout::basic_layouter::BasicLayouter;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use state::state::{StateList, DefaultState};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use std::slice::{Iter, IterMut};


/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Debug, WidgetCommon_)]
pub struct Oval<S> {
    pub id: Uuid,
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling.
    pub style: Style,
    /// The number of lines used to draw the edge.
    pub resolution: usize,
    /// A type describing the section of the `Oval` that is to be drawn.
    pub section: S,
    position: Point,
    dimension: Dimensions,
    color: Color,

    pub children: Vec<Box<dyn Widget>>
}

impl<S> Event for Oval<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList<DefaultState>) -> StateList<DefaultState> {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList<DefaultState>) -> StateList<DefaultState> {
        self.process_keyboard_event_default(event, state)
    }

    fn get_state(&self, current_state: StateList<DefaultState>) -> StateList<DefaultState> {
        current_state
    }

    fn apply_state(&mut self, states: StateList<DefaultState>) -> StateList<DefaultState> {
        states
    }

    fn sync_state(&mut self, states: StateList<DefaultState>) {
        self.sync_state_default(states);
    }
}

impl<S: 'static> WidgetExt for Oval<S> {}

impl<S> Layout for Oval<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        for child in &mut self.children {
            child.calculate_size(requested_size, fonts);
        }
        self.dimension = requested_size;

        requested_size

    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl<S> CommonWidget for Oval<S> {
    fn get_id(&self) -> Uuid {
        self.id
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

    fn get_position(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_x(&self) -> Scalar {
        self.position[0]
    }

    fn set_x(&mut self, x: f64) {
        self.position = Point::new(x, self.position.get_y());
    }

    fn get_y(&self) -> Scalar {
        self.position[1]
    }

    fn set_y(&mut self, y: f64) {
        self.position = Point::new(self.position.get_x(), y);
    }

    fn get_size(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        self.dimension[0]
    }

    fn get_height(&self) -> Scalar {
        self.dimension[1]
    }
}

impl<S> Render for Oval<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let points = widget::oval::circumference(Rect::new(self.position, self.dimension), DEFAULT_RESOLUTION);
        let mut triangles: Vec<Triangle<Point>> = Vec::new();
        triangles.extend(points.triangles());
        let kind = PrimitiveKind::TrianglesSingleColor {
            color: self.color.to_rgb(),
            triangles,
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(node_index(0), kind, Rect::new(self.position, self.dimension), Rect::new(self.position, self.dimension))];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

/// Types that may be used to describe the visible section of the `Oval`.
pub trait OvalSection: 'static + Copy + PartialEq + Send {
    /// The function used to determine if a point is over the oval section widget.
    const IS_OVER: widget::IsOverFn;
}

/// The entire `Oval` will be drawn.
///
/// To draw only a section of the oval, use the `section` builder method.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Full;

impl OvalSection for Full {
    const IS_OVER: widget::IsOverFn = is_over_widget;
}

/// A section of the oval will be drawn where the section is specified by the given radians.
///
/// A section with `radians` of `2.0 * PI` would be equivalent to the full `Oval`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Section {
    /// The angle occuppied by the section's circumference.
    pub radians: Scalar,
    /// The radians at which the section will begin.
    ///
    /// A value of `0.0` will begin at the right of the oval.
    pub offset_radians: Scalar,
}

impl OvalSection for Section {
    const IS_OVER: widget::IsOverFn = is_over_section_widget;
}

/// Unique state for the **Oval**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State<S> {
    /// The number of lines used to draw the edge.
    pub resolution: usize,
    /// A type describing the section of the `Oval` that is to be drawn.
    pub section: S,
}

/// The default circle resolution if none is specified.
pub const DEFAULT_RESOLUTION: usize = 50;

impl Oval<Full> {

    pub fn fill(mut self, color: Color) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn initialize(children: Vec<Box<dyn Widget>>) -> Box<Oval<Full>> {
        Box::new(Oval {
            id: Uuid::new_v4(),
            common: widget::CommonBuilder::default(),
            style: Style::fill(),
            resolution: 0,
            section: Full,
            position: [0.0, 0.0],
            dimension: [100.0,100.0],
            color: Color::random(),
            children
        })
    }

    pub fn new(position: Point, dimension: Dimensions, children: Vec<Box<dyn Widget>>) -> Box<Oval<Full>> {
        Box::new(Oval {
            id: Uuid::new_v4(),
            children,
            position,
            dimension,
            common: widget::CommonBuilder::default(),
            style: Style::fill(),
            resolution: 0,
            section: Full,
            color: Color::random()
        })
    }

    /// Build an **Oval** with the given dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Oval {
            id: Uuid::new_v4(),
            common: widget::CommonBuilder::default(),
            style: style,
            resolution: DEFAULT_RESOLUTION,
            section: Full,
            position: [0.0,0.0],
            dimension: [0.0,0.0],
            color: Color::random(),
            children: vec![]
        }.wh(dim)
    }

    /// Build a new **Fill**ed **Oval**.
    pub fn fill_old(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::fill())
    }

    /// Build a new **Oval** **Fill**ed with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Oval::styled(dim, Style::fill_with(color))
    }

    /// Build a new **Outline**d **Oval** widget.
    pub fn outline(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::outline())
    }

    /// Build a new **Oval** **Outline**d with the given style.
    pub fn outline_styled(dim: Dimensions, line_style: widget::line::Style) -> Self {
        Oval::styled(dim, Style::outline_styled(line_style))
    }
}

impl<S> Oval<S> {
    /// The number of lines used to draw the edge.
    ///
    /// By default, `DEFAULT_RESOLUTION` is used.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }

    /// Produces an `Oval` where only a section is drawn.
    ///
    /// The given `radians` describes the angle occuppied by the section's circumference.
    pub fn section(self, radians: Scalar) -> Oval<Section> {
        let Oval { common, style, resolution, .. } = self;
        let section = Section { radians, offset_radians: 0.0 };
        Oval { id: Uuid::new_v4(), common, style, resolution, section, position: [10.0, 10.0], dimension: [10.0,10.0], color: Color::random(), children: vec![] }
    }
}

impl Oval<Section> {
    /// The radians at which the section will begin.
    ///
    /// A value of `0.0` will begin at the rightmost point of the oval.
    pub fn offset_radians(mut self, offset_radians: Scalar) -> Self {
        self.section.offset_radians = offset_radians;
        self
    }
}

impl<S> OldWidget for Oval<S>
where
    S: OvalSection,
{
    type State = State<S>;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            resolution: DEFAULT_RESOLUTION,
            section: self.section,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn is_over(&self) -> widget::IsOverFn {
        S::IS_OVER
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, .. } = args;
        if state.resolution != self.resolution {
            state.update(|state| state.resolution = self.resolution);
        }
        if state.section != self.section {
            state.update(|state| state.section = self.section);
        }
    }
}

impl<S> Colorable for Oval<S> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}

/// An iterator yielding the `Oval`'s edges as a circumference represented as a series of points.
///
/// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
/// produces `NaN` values.
pub fn circumference(rect: Rect, resolution: usize) -> Circumference {
    Circumference::new(rect, resolution)
}

/// An iterator yielding the triangles that describe the given oval.
pub fn triangles(rect: Rect, resolution: usize) -> Triangles {
    circumference(rect, resolution).triangles()
}

/// An iterator yielding the edges of an `Oval` (or some section of an `Oval`) as a circumference
/// represented as a series of edges.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Circumference {
    index: usize,
    num_points: usize,
    point: Point,
    rad_step: Scalar,
    rad_offset: Scalar,
    half_w: Scalar,
    half_h: Scalar,
}

impl Circumference {
    fn new_inner(rect: Rect, num_points: usize, rad_step: Scalar) -> Self {
        let (x, y, w, h) = rect.x_y_w_h();
        Circumference {
            index: 0,
            num_points,
            point: [x, y],
            half_w: w * 0.5,
            half_h: h * 0.5,
            rad_step,
            rad_offset: 0.0,
        }
    }

    /// An iterator yielding the `Oval`'s edges as a circumference represented as a series of points.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new(rect: Rect, mut resolution: usize) -> Self {
        resolution = std::cmp::max(resolution, 1);
        use std::f64::consts::PI;
        let radians = 2.0 * PI;
        Self::new_section(rect, resolution, radians)
    }

    /// Produces a new iterator that yields only a section of the `Oval`'s circumference, where the
    /// section is described via its angle in radians.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new_section(rect: Rect, resolution: usize, radians: Scalar) -> Self {
        Self::new_inner(rect, resolution + 1, radians / resolution as Scalar)
    }
}

/// An iterator yielding triangles that describe an oval or some section of an oval.
#[derive(Clone)]
pub struct Triangles {
    // The last circumference point yielded by the `CircumferenceOffset` iterator.
    last: Point,
    // The circumference points used to yield yielded by the `CircumferenceOffset` iterator.
    points: Circumference,
}

impl Circumference {
    /// Produces a new iterator that yields only a section of the `Oval`'s circumference, where the
    /// section is described via its angle in radians.
    pub fn section(mut self, radians: Scalar) -> Self {
        let resolution = self.num_points - 1;
        self.rad_step = radians / resolution as Scalar;
        self
    }

    /// Rotates the position at which the iterator starts yielding points by the given radians.
    ///
    /// This is particularly useful for yielding a different section of the circumference when
    /// using `circumference_section`
    pub fn offset_radians(mut self, radians: Scalar) -> Self {
        self.rad_offset = radians;
        self
    }

    /// Produces an `Iterator` yielding `Triangle`s.
    ///
    /// Triangles are created by joining each edge yielded by the inner `Circumference` to the
    /// middle of the `Oval`.
    pub fn triangles(mut self) -> Triangles {
        let last = self.next().unwrap_or(self.point);
        Triangles { last, points: self }
    }
}

impl Iterator for Circumference {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference {
            ref mut index,
            num_points,
            point,
            rad_step,
            rad_offset,
            half_w,
            half_h,
        } = *self;
        if *index >= num_points {
            return None;
        }
        let x = point[0] + half_w * (rad_offset + rad_step * *index as Scalar).cos();
        let y = point[1] + half_h * (rad_offset + rad_step * *index as Scalar).sin();
        *index += 1;
        Some([x, y])
    }
}

impl Iterator for Triangles {
    type Item = Triangle<Point>;
    fn next(&mut self) -> Option<Self::Item> {
        let Triangles { ref mut points, ref mut last } = *self;
        points.next().map(|next| {
            let triangle = Triangle([points.point, *last, next]);
            *last = next;
            triangle
        })
    }
}

/// Returns `true` if the given `Point` is over an oval at the given rect.
pub fn is_over(r: Rect, p: Point) -> bool {
    let (px, py) = (p[0], p[1]);
    let (ox, oy, w, h) = r.x_y_w_h();
    let rx = w * 0.5;
    let ry = h * 0.5;
    ((px - ox).powi(2) / rx.powi(2) + (py - oy).powi(2) / ry.powi(2)) < 1.0
}

/// The function to use for picking whether a given point is over the oval.
pub fn is_over_widget(widget: &graph::Container, point: Point, _: &Theme) -> widget::IsOver {
    is_over(widget.rect, point).into()
}

/// Returns whether or not the given point is over the section described
pub fn is_over_section(circumference: Circumference, p: Point) -> bool {
    widget::triangles::is_over(circumference.triangles(), p)
}

/// The function to use for picking whether a given point is over the oval section.
pub fn is_over_section_widget(widget: &graph::Container, p: Point, _: &Theme) -> widget::IsOver {
    widget
        .state_and_style::<State<Section>, Style>()
        .map(|unique| {
            let res = unique.state.resolution;
            let offset_rad = unique.state.section.offset_radians;
            let rad = unique.state.section.radians;
            let circumference = Circumference::new_section(widget.rect, res, rad)
                .offset_radians(offset_rad);
            is_over_section(circumference, p)
        })
        .unwrap_or_else(|| widget.rect.is_over(p))
        .into()
}
