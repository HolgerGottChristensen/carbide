use carbide::draw::{Color, Position, Scalar};
use carbide::render::matrix::{One, Zero};
use crate::DataColor;
use crate::dataset::datavalue::DataValue;

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
        DataColor::Inherit
    }
}

impl<'a, T> DataPoint for &'a T where T: DataPoint {
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