use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use carbide_rusttype::Point;

use crate::draw::Dimension;
use crate::draw::Scalar;

/// # Position
///
/// The position is used to represent a vector in 2d space.
/// All widgets has a position, and is usually relative to the top-left corner of a window.
///
/// The position can be seen as a vector and can be translated, reversed, normalized and more.
/// You are also able to add Positions and [super::Dimension], to get new offset positions.
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct Position {
    pub(crate) x: Scalar,
    pub(crate) y: Scalar,
}

impl Position {
    /// Create a new Position from scalar values.
    pub const fn new(x: Scalar, y: Scalar) -> Position {
        Position { x, y }
    }

    pub fn origin() -> Position {
        Position { x: 0.0, y: 0.0 }
    }

    /// Get the x component of the position. This is the horizontal component.
    pub const fn x(&self) -> Scalar {
        self.x
    }

    /// Get the y component of the position. This is the vertical component.
    pub const fn y(&self) -> Scalar {
        self.y
    }

    /// Get a new position that takes the min x position from the two positions and uses it as
    /// its x, and the minimum y position of the two positions and uses it as its y.
    pub fn min(&self, other: &Position) -> Position {
        Position::new(
            self.x.min(other.x),
            self.y.min(other.y),
        )
    }

    /// Get a new position that takes the maximum x position from the two positions and uses it as
    /// its x, and the maximum y position of the two positions and uses it as its y.
    pub fn max(&self, other: &Position) -> Position {
        Position::new(
            self.x.max(other.x),
            self.y.max(other.y),
        )
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
            x = 1.0 + x;
        }

        if y < 0.0 {
            y = 1.0 + y;
        }

        Position::new(x, y)
    }

    /// Translate the position. Positive is rightwards and negative is leftwards.
    pub fn translate_x(&self, x: Scalar) -> Position {
        Position::new(self.x + x, self.y)
    }

    /// Translate the position. Positive is downwards and negative is upwards.
    pub fn translate_y(&self, y: Scalar) -> Position {
        Position::new(self.x, self.y + y)
    }

    pub fn normalized_offset(&self) -> Position {
        let mut x = self.x;
        let mut y = self.y;
        if x > 0.5 {
            x -= 1.0;
        } else if x < -0.5 {
            x += 1.0;
        }
        if y > 0.5 {
            y -= 1.0;
        } else if y < -0.5 {
            y += 1.0;
        }
        Position::new(x, y)
    }

    pub fn round_to_u16(&self) -> (u16, u16) {
        let x = (self.x + 0.5) as u16;
        let y = (self.y + 0.5) as u16;
        (x, y)
    }

    /// Returns a new position where x and y is rounded to the nearest whole number.
    #[inline]
    pub fn rounded(&self) -> Position {
        let x = self.x.round();
        let y = self.y.round();
        Position::new(x, y)
    }

    /// Returns a new position where x and y is truncated, always rounded towards 0
    #[inline]
    pub fn truncated(&self) -> Position {
        let x = self.x.trunc();
        let y = self.y.trunc();
        Position::new(x, y)
    }

    /// Returns the fractional parts of the x and y components using [f64::fract()]
    #[inline]
    pub fn fraction(&self) -> Position {
        let x = self.x.fract();
        let y = self.y.fract();
        Position::new(x, y)
    }

    /// Returns a boolean indicating whether the position is realistically 0
    #[inline]
    pub fn is_near_zero(&self) -> bool {
        let x = self.x.abs() <= Scalar::EPSILON;
        let y = self.y.abs() <= Scalar::EPSILON;
        x && y
    }

    /// Returns a new position orthogonal to the original.
    /// If the position is seen as a vector, the vector is rotated 90 deg anti-clockwise
    pub fn orthogonal(&self) -> Position {
        Position::new(self.y, -self.x)
    }

    /// Returns a new position in the opposite direction of the original.
    pub fn reverse(&self) -> Position {
        Position::new(-self.x, -self.y)
    }

    /// Get the length of a given position vector
    pub fn len(&self) -> Scalar {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// Returns a new position in the same direction of the given, with a length of 1
    pub fn normalized(&self) -> Position {
        let len = self.len();
        Position::new(self.x / len, self.y / len)
    }

    /// Get the absolute distance between two points.
    pub fn dist(&self, other: &Position) -> Scalar {
        (*self - *other).len()
    }

    /// Get the dot product between two positions.
    pub fn dot(&self, other: &Position) -> Scalar {
        self.x * other.x + self.y * other.y
    }

    pub fn tolerance(&self, tolerance: Scalar) -> Position {
        let mut position = *self / tolerance;
        position = position.rounded();
        position = position * tolerance;
        position
    }
}

impl AddAssign<Dimension> for Position {
    fn add_assign(&mut self, rhs: Dimension) {
        *self = *self + rhs;
    }
}

impl AddAssign<Position> for Position {
    fn add_assign(&mut self, rhs: Position) {
        *self = *self + rhs;
    }
}

impl SubAssign<Position> for Position {
    fn sub_assign(&mut self, rhs: Position) {
        *self = *self - rhs;
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
            x: pos.x as Scalar,
            y: pos.y as Scalar,
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
    assert_eq!(position.fraction_0_1(), expected, "Fraction 1");

    // 1.0 will return 0.0
    let position = Position::new(1.0, 1.0);
    let expected = Position::new(0.0, 0.0);
    assert_eq!(position.fraction_0_1(), expected, "Fraction 2");

    // 1.5 will return 0.5
    let position = Position::new(1.5, 1.5);
    let expected = Position::new(0.5, 0.5);
    assert_eq!(position.fraction_0_1(), expected, "Fraction 3");

    // -1.5 will return 0.5
    let position = Position::new(-1.5, -1.5);
    let expected = Position::new(0.5, 0.5);
    assert_eq!(position.fraction_0_1(), expected, "Fraction 4");

    // -0.2 will return 0.8
    let position = Position::new(-0.2, -0.2);
    let expected = Position::new(0.8, 0.8);
    assert_eq!(position.fraction_0_1(), expected, "Fraction 5");

    // 0.2 will return 0.2
    let position = Position::new(0.2, 0.2);
    let expected = Position::new(0.2, 0.2);
    assert_eq!(position.fraction_0_1(), expected, "Fraction 6");
}
