//! A simple, non-interactive widget for drawing a single **Oval**.

use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;
use crate::widget;
use crate::draw::shape::triangle::Triangle;
use crate::render::util::new_primitive;
use crate::draw::shape::circumference::{Circumference, Triangles};


/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Debug, Clone, Widget)]
pub struct Oval<S, GS> where S: 'static + Clone, GS: GlobalState {
    pub id: Uuid,
    /// The number of lines used to draw the edge.
    pub resolution: usize,
    /// A type describing the section of the `Oval` that is to be drawn.
    pub section: S,
    position: Point,
    dimension: Dimensions,
    color: Color,

    pub children: Vec<Box<dyn Widget<GS>>>
}

impl<S: 'static + Clone, GS: GlobalState> WidgetExt<GS> for Oval<S, GS> {}

impl<S: 'static + Clone, GS: GlobalState> Layout<GS> for Oval<S, GS> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<GS>) -> Dimensions {
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

impl<S: 'static + Clone, K: GlobalState> CommonWidget<K> for Oval<S, K> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<K> {
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
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
                if x.get_flag() == Flags::PROXY {
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

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<K> {
        self.children.iter_mut()
            .fold(WidgetIterMut::Empty, |acc, x| {
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

impl<S: 'static + Clone, GS: GlobalState> Render<GS> for Oval<S, GS> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let points = widget::oval::circumference(Rect::new(self.position, self.dimension), DEFAULT_RESOLUTION);
        let mut triangles: Vec<Triangle<Point>> = Vec::new();
        triangles.extend(points.triangles());
        let kind = PrimitiveKind::TrianglesSingleColor {
            color: self.color.to_rgb(),
            triangles,
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(kind, Rect::new(self.position, self.dimension))];
        prims.extend(Rectangle::<GS>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


/// The entire `Oval` will be drawn.
///
/// To draw only a section of the oval, use the `section` builder method.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Full;

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

impl<S: GlobalState> Oval<Full, S> {

    pub fn fill(mut self, color: Color) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Oval<Full, S>> {
        Box::new(Oval {
            id: Uuid::new_v4(),
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
            resolution: 0,
            section: Full,
            color: Color::random()
        })
    }

}

impl<S: Clone, K: GlobalState> Oval<S, K> {
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
        let Oval { resolution, .. } = self;
        let section = Section { radians, offset_radians: 0.0 };
        Oval { id: Uuid::new_v4(), resolution, section, position: [10.0, 10.0], dimension: [10.0,10.0], color: Color::random(), children: vec![] }
    }
}

impl<K: GlobalState> Oval<Section, K> {
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
