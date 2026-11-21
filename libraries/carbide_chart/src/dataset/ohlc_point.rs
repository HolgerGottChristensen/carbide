use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::dataset::datapoint::DataPoint;
use crate::dataset::datavalue::{DataValue, Value};
use crate::sequence::SequenceItem;

/// A data point with an open, a high, a low and a close value, such as a trading period's prices.
/// Candlesticks and OHLC bars (`OhlcDataSetExt`) exist only on datasets of such points.
///
/// Being a `DataPoint`, it still has an x and a y, so a dataset of them works everywhere any
/// dataset does: a line controller draws a line through the closes.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an OHLC point",
    label = "has no open, high, low and close",
    note = "OHLC points are `Ohlc {{ x, open, high, low, close }}`, and types of your own implementing `OhlcPoint`",
)]
pub trait OhlcPoint: DataPoint {
    /// The first value of the period, or `None` when it is missing.
    fn open(&self) -> Option<Scalar>;

    /// The highest value of the period, or `None` when it is missing.
    fn high(&self) -> Option<Scalar>;

    /// The lowest value of the period, or `None` when it is missing.
    fn low(&self) -> Option<Scalar>;

    /// The last value of the period, or `None` when it is missing.
    fn close(&self) -> Option<Scalar>;
}

/// The open, high, low and close of a period at `x`. Its y is the close.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ohlc<X = Scalar, V = Scalar> {
    pub x: X,
    pub open: V,
    pub high: V,
    pub low: V,
    pub close: V,
}

impl<X: DataValue, V: DataValue> DataPoint for Ohlc<X, V> {
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.close.value()
    }
}

impl<X: DataValue, V: DataValue> OhlcPoint for Ohlc<X, V> {
    fn open(&self) -> Option<Scalar> {
        self.open.value().and_then(Value::number)
    }

    fn high(&self) -> Option<Scalar> {
        self.high.value().and_then(Value::number)
    }

    fn low(&self) -> Option<Scalar> {
        self.low.value().and_then(Value::number)
    }

    fn close(&self) -> Option<Scalar> {
        self.close.value().and_then(Value::number)
    }
}

/// An OHLC point is an item of a sequence of them, so every collection and state of them is one,
/// next to being a sequence of plain data points.
impl<P: OhlcPoint + Clone + Debug + 'static> SequenceItem<dyn OhlcPoint> for P {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn OhlcPoint + 'static))) {
        f(index, self);
    }
}
