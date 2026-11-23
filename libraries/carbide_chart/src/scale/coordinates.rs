use carbide::draw::Scalar;

use crate::{DataPoint, Value};
use crate::scale::{Placement, ScaleId, Scales};

/// Where a value is placed along a scale: a number or a time at itself, and a category's name at
/// the category's index on the scale.
pub(crate) trait Places {
    fn place(&self, id: &ScaleId, value: Value) -> Option<Scalar>;

    /// Whether the scale `id` is radial, so a dataset on two of them is read as a polar dataset
    /// (see `Orientation::for_dataset`).
    fn is_radial(&self, id: &ScaleId) -> bool;
}

/// While the bounds are collected, a name is placed at its category's index among the categories
/// registered before collecting (see `ScaleBounds::register_values`); a name that wasn't has no
/// place.
impl Places for Placement {
    fn place(&self, id: &ScaleId, value: Value) -> Option<Scalar> {
        match value {
            Value::Number(number) | Value::Time(number) => Some(number),
            Value::Text(text) => self.category_index(id, text).map(|index| index as Scalar),
        }
    }

    fn is_radial(&self, id: &ScaleId) -> bool {
        Placement::is_radial(self, id)
    }
}

/// While drawing, a name is placed at its category's index on the fitted scale; on a scale
/// without that category, or without categories, it isn't placed.
impl Places for Scales {
    fn place(&self, id: &ScaleId, value: Value) -> Option<Scalar> {
        match value {
            Value::Number(number) | Value::Time(number) => self.get(id)?.place(number),
            Value::Text(text) => self.get(id)?.category_index(text),
        }
    }

    fn is_radial(&self, id: &ScaleId) -> bool {
        self.get(id).is_some_and(|scale| scale.axis() == crate::scale::Axis::Radial)
    }
}

/// Reads a dataset's points as numbers along the scales they are placed on: a number or a time is
/// itself, and a category's name is its category's index on the scale (see `Value`). Controllers
/// and marks read every point's x and y through it, so names work wherever numbers do.
///
/// `x` is placed on the scale the point's x goes on, which is the y scale for horizontal bars and
/// lines, and `y` on the scale its y goes on.
#[derive(Copy, Clone)]
pub struct Coordinates<'a> {
    places: &'a dyn Places,
    x_scale: &'a ScaleId,
    y_scale: &'a ScaleId,
}

impl<'a> Coordinates<'a> {
    pub(crate) fn new(places: &'a dyn Places, x_scale: &'a ScaleId, y_scale: &'a ScaleId) -> Coordinates<'a> {
        Coordinates { places, x_scale, y_scale }
    }

    /// Where the point's x is placed, or `None` when it is missing or has no place on the scale.
    pub fn x<P: DataPoint + ?Sized>(&self, point: &P) -> Option<Scalar> {
        self.places.place(self.x_scale, point.x()?)
    }

    /// Where the point's y is placed, or `None` when it is missing or has no place on the scale.
    pub fn y<P: DataPoint + ?Sized>(&self, point: &P) -> Option<Scalar> {
        self.places.place(self.y_scale, point.y()?)
    }
}
