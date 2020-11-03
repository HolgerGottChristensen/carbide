//! A simple, non-interactive rectangle shape widget.
//!
//! Due to the frequency of its use in GUIs, the `Rectangle` gets its own widget to allow backends
//! to specialise their rendering implementations.
use {Color, Colorable, Point, Rect, Sizeable, Widget};
use super::Style as Style;
use ::{widget, Scalar};
use widget::triangles::Triangle;
use widget::render::Render;
use graph::Container;
use widget::Id;
use color::rgb;
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::primitive::CWidget;
use position::Dimensions;
use daggy::petgraph::graph::node_index;
use ::{Range, text};
use render::owned_primitive::OwnedPrimitive;
use render::owned_primitive_kind::OwnedPrimitiveKind;
use widget::envelope_editor::EnvelopePoint;
use widget::primitive::shape::triangles::Vertex;
use std::convert::TryFrom;
use std::error::Error;
use std::collections::HashMap;
use std::any::Any;



/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Debug, WidgetCommon_)]
pub struct Rectangle {
    id: Uuid,
    children: Vec<CWidget>,
    position: Point,
    dimension: Dimensions,
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Rectangle**.
    pub style: Style,
}

impl CommonWidget for Rectangle {
    fn get_id(&self) -> Uuid {
        self.id
    }
    fn get_children(&self) -> &Vec<CWidget> {
        &self.children
    }

    fn get_position(&self) -> Point {
        unimplemented!()
    }

    fn get_x(&self) -> Scalar {
        unimplemented!()
    }

    fn get_y(&self) -> Scalar {
        unimplemented!()
    }

    fn get_size(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        unimplemented!()
    }

    fn get_height(&self) -> Scalar {
        unimplemented!()
    }
}

impl Render for Rectangle {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        let kind = PrimitiveKind::Rectangle { color: rgb(0.0,1.0, 0.0)};
        return Some(new_primitive(id, kind, clip, container.rect));
    }

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![
            Primitive {
                id: node_index(0),
                kind: PrimitiveKind::Rectangle { color: Color::random()},
                scizzor: Rect::new(self.position, self.dimension),
                rect: Rect::new(self.position, self.dimension)
            }
        ];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(fonts)).collect();
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


impl Rectangle {

    pub fn rect_outline(rect: Rect, width: Scalar) -> Vec<Primitive> {
        let (l, r, b, t) = rect.l_r_b_t();

        let left_border = Rect::new([l,b], [width, rect.w()]);
        let right_border = Rect::new([r-width,b], [width, rect.w()]);
        let top_border = Rect::new([l+width,b], [rect.w()-width*2.0, width]);
        let bottom_border = Rect::new([l+width,t-width], [rect.w()-width*2.0, width]);

        let border_color = Color::random();
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
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Rectangle {
            id: Uuid::new_v4(),
            children: vec![],
            position: [1.0, 1.0],
            dimension: [1.0, 1.0],
            common: widget::CommonBuilder::default(),
            style: style,
        }.wh(dim)
    }

    /// Build a new filled rectangle.
    pub fn fill(dim: Dimensions) -> Self {
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



    pub fn new(position: Point, dimension: Dimensions, children: Vec<CWidget>) -> CWidget {
        CWidget::Rectangle(Rectangle {
            id: Uuid::new_v4(),
            children,
            position,
            dimension,
            common: widget::CommonBuilder::default(),
            style: Style::fill()
        })
    }
}

impl Widget for Rectangle {
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


impl Colorable for Rectangle {
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
