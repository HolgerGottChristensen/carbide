use layout::bounds::Bounds;
use widget::common_widget::CommonWidget;
use position::Dimensions;
use Point;

pub trait Layout {
    fn calculate_size(&self, widget: &mut CommonWidget, dimensions: Dimensions);
    fn calculate_position(&self, widget: &mut CommonWidget, point: Point, dimensions: Dimensions);
}