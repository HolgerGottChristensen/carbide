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
use layout::basic_layouter::BasicLayouter;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};


/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Debug)]
pub struct ZStack {
    id: Uuid,
    children: Vec<CWidget>,
    position: Point,
    dimension: Dimensions
}

impl ZStack {
    pub fn initialize(children: Vec<CWidget>) -> CWidget {
        CWidget::ZStack(ZStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0]
        })
    }
}

impl Event for ZStack {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        self.process_mouse_event_default(event, consumed);
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.process_keyboard_event_default(event);
    }
}

impl Layout for ZStack {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {

        let mut children_flexibilty: Vec<(u32, &mut CWidget)> = self.children.iter_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, child) in children_flexibilty {
            let chosen_size = child.calculate_size(requested_size, fonts);

            if (chosen_size[0] > max_width) {
                max_width = chosen_size[0];
            }

            if (chosen_size[1] > max_height) {
                max_height = chosen_size[1];
            }

        }

        self.dimension = [max_width, max_height];
        self.dimension

    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::TopLeading.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl CommonWidget for ZStack {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_children(&self) -> &Vec<CWidget> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<CWidget> {
        &mut self.children
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

impl Render for ZStack {
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


