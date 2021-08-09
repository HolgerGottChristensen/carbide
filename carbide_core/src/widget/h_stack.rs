use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::widget::CrossAxisAlignment;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct HStack {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
}

impl HStack {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(HStack {
            id: Uuid::new_v4(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: 10.0,
        })
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        Box::new(self)
    }
}

impl Layout for HStack {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {

        // The number of children not containing any spacers
        let mut number_of_children_that_needs_sizing = self.children().filter(|m| m.flag() != Flags::SPACER).count() as f64;


        let non_spacers_vec: Vec<bool> = self.children().map(|n| n.flag() != Flags::SPACER).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length - 1).filter(|(n, b)| {
            **b && non_spacers_vec[n + 1]
        }).count() as f64;

        let spacing_total = (number_of_spaces) * self.spacing;
        let mut size_for_children = Dimension::new(requested_size.width - spacing_total, requested_size.height);

        let mut children_flexibilty: Vec<(u32, &mut dyn Widget)> = self.children_mut().filter(|m| m.flag() != Flags::SPACER).map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a, _), (b, _)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_height = 0.0;
        let mut total_width = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = Dimension::new(size_for_children.width / number_of_children_that_needs_sizing, size_for_children.height);
            let chosen_size = child.calculate_size(size_for_child, env);

            if chosen_size.height > max_height {
                max_height = chosen_size.height;
            }

            size_for_children = Dimension::new((size_for_children.width - chosen_size.width).max(0.0), size_for_children.height);

            number_of_children_that_needs_sizing -= 1.0;

            total_width += chosen_size.width;
        }

        let spacer_count = self.children().filter(|m| m.flag() == Flags::SPACER).count() as f64;
        let rest_space = requested_size.width - total_width - spacing_total;

        for spacer in self.children_mut().filter(|m| m.flag() == Flags::SPACER) {
            let chosen_size = spacer.calculate_size(Dimension::new(rest_space / spacer_count, requested_size.height), env);

            if chosen_size.height > max_height {
                max_height = chosen_size.height;
            }

            total_width += chosen_size.width;
        }

        self.dimension = Dimension::new(total_width + spacing_total, max_height);

        self.dimension
    }

    fn position_children(&mut self) {
        let cross_axis_alignment = CrossAxisAlignment::Center;
        let mut width_offset = 0.0;
        let position = self.position;
        let dimension = self.dimension;
        let spacing = self.spacing;

        let spacers: Vec<bool> = self.children().map(|n| n.flag() == Flags::SPACER).collect();

        for (n, child) in &mut self.children_mut().enumerate() {
            match cross_axis_alignment {
                CrossAxisAlignment::Start => { child.set_y(position.y) }
                CrossAxisAlignment::Center => { child.set_y(position.y + dimension.height / 2.0 - child.height() / 2.0) }
                CrossAxisAlignment::End => { child.set_y(position.y + dimension.height - child.height()) }
            }

            child.set_x(position.x + width_offset);

            if child.flag() != Flags::SPACER && n < spacers.len() - 1 && !spacers[n + 1] {
                width_offset += spacing;
            }
            width_offset += child.width();


            child.position_children();
        }
    }
}

impl CommonWidget for HStack {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
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

    fn flexibility(&self) -> u32 {
        1
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for HStack {}
