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

    /// Returns the fraction of the position between 0.0 and 1.0 (exclusive)
    /// 0.0 will return 0.0
    /// 1.0 will return 0.0
    /// 1.5 will return 0.5
    /// -1.5 will return 0.5
    /// -0.2 will return 0.8
    /// 0.2 will return 0.2
    pub fn fraction_0_1(&self) -> Position {
        // Returns a number between -1.0 and 1.0 (both exclusive)
        let mut x = self.x.fract();
        let mut y = self.y.fract();

        if x < 0.0 {
            x = 1.0 - x;
        }

        if y < 0.0 {
            y = 1.0 - y;
        }

        Position::new(x, y)
    }

    pub fn round_to_u16(&self) -> (u16, u16) {
        let x = (self.x + 0.5) as u16;
        let y = (self.y + 0.5) as u16;
        (x, y)
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

#[test]
fn fraction_0_1() {
    // 0.0 will return 0.0
    let position = Position::new(0.0, 0.0);
    let expected = Position::new(0.0, 0.0);
    assert_eq!(position.fraction_0_1(), expected);

    // 1.0 will return 0.0
    let position = Position::new(1.0, 1.0);
    let expected = Position::new(0.0, 0.0);
    assert_eq!(position.fraction_0_1(), expected);

    // 1.5 will return 0.5
    let position = Position::new(1.5, 1.5);
    let expected = Position::new(0.5, 0.5);
    assert_eq!(position.fraction_0_1(), expected);

    // -1.5 will return 0.5
    let position = Position::new(-1.5, -1.5);
    let expected = Position::new(0.5, 0.5);
    assert_eq!(position.fraction_0_1(), expected);

    // -0.2 will return 0.8
    let position = Position::new(-0.2, -0.2);
    let expected = Position::new(0.8, 0.8);
    assert_eq!(position.fraction_0_1(), expected);

    // 0.2 will return 0.2
    let position = Position::new(0.2, 0.2);
    let expected = Position::new(0.2, 0.2);
    assert_eq!(position.fraction_0_1(), expected);
}