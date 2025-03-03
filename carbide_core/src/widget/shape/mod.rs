use dyn_clone::DynClone;
use std::fmt::Debug;

use crate::draw::shape::DrawShape;
use crate::widget::{AnyWidget, WidgetId};
pub use capsule::*;
pub use circle::*;
pub use ellipse::*;
pub use rectangle::*;
pub use rounded_rectangle::*;

mod capsule;
mod circle;
mod ellipse;
mod rectangle;
mod rounded_rectangle;

pub trait AnyShape: Debug + DynClone + 'static {
    fn cache_key(&self) -> Option<WidgetId>; // TODO: ShapeId
    fn description(&self) -> DrawShape;
}

dyn_clone::clone_trait_object!(AnyShape);

pub trait Shape: AnyShape + Clone + private::Sealed {}
impl<T> Shape for T where T: AnyShape + Clone + private::Sealed {}

mod private {
    use crate::widget::AnyShape;

    // This disallows implementing Shape manually, and requires something to implement
    // AnyShape to implement Shape.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyShape {}
}