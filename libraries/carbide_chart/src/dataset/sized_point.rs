use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::{DataPoint, PointValue};
use crate::{DataValue, Value};
use crate::SequenceItem;

/// A data point with a size next to its x and y, such as a bubble's value. Marks that draw sizes
/// (`SizedDataSetExt`) exist only on datasets of such points.
///
/// Being a `DataPoint`, it still has an x and a y, so a dataset of sized points works everywhere
/// any dataset does: a scatter controller without a mark draws it as plain points.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a sized point",
    label = "has no size",
    note = "sized points are `(x, y, z)` tuples, whose size is z, `Bubble {{ x, y, size }}`, and types of your own implementing `SizedPoint`",
)]
pub trait SizedPoint: DataPoint {
    /// The point's size, or `None` when it is missing.
    fn size(&self) -> Option<Scalar>;
}

/// A tuple's third value is its size.
impl<X: DataValue, Y: DataValue, Z: DataValue> SizedPoint for (X, Y, Z) {
    fn size(&self) -> Option<Scalar> {
        self.2.value().and_then(Value::number)
    }
}

/// A point at `x` and `y` with a `size`, such as a bubble. Its z is its size.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bubble<X = Scalar, Y = Scalar, S = Scalar> {
    pub x: X,
    pub y: Y,
    pub size: S,
}

impl<X: DataValue, Y: DataValue, S: DataValue> DataPoint for Bubble<X, Y, S> {
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.y.value()
    }

    fn z(&self) -> Option<Value<'_>> {
        self.size.value()
    }

    fn describe(&self, f: &mut dyn FnMut(PointValue<'_>)) {
        if let Some(y) = self.y.value() {
            f(PointValue::Value { name: None, value: y });
        }
        if let Some(size) = self.size.value() {
            f(PointValue::Other { name: "size", value: size });
        }
    }
}

impl<X: DataValue, Y: DataValue, S: DataValue> SizedPoint for Bubble<X, Y, S> {
    fn size(&self) -> Option<Scalar> {
        self.size.value().and_then(Value::number)
    }
}

/// A sized point is an item of a sequence of sized points, so every collection and state of them
/// is one, next to being a sequence of plain data points.
impl<P: SizedPoint + Clone + Debug + 'static> SequenceItem<dyn SizedPoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn SizedPoint + 'static))) {
        f(index, self);
    }
}
