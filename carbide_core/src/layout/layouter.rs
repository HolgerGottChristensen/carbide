use crate::Point;
use crate::position::Dimensions;
use crate::widget::Widget;

pub trait Layouter {
    fn position(&self) -> fn(Point, Dimensions, &mut dyn Widget);
}