use widget::common_widget::CommonWidget;
use Point;
use position::Dimensions;

pub trait Layouter<S> {
    fn position(&self) -> fn(Point, Dimensions, &mut dyn CommonWidget<S>);
}