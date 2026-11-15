use std::fmt::Debug;
use carbide::draw::Scalar;

/// One coordinate of a data point, of any type that can be placed along a scale: `value` reads it
/// as a number, which controllers and scales work in. A type of your own (a date as a timestamp,
/// say) becomes a coordinate by saying what number it stands for.
///
/// Implemented for `Scalar` (f64), `usize` and `Option<T>` of any `DataValue`, where `None` is
/// a missing value: `vec![Some(12.0), None, Some(9.0)]` is a dataset with a gap in it.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be a coordinate of a data point",
    label = "not a coordinate",
    note = "coordinates are `Scalar` (f64), `usize`, `Option` of a coordinate (where `None` is missing), and types implementing `DataValue`",
)]
pub trait DataValue: Clone + Debug + 'static {
    /// The number the value stands for, or `None` when it is missing. A missing value is not
    /// drawn, adds nothing to its dataset's bounds or stack, and leaves a gap in a line (see
    /// `SpanGaps`).
    fn value(&self) -> Option<Scalar>;
}

/// A NaN is missing, as it can't be drawn.
impl DataValue for Scalar {
    fn value(&self) -> Option<Scalar> {
        (!self.is_nan()).then_some(*self)
    }
}

impl DataValue for usize {
    fn value(&self) -> Option<Scalar> {
        Some(*self as Scalar)
    }
}

/// `None` is missing; `Some` is the value inside it.
impl<T: DataValue> DataValue for Option<T> {
    fn value(&self) -> Option<Scalar> {
        self.as_ref().and_then(DataValue::value)
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use crate::{DataSet, DataValue};

    /// A coordinate type of the test's own: a temperature in kelvin, placed along a scale in
    /// degrees Celsius. Absolute zero and below stand for a broken sensor, which is missing.
    #[derive(Clone, Debug)]
    struct Kelvin(Scalar);

    impl DataValue for Kelvin {
        fn value(&self) -> Option<Scalar> {
            (self.0 > 0.0).then_some(self.0 - 273.15)
        }
    }

    /// Any type with a number to stand for is a coordinate, in points and datasets, next to the
    /// built-in ones, and decides itself when it is missing.
    #[test]
    fn coordinate_types_of_your_own() {
        let readings = vec![(0usize, Kelvin(293.15)), (1, Kelvin(0.0)), (2, Kelvin(303.15))];

        let mut ys = vec![];
        readings.points(&mut |_, point| ys.push(point.y().map(|y| y.round())));
        assert_eq!(ys, vec![Some(20.0), None, Some(30.0)]);
        assert_eq!(readings.max().map(|(x, y)| (x, y.round())), Some((2.0, 30.0)));

        assert_eq!(Some(Kelvin(373.15)).value().map(Scalar::round), Some(100.0), "an `Option` of it is a coordinate too");
        assert_eq!(None::<Kelvin>.value(), None);
    }
}
