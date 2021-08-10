use crate::draw::{Dimension, Position};
use crate::widget::Widget;

pub trait Layouter {
    fn positioner(&self) -> fn(Position, Dimension, &mut dyn Widget);
}
