use {Color, Colorable, Point, Rect, Sizeable, OldWidget};
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
use widget::primitive::Widget;
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
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use state::state::{StateList, DefaultState};


/// A basic, non-interactive rectangle shape widget.
#[derive(Debug)]
pub struct VStack {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar
}

impl VStack {
    pub fn initialize(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(VStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            spacing: 10.0
        })
    }
}

impl Event for VStack {
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

impl WidgetExt for VStack {}

impl Layout for VStack {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        let mut number_of_children_that_needs_sizing = self.children.len() as f64;
        let mut size_for_children = [requested_size[0], requested_size[1] - ((number_of_children_that_needs_sizing - 1.0)*self.spacing)];

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget>)> = self.children.iter_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut total_height = 0.0;
        let mut min_chosen_width = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = [size_for_children[0], size_for_children[1] / number_of_children_that_needs_sizing];
            let chosen_size = child.calculate_size(size_for_child, fonts);

            if chosen_size[0] > max_width {
                max_width = chosen_size[0];
            }

            if chosen_size[0] < min_chosen_width {
                min_chosen_width = chosen_size[0];
            }

            size_for_children = [size_for_children[0], (size_for_children[1] - chosen_size[1]).max(0.0)];

            number_of_children_that_needs_sizing -= 1.0;

            total_height += chosen_size[1];
        }

        self.dimension = [max_width, total_height + ((self.children.len() as f64 - 1.0) * self.spacing)];

        self.dimension


    }

    fn position_children(&mut self) {
        let cross_axis_alignment = CrossAxisAlignment::Center;
        let mut height_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;

        for child in &mut self.children {
            match cross_axis_alignment {
                CrossAxisAlignment::Start => {child.set_x(position[0])}
                CrossAxisAlignment::Center => {child.set_x(position[0] + dimension[0]/2.0 - child.get_width()/2.0)}
                CrossAxisAlignment::End => {child.set_x(position[0] + dimension[0] - child.get_width())}
            }

            child.set_y(position[1]+height_offset);

            height_offset += spacing;
            height_offset += child.get_height();


            child.position_children();
        }
    }
}

impl CommonWidget for VStack {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
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

impl Render for VStack {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


