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
use widget::layout::Layout;
use text::font::Map;
use layout::CrossAxisAlignment;


/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Debug)]
pub struct HStack {
    id: Uuid,
    children: Vec<CWidget>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar
}

impl HStack {
    pub fn initialize(children: Vec<CWidget>) -> CWidget {
        CWidget::HStack(HStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            spacing: 10.0
        })
    }
}

impl Layout for HStack {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        let mut number_of_children_that_needs_sizing = self.children.len() as f64;
        let mut size_for_children = [requested_size[0] - ((number_of_children_that_needs_sizing - 1.0)*self.spacing), requested_size[1]];

        let mut children_flexibilty: Vec<(u32, &mut CWidget)> = self.children.iter_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_height = 0.0;
        let mut total_width = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = [size_for_children[0] / number_of_children_that_needs_sizing, size_for_children[1]];
            let chosen_size = child.calculate_size(size_for_child, fonts);

            if (chosen_size[1] > max_height) {
                max_height = chosen_size[1];
            }

            size_for_children = [(size_for_children[0] - chosen_size[0]).max(0.0), size_for_children[1]];

            number_of_children_that_needs_sizing -= 1.0;

            total_width += chosen_size[0];
        }

        self.dimension = [total_width + ((self.children.len() as f64 - 1.0) * self.spacing), max_height];

        self.dimension


    }

    fn position_children(&mut self) {
        let cross_axis_alignment = CrossAxisAlignment::Center;
        let mut width_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;

        for child in &mut self.children {
            match cross_axis_alignment {
                CrossAxisAlignment::Start => {child.set_y(position[1])}
                CrossAxisAlignment::Center => {child.set_y(position[1] + dimension[1]/2.0 - child.get_height()/2.0)}
                CrossAxisAlignment::End => {child.set_y(position[1] + dimension[1] - child.get_height())}
            }

            child.set_x(position[0]+width_offset);

            width_offset += spacing;
            width_offset += child.get_width();


            child.position_children();
        }
    }
}

impl CommonWidget for HStack {
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

    fn get_size(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        self.dimension[0]
    }

    fn get_height(&self) -> Scalar {
        self.dimension[1]
    }
}

impl Render for HStack {
    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &dyn Fn(&mut CommonWidget, Dimensions)) {
        unimplemented!()
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, proposed_dimensions: Dimensions, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(proposed_dimensions, fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


