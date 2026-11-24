use std::fmt::Debug;
use carbide::draw::Scalar;
use crate::Value;

/// One coordinate of a data point, of any type that can be placed along a scale: `value` says what
/// it stands for (see `Value`), or that it is missing. A type of your own (a date as a timestamp,
/// say) becomes a coordinate by saying what it stands for.
///
/// Implemented for `Scalar` (f64) and `usize` (numbers), `&'static str` and `String` (names of
/// categories), chrono's `DateTime` (times), and `Option<T>` of any `DataValue`, where `None` is a
/// missing value: `vec![Some(12.0), None, Some(9.0)]` is a dataset with a gap in it.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be a coordinate of a data point",
    label = "not a coordinate",
    note = "coordinates are `Scalar` (f64), `usize`, `&'static str`, `String`, chrono's `DateTime`, `Option` of a coordinate (where `None` is missing), and types implementing `DataValue`",
)]
pub trait DataValue: Clone + Debug + 'static {
    /// What the value stands for, or `None` when it is missing. A missing value is not drawn, adds
    /// nothing to its dataset's bounds or stack, and leaves a gap in a line (see `SpanGaps`).
    fn value(&self) -> Option<Value<'_>>;
}

/// A NaN is missing, as it can't be drawn.
impl DataValue for Scalar {
    fn value(&self) -> Option<Value<'_>> {
        (!self.is_nan()).then_some(Value::Number(*self))
    }
}

impl DataValue for usize {
    fn value(&self) -> Option<Value<'_>> {
        Some(Value::Number(*self as Scalar))
    }
}

/// `None` is missing; `Some` is the value inside it.
impl<T: DataValue> DataValue for Option<T> {
    fn value(&self) -> Option<Value<'_>> {
        self.as_ref().and_then(DataValue::value)
    }
}

/// A category's name is a coordinate: its place is the category's index on a category scale.
impl DataValue for &'static str {
    fn value(&self) -> Option<Value<'_>> {
        Some(Value::Text(self))
    }
}

/// A category's name is a coordinate: its place is the category's index on a category scale.
impl DataValue for String {
    fn value(&self) -> Option<Value<'_>> {
        Some(Value::Text(self))
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use crate::{DataPoint, DataSet, DataValue, Value};

    /// A coordinate type of the test's own: a temperature in kelvin, placed along a scale in
    /// degrees Celsius. Absolute zero and below stand for a broken sensor, which is missing.
    #[derive(Clone, Debug)]
    struct Kelvin(Scalar);

    impl DataValue for Kelvin {
        fn value(&self) -> Option<Value<'_>> {
            (self.0 > 0.0).then_some(Value::Number(self.0 - 273.15))
        }
    }

    /// Any type with a number to stand for is a coordinate, in points and datasets, next to the
    /// built-in ones, and decides itself when it is missing.
    #[test]
    fn coordinate_types_of_your_own() {
        let readings = vec![(0usize, Kelvin(293.15)), (1, Kelvin(0.0)), (2, Kelvin(303.15))];

        let mut ys = vec![];
        readings.points(&mut |_, point| ys.push(point.y().and_then(Value::number).map(Scalar::round)));
        assert_eq!(ys, vec![Some(20.0), None, Some(30.0)]);

        assert_eq!(Some(Kelvin(373.15)).value().and_then(Value::number).map(Scalar::round), Some(100.0), "an `Option` of it is a coordinate too");
        assert_eq!(None::<Kelvin>.value(), None);
    }

    /// A string is a coordinate that names a category: its value is text, with no number, and a
    /// point with a string x isn't missing, as the chart places it by its category.
    #[test]
    fn strings_name_categories() {
        assert_eq!(("Red", 12.0).x(), Some(Value::Text("Red")));
        assert_eq!(Value::Text("Red").number(), None);
        assert!(!("Red", 12.0).is_missing());
        assert!((None::<&str>, 12.0).is_missing(), "a missing name is a missing x");
        assert_eq!((Some("Red".to_string()), 12.0).x(), Some(Value::Text("Red")));
    }
}
