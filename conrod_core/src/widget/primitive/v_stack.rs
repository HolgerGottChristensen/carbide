use uuid::Uuid;

use crate::{Color, Colorable, Point, Rect, Sizeable};
use crate::Scalar;
use crate::text;
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::{CrossAxisAlignment, Layout};
use crate::position::Dimensions;
use crate::render::primitive::Primitive;
use crate::state::environment::Environment;
use crate::state::state_sync::NoLocalStateSync;
use crate::widget::Rectangle;
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::Render;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::state::global_state::GlobalState;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone)]
pub struct VStack<S> where S: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<S>>>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment
}

impl<S: GlobalState> Widget<S> for VStack<S> {}

impl<S: GlobalState> VStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Self> {
        Box::new(VStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            spacing: 10.0,
            cross_axis_alignment: CrossAxisAlignment::Center
        })
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self>{
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        Box::new(self)
    }
}

impl<S: GlobalState> NoEvents for VStack<S> {}

impl<S: GlobalState> NoLocalStateSync for VStack<S> {}

impl<S: GlobalState> WidgetExt<S> for VStack<S> {}

impl<S: GlobalState> Layout<S> for VStack<S> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
        let mut number_of_children_that_needs_sizing = self.children.len() as f64;

        let non_spacers_vec: Vec<bool> = self.get_children().map(|n| n.get_flag() != Flags::Spacer).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length.max(1) - 1).filter(|(n, b)| {
            **b && non_spacers_vec[n+1]
        }).count() as f64;

        let spacing_total = (number_of_spaces) * self.spacing;
        let mut size_for_children = [requested_size[0], requested_size[1] - spacing_total];

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget<S>>)> = self.get_children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut total_height = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = [size_for_children[0], size_for_children[1] / number_of_children_that_needs_sizing];
            let chosen_size = child.calculate_size(size_for_child, env);

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
            let chosen_size = spacer.calculate_size([requested_size[0], rest_space/spacer_count], env);

            if chosen_size[0] > max_width {
                max_width = chosen_size[0];
            }

            total_height += chosen_size[1];
        }

        self.dimension = [max_width, total_height + spacing_total];

        self.dimension


    }

    fn position_children(&mut self) {
        let mut height_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;
        let alignment = self.cross_axis_alignment.clone();

        let spacers: Vec<bool> = self.get_children().map(|n| n.get_flag() == Flags::Spacer).collect();

        for (n, child) in self.get_children_mut().enumerate() {
            match alignment {
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

impl<S: GlobalState> CommonWidget<S> for VStack<S> {
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

impl<S: GlobalState> Render<S> for VStack<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


