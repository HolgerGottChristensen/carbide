use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::prelude::Environment;
use crate::widget::common_widget::CommonWidget;

pub trait Layout {
    /// 0 is the most flexible and the largest number is the least flexible
    fn flexibility(&self) -> u32;
    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions;
    fn position_children(&mut self);
}

pub trait SingleChildLayout {
    fn flexibility(&self) -> u32;
}

impl<T> Layout for T where T: SingleChildLayout + CommonWidget {
    fn flexibility(&self) -> u32 {
        self.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        let mut dimentions = [0.0, 0.0];
        if let Some(child) = self.get_children_mut().next() {
            dimentions = child.calculate_size(requested_size, env);
        }

        self.set_dimension(dimentions);

        self.get_dimension()
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.get_position();
        let dimension = self.get_dimension();

        if let Some(child) = self.get_children_mut().next() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}