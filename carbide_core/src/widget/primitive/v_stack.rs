use crate::layout::CrossAxisAlignment;
use crate::prelude::*;
use crate::widget::ChildRender;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct VStack<GS> where GS: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl<GS: GlobalState> WidgetExt<GS> for VStack<GS> {}

impl<S: GlobalState> VStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Self> {
        Box::new(VStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            spacing: 10.0,
            cross_axis_alignment: CrossAxisAlignment::Center,
        })
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        Box::new(self)
    }
}

impl<GS: GlobalState> Layout<GS> for VStack<GS> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment<GS>) -> Dimensions {
        let mut number_of_children_that_needs_sizing = self.children.len() as f64;

        let non_spacers_vec: Vec<bool> = self.get_children().map(|n| n.get_flag() != Flags::SPACER).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length.max(1) - 1).filter(|(n, b)| {
            **b && non_spacers_vec[n + 1]
        }).count() as f64;

        let spacing_total = (number_of_spaces) * self.spacing;
        let mut size_for_children = [requested_size[0], requested_size[1] - spacing_total];

        let mut children_flexibilty: Vec<(u32, &mut dyn Widget<GS>)> = self.get_children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a, _), (b, _)| a.cmp(&b));
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

        let spacer_count = self.get_children().filter(|m| m.get_flag() == Flags::SPACER).count() as f64;
        let rest_space = requested_size[1] - total_height - spacing_total;

        for spacer in self.get_children_mut().filter(|m| m.get_flag() == Flags::SPACER) {
            let chosen_size = spacer.calculate_size([requested_size[0], rest_space / spacer_count], env);

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

        let spacers: Vec<bool> = self.get_children().map(|n| n.get_flag() == Flags::SPACER).collect();

        for (n, child) in self.get_children_mut().enumerate() {
            match alignment {
                CrossAxisAlignment::Start => { child.set_x(position[0]) }
                CrossAxisAlignment::Center => { child.set_x(position[0] + dimension[0] / 2.0 - child.get_width() / 2.0) }
                CrossAxisAlignment::End => { child.set_x(position[0] + dimension[0] - child.get_width()) }
            }

            child.set_y(position[1] + height_offset);

            if child.get_flag() != Flags::SPACER && n < spacers.len() - 1 && !spacers[n + 1] {
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

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        self.children
            .iter()
            .map(|x| x.deref())
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        self.children
            .iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
            .map(|x| x.deref_mut())
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

impl<S: GlobalState> ChildRender for VStack<S> {}


