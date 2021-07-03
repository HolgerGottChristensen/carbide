use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use rusttype::Point;

use crate::draw::Dimension;
use crate::Scalar;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct Position {
    pub(crate) x: Scalar,
    pub(crate) y: Scalar,
}

impl Position {
    pub fn new(x: Scalar, y: Scalar) -> Position {
        Position {
            x,
            y,
        }
    }
}

impl AddAssign<Dimension> for Position {
    fn add_assign(&mut self, rhs: Dimension) {
        *self = *self + rhs;
    }
}

impl Mul<Scalar> for Position {
    type Output = Position;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.x *= rhs;
        self.y *= rhs;
        self
    }
}

impl Div<Scalar> for Position {
    type Output = Position;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.x /= rhs;
        self.y /= rhs;
        self
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(mut self, rhs: Position) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Add<Dimension> for Position {
    type Output = Position;

    fn add(mut self, rhs: Dimension) -> Self::Output {
        self.x += rhs.width;
        self.y += rhs.height;
        self
    }
}

impl From<Point<f32>> for Position {
    fn from(pos: Point<f32>) -> Self {
        Position {
            x: pos.x as f64,
            y: pos.y as f64,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}