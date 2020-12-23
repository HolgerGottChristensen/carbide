//! A simple, non-interactive widget for drawing a single **Oval**.




use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use crate::{Color, Colorable, Point, Rect, Scalar, Theme};
use crate::{graph, text};
use crate::draw::shape::circumference::{Circumference, Triangles};
use crate::draw::shape::triangle::Triangle;
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;
use crate::render::util::new_primitive;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget;
use crate::widget::Rectangle;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};

use super::Style as Style;

/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Debug, Clone, WidgetCommon_)]
pub struct Oval<S, K> {
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

    pub children: Vec<Box<dyn Widget<K>>>
}

impl<K, S: 'static + Clone> NoEvents for Oval<S, K> {}

impl<S, K> NoLocalStateSync for Oval<S, K> {}

impl<S: 'static + Clone, K: 'static + Clone> WidgetExt<K> for Oval<S, K> {}

impl<S, K> Layout<K> for Oval<S, K> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment) -> Dimensions {
        for child in &mut self.children {
            child.calculate_size(requested_size, env);
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

impl<S: 'static + Clone, K> CommonWidget<K> for Oval<S, K> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<K> {
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

    fn get_children_mut(&mut self) -> WidgetIterMut<K> {
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

    fn get_proxied_children(&mut self) -> WidgetIterMut<K> {
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

impl<K, S: 'static + Clone> Render<K> for Oval<S, K> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let points = widget::oval::circumference(Rect::new(self.position, self.dimension), DEFAULT_RESOLUTION);
        let mut triangles: Vec<Triangle<Point>> = Vec::new();
        triangles.extend(points.triangles());
        let kind = PrimitiveKind::TrianglesSingleColor {
            color: self.color.to_rgb(),
            triangles,
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(node_index(0), kind, Rect::new(self.position, self.dimension), Rect::new(self.position, self.dimension))];
        prims.extend(Rectangle::<K>::rect_outline(Rect::new(self.position, self.dimension), 1.0));
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

/*impl OvalSection for Section {
    const IS_OVER: widget::IsOverFn = is_over_section_widget;
}*/

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

impl<S> Oval<Full, S> {

    pub fn fill(mut self, color: Color) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Oval<Full, S>> {
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

    pub fn new(position: Point, dimension: Dimensions, children: Vec<Box<dyn Widget<S>>>) -> Box<Oval<Full, S>> {
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
    pub fn styled(_dim: Dimensions, style: Style) -> Self {
        Oval {
            id: Uuid::new_v4(),
            common: widget::CommonBuilder::default(),
            style: style,
            resolution: DEFAULT_RESOLUTION,
            section: Full,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            color: Color::random(),
            children: vec![]
        }//.wh(dim)
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

impl<S, K> Oval<S, K> {
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
    pub fn section(self, radians: Scalar) -> Oval<Section, K> {
        let Oval { common, style, resolution, .. } = self;
        let section = Section { radians, offset_radians: 0.0 };
        Oval { id: Uuid::new_v4(), common, style, resolution, section, position: [10.0, 10.0], dimension: [10.0,10.0], color: Color::random(), children: vec![] }
    }
}

impl<K> Oval<Section, K> {
    /// The radians at which the section will begin.
    ///
    /// A value of `0.0` will begin at the rightmost point of the oval.
    pub fn offset_radians(mut self, offset_radians: Scalar) -> Self {
        self.section.offset_radians = offset_radians;
        self
    }
}

/*impl<S, K> OldWidget<K> for Oval<S, K>
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

    fn update(self, args: widget::UpdateArgs<Self, S>) -> Self::Event {
        let widget::UpdateArgs { state, .. } = args;
        if state.resolution != self.resolution {
            state.update(|state| state.resolution = self.resolution);
        }
        if state.section != self.section {
            state.update(|state| state.section = self.section);
        }
    }
}*/

impl<S, K> Colorable for Oval<S, K> {
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

/*/// The function to use for picking whether a given point is over the oval section.
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
}*/
