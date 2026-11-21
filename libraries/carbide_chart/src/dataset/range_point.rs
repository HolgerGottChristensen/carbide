use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::datavalue::DataValue;
use crate::sequence::SequenceItem;

/// A data point with a low and a high value next to its x and y: the two ends of a floating bar,
/// or the error interval around a measured value. Marks that draw ranges (`RangeDataSetExt`)
/// exist only on datasets of such points.
///
/// Being a `DataPoint`, it still has an x and a y, so a dataset of range points works everywhere
/// any dataset does: a bar controller without a mark draws its y as a plain bar, and a line
/// controller draws a line through its y values.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a range point",
    label = "has no low and high value",
    note = "range points are `Range {{ x, low, high }}`, `Estimate {{ x, y, low, high }}`, and types of your own implementing `RangePoint`",
)]
pub trait RangePoint: DataPoint {
    /// The low end of the range, or `None` when it is missing.
    fn low(&self) -> Option<Scalar>;

    /// The high end of the range, or `None` when it is missing.
    fn high(&self) -> Option<Scalar>;
}

/// A range from `low` to `high` at `x`, such as a floating bar. Its y is `high`, so it is the
/// far end of a plain bar when drawn without a mark.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range<X = Scalar, V = Scalar> {
    pub x: X,
    pub low: V,
    pub high: V,
}

impl<X: DataValue, V: DataValue> DataPoint for Range<X, V> {
    fn x(&self) -> Option<Scalar> {
        self.x.value()
    }

    fn y(&self) -> Option<Scalar> {
        self.high.value()
    }
}

impl<X: DataValue, V: DataValue> RangePoint for Range<X, V> {
    fn low(&self) -> Option<Scalar> {
        self.low.value()
    }

    fn high(&self) -> Option<Scalar> {
        self.high.value()
    }
}

/// A value `y` at `x`, with the interval from `low` to `high` it is known within, such as a
/// measurement and its error.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Estimate<X = Scalar, V = Scalar> {
    pub x: X,
    pub y: V,
    pub low: V,
    pub high: V,
}

impl<X: DataValue, V: DataValue> DataPoint for Estimate<X, V> {
    fn x(&self) -> Option<Scalar> {
        self.x.value()
    }

    fn y(&self) -> Option<Scalar> {
        self.y.value()
    }
}

impl<X: DataValue, V: DataValue> RangePoint for Estimate<X, V> {
    fn low(&self) -> Option<Scalar> {
        self.low.value()
    }

    fn high(&self) -> Option<Scalar> {
        self.high.value()
    }
}

/// A range point is an item of a sequence of range points, so every collection and state of them
/// is one, next to being a sequence of plain data points.
impl<P: RangePoint + Clone + Debug + 'static> SequenceItem<dyn RangePoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn RangePoint + 'static))) {
        f(index, self);
    }
}
