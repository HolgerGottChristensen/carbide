use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use crate::draw::Position;
use crate::draw::Scalar;

#[derive(Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Dimension {
    pub width: Scalar,
    pub height: Scalar,
    // https://users.rust-lang.org/t/public-getter-method-vs-pub-field/20147/4
    _private: ()
}

impl Dimension {
    #[inline]
    pub fn new(width: Scalar, height: Scalar) -> Dimension {
        Dimension { width, height, _private: () }
    }

}

impl AddAssign<Position> for Dimension {
    fn add_assign(&mut self, rhs: Position) {
        *self = *self + rhs;
    }
}

impl Add<Position> for Dimension {
    type Output = Dimension;

    fn add(mut self, rhs: Position) -> Self::Output {
        self.width += rhs.x;
        self.height += rhs.y;
        self
    }
}

impl AddAssign for Dimension {
    fn add_assign(&mut self, rhs: Dimension) {
        *self = *self + rhs;
    }
}

impl Add for Dimension {
    type Output = Dimension;

    fn add(mut self, rhs: Dimension) -> Self::Output {
        self.width += rhs.width;
        self.height += rhs.height;
        self
    }
}

impl SubAssign<Position> for Dimension {
    fn sub_assign(&mut self, rhs: Position) {
        *self = *self - rhs;
    }
}

impl Sub<Position> for Dimension {
    type Output = Dimension;

    fn sub(mut self, rhs: Position) -> Self::Output {
        self.width -= rhs.x;
        self.height -= rhs.y;
        self
    }
}

impl SubAssign for Dimension {
    fn sub_assign(&mut self, rhs: Dimension) {
        *self = *self - rhs;
    }
}

impl Sub for Dimension {
    type Output = Dimension;

    fn sub(mut self, rhs: Dimension) -> Self::Output {
        self.width -= rhs.width;
        self.height -= rhs.height;
        self
    }
}

impl Div<Scalar> for Dimension {
    type Output = Dimension;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.width /= rhs;
        self.height /= rhs;
        self
    }
}

impl Mul<Scalar> for Dimension {
    type Output = Dimension;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.width *= rhs;
        self.height *= rhs;
        self
    }
}

impl Display for Dimension {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(w: {}px, h: {}px)", self.width, self.height)
    }
}

impl Debug for Dimension {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dimension")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}