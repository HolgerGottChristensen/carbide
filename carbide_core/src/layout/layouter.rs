use crate::draw::{Dimension, Point, Position};
use crate::draw::Dimensions;
use crate::widget::Widget;

pub trait Layouter {
    fn position(&self) -> fn(Position, Dimension, &mut dyn Widget);
}