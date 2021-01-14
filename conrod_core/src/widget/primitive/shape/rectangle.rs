//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.






use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use crate::{Color, Colorable, Point, Rect, Sizeable};
use crate::{Scalar, widget};
use crate::text;
use crate::draw::shape::triangle::Triangle;
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::render::primitive_kind::PrimitiveKind;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};

use super::Style as Style;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, WidgetCommon_)]
pub struct Rectangle<S> {
    id: Uuid,
    children: Vec<Box<dyn Widget<S>>>,
    position: Point,
    dimension: Dimensions,
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Rectangle**.
    pub style: Style,
    color: Color
}

impl<S: 'static + Clone> WidgetExt<S> for Rectangle<S> {}

impl<S> NoEvents for Rectangle<S> {}

impl<S> NoLocalStateSync for Rectangle<S> {}

impl<S> Layout<S> for Rectangle<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
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

impl<S> CommonWidget<S> for Rectangle<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
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

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
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

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
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

impl<S> Render<S> for Rectangle<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: self.color},
                scizzor: Rect::new(self.position, self.dimension),
                rect: Rect::new(self.position, self.dimension)
            }
        ];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

/// Unique state for the Rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    kind: Kind,
}

/// Whether the rectangle is drawn as an outline or a filled color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the outline of the rectangle is drawn.
    Outline,
    /// The rectangle area is filled with some color.
    Fill,
}


impl<S> Rectangle<S> {

    pub fn fill(mut self, color: Color) -> Box<Self> {
        self.color = color;
        Box::new(self)
    }

    #[cfg(not(debug_assertions))]
    pub fn debug_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        vec![]
    }

    #[cfg(debug_assertions)]
    pub fn debug_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new([l,b], [width, rect.h()]);
        let right_border = Rect::new([r-width,b], [width, rect.h()]);
        let top_border = Rect::new([l+width,b], [rect.w()-width*2.0, width]);
        let bottom_border = Rect::new([l+width,t-width], [rect.w()-width*2.0, width]);

        let border_color = Color::Rgba(0.0 / 255.0, 255.0 / 255.0, 251.0 / 255.0, 1.0);//Color::random();
        vec![
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: left_border,
                rect: left_border
            },
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: right_border,
                rect: right_border
            },
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: top_border,
                rect: top_border
            },
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: border_color.clone()},
                scizzor: bottom_border,
                rect: bottom_border
            },
        ]
    }

    /// Build a rectangle with the dimensions and style.
    pub fn styled(_dim: Dimensions, style: Style) -> Self {
        Rectangle {
            id: Uuid::new_v4(),
            children: vec![],
            position: [1.0, 1.0],
            dimension: [1.0, 1.0],
            common: widget::CommonBuilder::default(),
            style,
            color: Color::random(),
        }//.wh(dim)
    }

    /// Build a new filled rectangle.
    pub fn fill_old(dim: Dimensions) -> Self {
        Rectangle::styled(dim, Style::fill())
    }

    /// Build a new filled rectangle widget filled with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Rectangle::styled(dim, Style::fill_with(color))
    }

    /// Build a new outlined rectangle widget.
    pub fn outline(dim: Dimensions) -> Self {
        Rectangle::styled(dim, Style::outline())
    }

    /// Build an outlined rectangle rather than a filled one.
    pub fn outline_styled(dim: Dimensions, line_style: widget::line::Style) -> Self {
        Rectangle::styled(dim, Style::outline_styled(line_style))
    }

    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Rectangle<S>> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            common: widget::CommonBuilder::default(),
            style: Style::fill(),
            color: Color::random()
        })
    }

    pub fn new(position: Point, dimension: Dimensions, children: Vec<Box<dyn Widget<S>>>) -> Box<Rectangle<S>> {
        Box::new(Rectangle {
            id: Uuid::new_v4(),
            children,
            position,
            dimension,
            common: widget::CommonBuilder::default(),
            style: Style::fill(),
            color: Color::random()
        })
    }
}

/*impl<S> OldWidget<S> for Rectangle<S> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            kind: Kind::Fill,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, style, .. } = args;

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.kind != kind {
            state.update(|state| state.kind = kind);
        }
    }

}
*/

impl<S> Colorable for Rectangle<S> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}


/// The two triangles that describe the given `Rect`.
pub fn triangles(rect: Rect) -> (Triangle<Point>, Triangle<Point>) {
    let (l, r, b, t) = rect.l_r_b_t();
    let quad = [[l, t], [r, t], [r, b], [l, b]];
    widget::triangles::from_quad(quad)
}
