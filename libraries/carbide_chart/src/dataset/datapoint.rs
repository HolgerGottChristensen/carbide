use carbide::draw::Position;
use crate::DataColor;
use crate::dataset::datavalue::{DataValue, Value};

/// One datum, as a controller reads it to draw a bar or a point: what its coordinates stand for
/// (`Value`: a number, a time or a category's name), each `None` when it is missing, and
/// optionally a colour of its own. Controllers place the coordinates on their scales through
/// `Coordinates`.
///
/// Implemented for `Position`, for `(x, y)` and `(x, y, z)` tuples of any coordinate types
/// (`DataValue`, so `(Scalar, Option<Scalar>)` is a point whose y may be missing), for `Option`
/// of a point (`None` is a missing point), and for references to points. A type of your own
/// becomes a point by saying what its coordinates are.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a data point",
    label = "not a data point",
    note = "data points are `Position`, `(x, y)` and `(x, y, z)` tuples of coordinates (`Scalar`, `usize`, `Option` of a coordinate, or any `DataValue`), `Option` of a point, and references to them",
    note = "a plain `Scalar` or `Option<Scalar>` is not a point on its own, but is the y value at its index inside a sequence",
)]
pub trait DataPoint {
    /// What the point's x stands for (see `Value`), or `None` when it is missing.
    fn x(&self) -> Option<Value<'_>>;

    /// What the point's y stands for (see `Value`), or `None` when it is missing.
    fn y(&self) -> Option<Value<'_>>;

    /// What the point's z stands for (see `Value`), for charts with a third coordinate, such as a
    /// bubble's size, or `None`, the default, when it has none or it is missing.
    fn z(&self) -> Option<Value<'_>> {
        None
    }

    /// The point's own colour, which overrides its dataset's (see `DataColor`).
    fn color(&self) -> DataColor {
        DataColor::ByDataSet
    }

    /// Whether the point is missing: its x or y is. A missing point is not drawn, adds nothing to
    /// its dataset's bounds or stack, and leaves a gap in a line (see `SpanGaps`).
    fn is_missing(&self) -> bool {
        self.x().is_none() || self.y().is_none()
    }
}

impl<T: DataPoint + ?Sized> DataPoint for &T {
    fn x(&self) -> Option<Value<'_>> {
        T::x(self)
    }

    fn y(&self) -> Option<Value<'_>> {
        T::y(self)
    }

    fn z(&self) -> Option<Value<'_>> {
        T::z(self)
    }

    fn color(&self) -> DataColor {
        T::color(self)
    }
}

impl DataPoint for Position {
    fn x(&self) -> Option<Value<'_>> {
        self.x.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.y.value()
    }
}

impl<X: DataValue, Y: DataValue> DataPoint for (X, Y) {
    fn x(&self) -> Option<Value<'_>> {
        self.0.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.1.value()
    }
}

impl<X: DataValue, Y: DataValue, Z: DataValue> DataPoint for (X, Y, Z) {
    fn x(&self) -> Option<Value<'_>> {
        self.0.value()
    }

    fn y(&self) -> Option<Value<'_>> {
        self.1.value()
    }

    fn z(&self) -> Option<Value<'_>> {
        self.2.value()
    }
}

/// `None` is a missing point, with no coordinates; `Some` is the point inside it.
impl<P: DataPoint> DataPoint for Option<P> {
    fn x(&self) -> Option<Value<'_>> {
        self.as_ref().and_then(DataPoint::x)
    }

    fn y(&self) -> Option<Value<'_>> {
        self.as_ref().and_then(DataPoint::y)
    }

    fn z(&self) -> Option<Value<'_>> {
        self.as_ref().and_then(DataPoint::z)
    }

    fn color(&self) -> DataColor {
        self.as_ref().map_or(DataColor::ByDataSet, DataPoint::color)
    }
}

#[cfg(test)]
mod tests {
    use carbide::color::RED;
    use carbide::draw::{Position, Scalar};
    use crate::{DataColor, DataPoint, Value};

    /// A point with a colour of its own, as a user's data type would be.
    struct Reading(Scalar);

    impl DataPoint for Reading {
        fn x(&self) -> Option<Value<'_>> {
            Some(Value::Number(0.0))
        }

        fn y(&self) -> Option<Value<'_>> {
            Some(Value::Number(self.0))
        }

        fn color(&self) -> DataColor {
            DataColor::Color(RED)
        }
    }

    /// A point held by reference is the same point: its colour comes through too, not just its
    /// coordinates.
    #[test]
    fn points_by_reference_keep_their_colour() {
        let reading = Reading(4.0);
        let by_reference = &reading;

        assert!(matches!(DataPoint::color(&by_reference), DataColor::Color(color) if color == RED));
        assert_eq!(DataPoint::y(&by_reference), Some(Value::Number(4.0)));
    }

    /// A coordinate is missing when it is `None`, or a NaN, which can't be drawn; a point is
    /// missing when its x or y is. Coordinates of different types mix in one point.
    #[test]
    fn missing_coordinates_make_missing_points() {
        assert_eq!((2.0, Some(3.0)).y(), Some(Value::Number(3.0)));
        assert!((2.0, None::<Scalar>).is_missing());
        assert!((None::<Scalar>, 3.0).is_missing());
        assert!((Scalar::NAN, 3.0).is_missing(), "a NaN is missing too");
        assert!(!(4usize, 3.0).is_missing());
        assert_eq!((4usize, 3.0).x(), Some(Value::Number(4.0)));

        assert!(None::<Position>.is_missing(), "`None` is a missing point");
        assert_eq!(Some(Position::new(1.0, 2.0)).y(), Some(Value::Number(2.0)));
        assert_eq!((1.0, 2.0, Some(5.0)).z(), Some(Value::Number(5.0)));
        assert_eq!((1.0, 2.0).z(), None);
    }
}
