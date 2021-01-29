use crate::Point;
use crate::position::Dimensions;
use crate::widget::common_widget::CommonWidget;

pub trait Layouter<S> {
    fn position(&self) -> fn(Point, Dimensions, &mut dyn CommonWidget<S>);
}