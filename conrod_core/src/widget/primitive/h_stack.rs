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
use flags::Flags;
use widget::widget_iterator::{WidgetIterMut, WidgetIter};


/// A basic, non-interactive rectangle shape widget.
#[derive(Debug)]
pub struct HStack {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar
}

impl HStack {
    pub fn initialize(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(HStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            spacing: 10.0
        })
    }
}

impl WidgetExt for HStack {}

impl Event for HStack {
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

impl Layout for HStack {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {


        // The number of children not containing any spacers
        let mut number_of_children_that_needs_sizing = self.get_children().filter(|m| m.get_flag() != Flags::Spacer).count() as f64;


        let non_spacers_vec: Vec<bool> = self.get_children().map(|n| n.get_flag() != Flags::Spacer).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length -1).filter(|(n, b)| {
            **b && non_spacers_vec[n+1]
        }).count() as f64;

        let spacing_total = ((number_of_spaces)*self.spacing);
        let mut size_for_children = [requested_size[0] - spacing_total, requested_size[1]];

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget>)> = self.get_children_mut().filter(|m| m.get_flag() != Flags::Spacer).map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_height = 0.0;
        let mut total_width = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = [size_for_children[0] / number_of_children_that_needs_sizing, size_for_children[1]];
            let chosen_size = child.calculate_size(size_for_child, fonts);

            if chosen_size[1] > max_height {
                max_height = chosen_size[1];
            }

            size_for_children = [(size_for_children[0] - chosen_size[0]).max(0.0), size_for_children[1]];

            number_of_children_that_needs_sizing -= 1.0;

            total_width += chosen_size[0];
        }

        let spacer_count = self.get_children().filter(|m| m.get_flag() == Flags::Spacer).count() as f64;
        let rest_space = requested_size[0] - total_width - spacing_total;

        for spacer in self.get_children_mut().filter(|m| m.get_flag() == Flags::Spacer) {
            let chosen_size = spacer.calculate_size([rest_space/spacer_count, requested_size[1]], fonts);

            if chosen_size[1] > max_height {
                max_height = chosen_size[1];
            }

            total_width += chosen_size[0];
        }

        self.dimension = [total_width + spacing_total, max_height];

        self.dimension


    }

    fn position_children(&mut self) {
        let cross_axis_alignment = CrossAxisAlignment::Center;
        let mut width_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;

        let spacers: Vec<bool> = self.get_children().map(|n| n.get_flag() == Flags::Spacer).collect();

        for (n, child) in &mut self.get_children_mut().enumerate() {
            match cross_axis_alignment {
                CrossAxisAlignment::Start => {child.set_y(position[1])}
                CrossAxisAlignment::Center => {child.set_y(position[1] + dimension[1]/2.0 - child.get_height()/2.0)}
                CrossAxisAlignment::End => {child.set_y(position[1] + dimension[1] - child.get_height())}
            }

            child.set_x(position[0]+width_offset);

            if child.get_flag() != Flags::Spacer && n < spacers.len()-1 && !spacers[n+1] {
                width_offset += spacing;
            }
            width_offset += child.get_width();


            child.position_children();
        }
    }
}

impl CommonWidget for HStack {
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

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


