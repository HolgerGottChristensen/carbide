use {Color, Colorable, Point, Rect, Sizeable, Widget};
use ::{widget, Scalar};
use widget::triangles::Triangle;
use widget::render::Render;
use graph::Container;
use widget::{Id, Rectangle};
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
pub struct VStack {
    id: Uuid,
    children: Vec<CWidget>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar,
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder
}

impl CommonWidget for VStack {
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

    fn set_x(&mut self, x: f64) {
        self.position = Point::new(x, self.position.get_y());
    }

    fn get_y(&self) -> Scalar {
        unimplemented!()
    }

    fn set_y(&mut self, y: f64) {
        self.position = Point::new(self.position.get_x(), y);
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

impl Render for VStack {
    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &dyn Fn(&mut CommonWidget, Dimensions)) {
        let dimension = self.dimension.clone();
        let number_of_children = self.children.len() as f64;
        let total_proposed_size = [proposed_size[0], proposed_size[1]-(number_of_children*self.spacing)];
        let mut current_proposed_size = [total_proposed_size[0], total_proposed_size[1] / number_of_children];
        let mut rest_current_proposed_size = [total_proposed_size[0], total_proposed_size[1] - current_proposed_size[1]];
        let mut total_height = 0.0;

        for child in &mut self.children {
            total_height += child.calc_height(current_proposed_size[1]);
        }





        for child in &mut self.children {
            child.layout(proposed_size, fonts, positioner)
        }

        positioner(self, dimension);
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, proposed_dimensions: Dimensions, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(proposed_dimensions, fonts)).collect();
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


impl VStack {


    pub fn initialize(dimension: Dimensions, children: Vec<CWidget>) -> CWidget {
        CWidget::VStack(VStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension,
            spacing: 10.0,
            common: widget::CommonBuilder::default()
        })
    }
}