use {Color, Colorable, Point, Rect, Sizeable};
use ::{widget, Scalar};
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
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};


/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone)]
pub struct VStack<S> {
    id: Uuid,
    children: Vec<Box<dyn Widget<S>>>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar
}

impl<S> VStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Self> {
        Box::new(VStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            spacing: 10.0
        })
    }
}

impl<S> Event<S> for VStack<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList) {
        self.sync_state_default(states);
    }
}

impl<S: 'static + Clone> WidgetExt<S> for VStack<S> {}

impl<S> Layout for VStack<S> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        let mut number_of_children_that_needs_sizing = self.children.len() as f64;

        let non_spacers_vec: Vec<bool> = self.get_children().map(|n| n.get_flag() != Flags::Spacer).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length.max(1) - 1).filter(|(n, b)| {
            **b && non_spacers_vec[n+1]
        }).count() as f64;

        let spacing_total = ((number_of_spaces)*self.spacing);
        let mut size_for_children = [requested_size[0], requested_size[1] - spacing_total];

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget<S>>)> = self.get_children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut total_height = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = [size_for_children[0], size_for_children[1] / number_of_children_that_needs_sizing];
            let chosen_size = child.calculate_size(size_for_child, fonts);

            if chosen_size[0] > max_width {
                max_width = chosen_size[0];
            }

            size_for_children = [size_for_children[0], (size_for_children[1] - chosen_size[1]).max(0.0)];

            number_of_children_that_needs_sizing -= 1.0;

            total_height += chosen_size[1];
        }

        let spacer_count = self.get_children().filter(|m| m.get_flag() == Flags::Spacer).count() as f64;
        let rest_space = requested_size[1] - total_height - spacing_total;

        for spacer in self.get_children_mut().filter(|m| m.get_flag() == Flags::Spacer) {
            let chosen_size = spacer.calculate_size([requested_size[0], rest_space/spacer_count], fonts);

            if chosen_size[0] > max_width {
                max_width = chosen_size[0];
            }

            total_height += chosen_size[1];
        }

        self.dimension = [max_width, total_height + spacing_total];

        self.dimension


    }

    fn position_children(&mut self) {
        let cross_axis_alignment = CrossAxisAlignment::Center;
        let mut height_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;

        let spacers: Vec<bool> = self.get_children().map(|n| n.get_flag() == Flags::Spacer).collect();

        for (n, child) in self.get_children_mut().enumerate() {
            match cross_axis_alignment {
                CrossAxisAlignment::Start => {child.set_x(position[0])}
                CrossAxisAlignment::Center => {child.set_x(position[0] + dimension[0]/2.0 - child.get_width()/2.0)}
                CrossAxisAlignment::End => {child.set_x(position[0] + dimension[0] - child.get_width())}
            }

            child.set_y(position[1]+height_offset);

            if child.get_flag() != Flags::Spacer && n < spacers.len()-1 && !spacers[n+1] {
                height_offset += spacing;
            }
            height_offset += child.get_height();


            child.position_children();
        }
    }
}

impl<S> CommonWidget<S> for VStack<S> {
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

impl<S> Render<S> for VStack<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


