use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::datavalue::{DataValue, ValueKind};
use crate::sequence::SequenceItem;

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
        self.2.value()
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
    fn x(&self) -> Option<Scalar> {
        self.x.value()
    }

    fn y(&self) -> Option<Scalar> {
        self.y.value()
    }

    fn x_kind(&self) -> ValueKind {
        X::KIND
    }

    fn y_kind(&self) -> ValueKind {
        Y::KIND
    }

    fn z(&self) -> Option<Scalar> {
        self.size.value()
    }
}

impl<X: DataValue, Y: DataValue, S: DataValue> SizedPoint for Bubble<X, Y, S> {
    fn size(&self) -> Option<Scalar> {
        self.size.value()
    }
}

/// A sized point is an item of a sequence of sized points, so every collection and state of them
/// is one, next to being a sequence of plain data points.
impl<P: SizedPoint + Clone + Debug + 'static> SequenceItem<dyn SizedPoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn SizedPoint + 'static))) {
        f(index, self);
    }
}
