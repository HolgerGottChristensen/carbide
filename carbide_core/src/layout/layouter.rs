use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::draw::{Dimension, Position};
use crate::widget::Widget;

pub trait Layouter: Debug + DynClone {
    fn positioner(&self) -> fn(Position, Dimension, &mut dyn Widget);
}

dyn_clone::clone_trait_object!(Layouter);
