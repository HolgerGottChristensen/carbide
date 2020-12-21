use position::Dimensions;
use text;
use text::font::Map;
use widget::common_widget::CommonWidget;
use layout::basic_layouter::BasicLayouter;
use layout::layouter::Layouter;
use state::environment::Environment;

pub trait Layout<U> {
    /// 0 is the most flexible and the largest number is the least flexible
    fn flexibility(&self) -> u32;
    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment) -> Dimensions;
    fn position_children(&mut self);
}

pub trait SingleChildLayout {
    fn flexibility(&self) -> u32;
}

impl<T, U> Layout<U> for T where T: SingleChildLayout + CommonWidget<U> {
    fn flexibility(&self) -> u32 {
        self.flexibility()
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], env: &Environment) -> [f64; 2] {
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