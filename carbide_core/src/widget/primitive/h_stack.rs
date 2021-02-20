use crate::prelude::*;
use crate::layout::CrossAxisAlignment;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct HStack<GS> where GS: GlobalState {
    id: Uuid,
    children: Vec<Box<dyn Widget<GS>>>,
    position: Point,
    dimension: Dimensions,
    spacing: Scalar,
}

impl<GS: GlobalState> WidgetExt<GS> for HStack<GS> {}

impl<S: GlobalState> HStack<S> {
    pub fn initialize(children: Vec<Box<dyn Widget<S>>>) -> Box<Self> {
        Box::new(HStack {
            id: Uuid::new_v4(),
            children,
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            spacing: 10.0
        })
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        Box::new(self)
    }
}

impl<S: GlobalState> Layout<S> for HStack<S> {
    fn flexibility(&self) -> u32 {
        1
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {

        // The number of children not containing any spacers
        let mut number_of_children_that_needs_sizing = self.get_children().filter(|m| m.get_flag() != Flags::SPACER).count() as f64;


        let non_spacers_vec: Vec<bool> = self.get_children().map(|n| n.get_flag() != Flags::SPACER).collect();
        let non_spacers_vec_length = non_spacers_vec.len();

        let number_of_spaces = non_spacers_vec.iter().enumerate().take(non_spacers_vec_length -1).filter(|(n, b)| {
            **b && non_spacers_vec[n+1]
        }).count() as f64;

        let spacing_total = (number_of_spaces) * self.spacing;
        let mut size_for_children = [requested_size[0] - spacing_total, requested_size[1]];

        let mut children_flexibilty: Vec<(u32, &mut Box<dyn Widget<S>>)> = self.get_children_mut().filter(|m| m.get_flag() != Flags::SPACER).map(|child| (child.flexibility(), child)).collect();
        children_flexibilty.sort_by(|(a,_), (b,_)| a.cmp(&b));
        children_flexibilty.reverse();

        let mut max_height = 0.0;
        let mut total_width = 0.0;

        for (_, child) in children_flexibilty {
            let size_for_child = [size_for_children[0] / number_of_children_that_needs_sizing, size_for_children[1]];
            let chosen_size = child.calculate_size(size_for_child, env);

            if chosen_size[1] > max_height {
                max_height = chosen_size[1];
            }

            size_for_children = [(size_for_children[0] - chosen_size[0]).max(0.0), size_for_children[1]];

            number_of_children_that_needs_sizing -= 1.0;

            total_width += chosen_size[0];
        }

        let spacer_count = self.get_children().filter(|m| m.get_flag() == Flags::SPACER).count() as f64;
        let rest_space = requested_size[0] - total_width - spacing_total;

        for spacer in self.get_children_mut().filter(|m| m.get_flag() == Flags::SPACER) {
            let chosen_size = spacer.calculate_size([rest_space/spacer_count, requested_size[1]], env);

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

        let spacers: Vec<bool> = self.get_children().map(|n| n.get_flag() == Flags::SPACER).collect();

        for (n, child) in &mut self.get_children_mut().enumerate() {
            match cross_axis_alignment {
                CrossAxisAlignment::Start => {child.set_y(position[1])}
                CrossAxisAlignment::Center => {child.set_y(position[1] + dimension[1]/2.0 - child.get_height()/2.0)}
                CrossAxisAlignment::End => {child.set_y(position[1] + dimension[1] - child.get_height())}
            }

            child.set_x(position[0]+width_offset);

            if child.get_flag() != Flags::SPACER && n < spacers.len()-1 && !spacers[n+1] {
                width_offset += spacing;
            }
            width_offset += child.get_width();


            child.position_children();
        }
    }
}

impl<S: GlobalState> CommonWidget<S> for HStack<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        self.children
            .iter()
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
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut<S> {
        self.children.iter_mut()
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

impl<S: GlobalState> Render<S> for HStack<S> {

    fn get_primitives(&mut self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}


