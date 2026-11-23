use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::dataset::datapoint::{DataPoint, PointValue};
use crate::dataset::datavalue::{DataValue, Value};
use crate::sequence::SequenceItem;

/// A data point with the five statistics a box plot draws: the minimum, the first quartile, the
/// median, the third quartile and the maximum. Box plots (`BoxDataSetExt::box_plot`) exist only on
/// datasets of such points.
///
/// Being a `DataPoint`, it still has an x and a y, so a dataset of box points works everywhere any
/// dataset does: a bar controller without a mark draws its y as a plain bar.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a box point",
    label = "has no minimum, quartiles, median and maximum",
    note = "box points are `BoxStats {{ x, min, q1, median, q3, max }}`, and types of your own implementing `BoxPoint`",
)]
pub trait BoxPoint: DataPoint {
    /// The smallest value, where the lower whisker ends, or `None` when it is missing.
    fn min(&self) -> Option<Scalar>;

    /// The first quartile, where the box starts, or `None` when it is missing.
    fn q1(&self) -> Option<Scalar>;

    /// The median, drawn across the box, or `None` when it is missing.
    fn median(&self) -> Option<Scalar>;

    /// The third quartile, where the box ends, or `None` when it is missing.
    fn q3(&self) -> Option<Scalar>;

    /// The largest value, where the upper whisker ends, or `None` when it is missing.
    fn max(&self) -> Option<Scalar>;
}

/// The statistics of a sample at `x`, as a box plot draws them. Its y is the median.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoxStats<X = Scalar, V = Scalar> {
    pub x: X,
    pub min: V,
    pub q1: V,
    pub median: V,
    pub q3: V,
    pub max: V,
}

impl<X: DataValue, V: DataValue> DataPoint for BoxStats<X, V> {
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.median.value()
    }

    fn describe(&self, f: &mut dyn FnMut(PointValue<'_>)) {
        for (name, value) in [("min", &self.min), ("q1", &self.q1), ("median", &self.median), ("q3", &self.q3), ("max", &self.max)] {
            if let Some(value) = value.value() {
                f(PointValue::Value { name: Some(name), value });
            }
        }
    }
}

impl<X: DataValue, V: DataValue> BoxPoint for BoxStats<X, V> {
    fn min(&self) -> Option<Scalar> {
        self.min.value().and_then(Value::number)
    }

    fn q1(&self) -> Option<Scalar> {
        self.q1.value().and_then(Value::number)
    }

    fn median(&self) -> Option<Scalar> {
        self.median.value().and_then(Value::number)
    }

    fn q3(&self) -> Option<Scalar> {
        self.q3.value().and_then(Value::number)
    }

    fn max(&self) -> Option<Scalar> {
        self.max.value().and_then(Value::number)
    }
}

/// A box point is an item of a sequence of box points, so every collection and state of them is
/// one, next to being a sequence of plain data points.
impl<P: BoxPoint + Clone + Debug + 'static> SequenceItem<dyn BoxPoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn BoxPoint + 'static))) {
        f(index, self);
    }
}
