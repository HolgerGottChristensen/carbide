use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::dataset::datapoint::{DataPoint, PointValue};
use crate::dataset::datavalue::{DataValue, Value};
use crate::sequence::SequenceItem;

/// A data point with a low and a high value next to its x and y: the two ends of a floating bar,
/// or the error interval around a measured value. Marks that draw ranges (`RangeDataSetExt`)
/// exist only on datasets of such points.
///
/// It can also have a low and a high x, the error interval along x, which error bars on scatter
/// points draw (see `RangeDataSetExt::error_bars`). They are `None` by default. Each interval is
/// drawn only where both its ends are there, so a type of your own gives whichever it has: a low
/// and a high, an x low and an x high, both, or, per point, neither.
///
/// Being a `DataPoint`, it still has an x and a y, so a dataset of range points works everywhere
/// any dataset does: a bar controller without a mark draws its y as a plain bar, and a line
/// controller draws a line through its y values.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a range point",
    label = "has no low and high value",
    note = "range points are `Range {{ x, low, high }}`, `Estimate {{ x, y, low, high }}`, `Spread {{ x, y, x_low, x_high, low, high }}`, and types of your own implementing `RangePoint`",
)]
pub trait RangePoint: DataPoint {
    /// The low end of the range, or `None` when it is missing.
    fn low(&self) -> Option<Scalar>;

    /// The high end of the range, or `None` when it is missing.
    fn high(&self) -> Option<Scalar>;

    /// The low end of the interval along x, or `None`, the default, when there is none. Only error
    /// bars on scatter points draw it; bar and line charts, where x is the category, leave it out.
    fn x_low(&self) -> Option<Scalar> {
        None
    }

    /// The high end of the interval along x, or `None`, the default, when there is none (see
    /// `x_low`).
    fn x_high(&self) -> Option<Scalar> {
        None
    }
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
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.high.value()
    }

    fn describe(&self, f: &mut dyn FnMut(PointValue<'_>)) {
        if let (Some(low), Some(high)) = (self.low.value(), self.high.value()) {
            f(PointValue::Span { name: None, low, high });
        }
    }
}

impl<X: DataValue, V: DataValue> RangePoint for Range<X, V> {
    fn low(&self) -> Option<Scalar> {
        self.low.value().and_then(Value::number)
    }

    fn high(&self) -> Option<Scalar> {
        self.high.value().and_then(Value::number)
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
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.y.value()
    }

    fn describe(&self, f: &mut dyn FnMut(PointValue<'_>)) {
        if let Some(y) = self.y.value() {
            f(PointValue::Value { name: None, value: y });
        }
        if let (Some(low), Some(high)) = (self.low.value(), self.high.value()) {
            f(PointValue::Span { name: Some("range"), low, high });
        }
    }
}

impl<X: DataValue, V: DataValue> RangePoint for Estimate<X, V> {
    fn low(&self) -> Option<Scalar> {
        self.low.value().and_then(Value::number)
    }

    fn high(&self) -> Option<Scalar> {
        self.high.value().and_then(Value::number)
    }
}

/// A value `y` at `x`, with the intervals it is known within along both axes: from `x_low` to
/// `x_high` along x, and from `low` to `high` along y, such as a measurement with errors in both
/// of its coordinates. Error bars on scatter points draw both (see `RangeDataSetExt::error_bars`);
/// either interval is left out where one of its ends is missing.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spread<X = Scalar, V = Scalar> {
    pub x: X,
    pub y: V,
    pub x_low: X,
    pub x_high: X,
    pub low: V,
    pub high: V,
}

impl<X: DataValue, V: DataValue> DataPoint for Spread<X, V> {
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.y.value()
    }

    fn describe(&self, f: &mut dyn FnMut(PointValue<'_>)) {
        if let Some(y) = self.y.value() {
            f(PointValue::Value { name: None, value: y });
        }
        if let (Some(low), Some(high)) = (self.low.value(), self.high.value()) {
            f(PointValue::Span { name: Some("range"), low, high });
        }
    }
}

impl<X: DataValue, V: DataValue> RangePoint for Spread<X, V> {
    fn low(&self) -> Option<Scalar> {
        self.low.value().and_then(Value::number)
    }

    fn high(&self) -> Option<Scalar> {
        self.high.value().and_then(Value::number)
    }

    fn x_low(&self) -> Option<Scalar> {
        self.x_low.value().and_then(Value::number)
    }

    fn x_high(&self) -> Option<Scalar> {
        self.x_high.value().and_then(Value::number)
    }
}

/// A range point is an item of a sequence of range points, so every collection and state of them
/// is one, next to being a sequence of plain data points.
impl<P: RangePoint + Clone + Debug + 'static> SequenceItem<dyn RangePoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn RangePoint + 'static))) {
        f(index, self);
    }
}
