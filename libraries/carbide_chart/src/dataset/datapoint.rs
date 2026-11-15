use carbide::draw::{Position, Scalar};
use carbide::math::Zero;
use crate::DataColor;
use crate::dataset::datavalue::DataValue;

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a data point",
    label = "not a data point",
    note = "data points are `Position`, `(x, y)` and `(x, y, z)` tuples of coordinates (`Scalar` or `usize`), and references to them",
    note = "a plain `Scalar` is not a point on its own, but is the y value at its index inside a sequence",
)]
pub trait DataPoint {
    type X: DataValue;
    type Y: DataValue;
    type Z: DataValue;

    fn x(&self) -> Self::X;
    fn y(&self) -> Self::Y;
    fn z(&self) -> Self::Z {
        Self::Z::zero()
    }

    fn color(&self) -> DataColor {
        DataColor::ByDataSet
    }
}

impl<T> DataPoint for &T where T: DataPoint {
    type X = T::X;
    type Y = T::Y;
    type Z = T::Z;

    fn x(&self) -> Self::X {
        T::x(self)
    }

    fn y(&self) -> Self::Y {
        T::y(self)
    }

    fn z(&self) -> Self::Z {
        T::z(self)
    }

    fn color(&self) -> DataColor {
        T::color(self)
    }
}

impl DataPoint for Position {
    type X = Scalar;
    type Y = Scalar;
    type Z = Scalar;

    fn x(&self) -> Scalar {
        self.x
    }

    fn y(&self) -> Scalar {
        self.y
    }
}

impl<X: DataValue, Y: DataValue> DataPoint for (X, Y) {
    type X = X;
    type Y = Y;
    type Z = X;

    fn x(&self) -> X {
        self.0
    }

    fn y(&self) -> Y {
        self.1
    }
}

impl<X: DataValue, Y: DataValue, Z: DataValue> DataPoint for (X, Y, Z) {
    type X = X;
    type Y = Y;
    type Z = Z;

    fn x(&self) -> X {
        self.0
    }

    fn y(&self) -> Y {
        self.1
    }

    fn z(&self) -> Z {
        self.2
    }
}

#[cfg(test)]
mod tests {
    use carbide::color::RED;
    use carbide::draw::Scalar;
    use crate::{DataColor, DataPoint};

    /// A point with a colour of its own, as a user's data type would be.
    struct Reading(Scalar);

    impl DataPoint for Reading {
        type X = Scalar;
        type Y = Scalar;
        type Z = Scalar;

        fn x(&self) -> Scalar {
            0.0
        }

        fn y(&self) -> Scalar {
            self.0
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
        assert_eq!(DataPoint::y(&by_reference), 4.0);
    }
}
