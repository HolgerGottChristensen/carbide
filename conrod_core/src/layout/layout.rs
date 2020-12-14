use widget::common_widget::CommonWidget;
use position::Dimensions;
use Point;

pub trait Layout<S> {
    fn calculate_size(&self, widget: &mut CommonWidget<S>, dimensions: Dimensions);
    fn calculate_position(&self, widget: &mut CommonWidget<S>, point: Point, dimensions: Dimensions);
}