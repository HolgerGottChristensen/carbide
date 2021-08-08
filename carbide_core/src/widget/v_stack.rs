use crate::draw::{Dimension, Position};
use crate::prelude::*;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct VStack {
    id: Id,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl VStack {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(VStack {
            id: Uuid::new_v4(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
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

impl Layout for VStack {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let mut number_of_children_that_needs_sizing = self.children.len() as f64;

        let non_spacers_vec: Vec<bool> = self.children().map(|n| n.flag() != Flags::SPACER).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length.max(1) - 1).filter(|(n, b)| {
            **b && non_spacers_vec[n + 1]
        }).count() as f64;

        let spacing_total = (number_of_spaces) * self.spacing;
        let mut size_for_children = Dimension::new(requested_size.width, requested_size.height - spacing_total);

        let mut children_flexibilty: Vec<(u32, &mut dyn Widget)> = self.children_mut().map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a, _), (b, _)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_width = 0.0;
        let mut total_height = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = Dimension::new(size_for_children.width, size_for_children.height / number_of_children_that_needs_sizing);
            let chosen_size = child.calculate_size(size_for_child, env);

            if chosen_size.width > max_width {
                max_width = chosen_size.width;
            }

            size_for_children = Dimension::new(size_for_children.width, (size_for_children.height - chosen_size.height).max(0.0));

            number_of_children_that_needs_sizing -= 1.0;

            total_height += chosen_size.height;
        }

        let spacer_count = self.children().filter(|m| m.flag() == Flags::SPACER).count() as f64;
        let rest_space = requested_size.height - total_height - spacing_total;

        for spacer in self.children_mut().filter(|m| m.flag() == Flags::SPACER) {
            let chosen_size = spacer.calculate_size(Dimension::new(requested_size.width, rest_space / spacer_count), env);

            if chosen_size.width > max_width {
                max_width = chosen_size.width;
            }

            total_height += chosen_size.height;
        }

        self.dimension = Dimension::new(max_width, total_height + spacing_total);

        self.dimension
    }

    fn position_children(&mut self) {
        let mut height_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;
        let alignment = self.cross_axis_alignment.clone();

        let spacers: Vec<bool> = self.children().map(|n| n.flag() == Flags::SPACER).collect();

        for (n, child) in self.children_mut().enumerate() {
            match alignment {
                CrossAxisAlignment::Start => { child.set_x(position.x) }
                CrossAxisAlignment::Center => { child.set_x(position.x + dimension.width / 2.0 - child.width() / 2.0) }
                CrossAxisAlignment::End => { child.set_x(position.x + dimension.width - child.width()) }
            }

            child.set_y(position.y + height_offset);

            if child.flag() != Flags::SPACER && n < spacers.len() - 1 && !spacers[n + 1] {
                height_offset += spacing;
            }
            height_offset += child.height();


            child.position_children();
        }
    }
}

impl CommonWidget for VStack {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        self.children
            .iter()
            .map(|x| x.deref())
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        self.children.iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        self.children.iter_mut()
            .map(|x| x.deref_mut())
            .fold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }


    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl WidgetExt for VStack {}
