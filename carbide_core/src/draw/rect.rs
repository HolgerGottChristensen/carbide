use std::ops::{Add, Div, Mul};

use crate::draw::dimension::Dimension;
use crate::draw::Position;
use crate::draw::Scalar;

pub type BoundingBox = Rect;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct Rect {
    pub position: Position,
    pub dimension: Dimension,
}

impl Rect {
    pub fn new(position: Position, dimension: Dimension) -> Rect {
        Rect {
            position,
            dimension,
        }
    }

    pub fn width(&self) -> Scalar {
        self.dimension.width
    }

    pub fn height(&self) -> Scalar {
        self.dimension.height
    }

    pub fn within_bounding_box(&self, bounding_box: &Rect) -> Rect {
        let bottom_left_1 = self.bottom_left();
        let bottom_left_2 = bounding_box.bottom_left();

        let top_right_1 = self.top_right();
        let top_right_2 = bounding_box.top_right();

        let bottom_left = bottom_left_1.max(&bottom_left_2);
        let top_right = top_right_1.min(&top_right_2);

        Rect::from_corners(bottom_left, top_right)
    }

    pub fn from_corners(corner1: Position, corner2: Position) -> Rect {
        let min_x = corner1.x.min(corner2.x);
        let min_y = corner1.y.min(corner2.y);
        let max_x = corner1.x.max(corner2.x);
        let max_y = corner1.y.max(corner2.y);

        let position = Position::new(min_x, min_y);
        let dimension = Dimension::new(max_x - min_x, max_y - min_y);

        Rect::new(position, dimension)
    }

    pub fn left(&self) -> Scalar {
        self.position.x
    }
    pub fn right(&self) -> Scalar {
        self.position.x + self.dimension.width
    }
    pub fn bottom(&self) -> Scalar {
        self.position.y
    }
    pub fn top(&self) -> Scalar {
        self.position.y + self.dimension.height
    }

    pub fn bottom_left(&self) -> Position {
        Position::new(self.left(), self.bottom())
    }

    pub fn bottom_right(&self) -> Position {
        Position::new(self.right(), self.bottom())
    }

    pub fn top_left(&self) -> Position {
        Position::new(self.left(), self.top())
    }

    pub fn top_right(&self) -> Position {
        Position::new(self.right(), self.top())
    }

    pub fn round(&mut self) {
        self.position.x = self.position.x.round();
        self.position.y = self.position.y.round();
        self.dimension.width = self.dimension.width.round();
        self.dimension.height = self.dimension.height.round();
    }

    pub fn l_r_b_t_with_precision(&self, precision: Scalar) -> (Scalar, Scalar, Scalar, Scalar) {
        (
            (self.position.x / precision).round() * precision,
            ((self.position.x + self.dimension.width) / precision).round() * precision,
            (self.position.y / precision).round() * precision,
            ((self.position.y + self.dimension.height) / precision).round() * precision,
        )
    }

    pub fn l_r_b_t(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        (
            self.position.x,
            self.position.x + self.dimension.width,
            self.position.y,
            self.position.y + self.dimension.height,
        )
    }

    pub fn l_r_b_t_scaled(&self, scale_factor: Scalar) -> (Scalar, Scalar, Scalar, Scalar) {
        (
            self.position.x / scale_factor,
            (self.position.x + self.dimension.width) / scale_factor,
            self.position.y / scale_factor,
            (self.position.y + self.dimension.height) / scale_factor,
        )
    }
}

impl From<carbide_rusttype::Rect<f64>> for Rect {
    fn from(rect: carbide_rusttype::Rect<f64>) -> Self {
        let width = rect.max.x - rect.min.x;
        let height = rect.max.y - rect.min.y;
        Rect {
            position: Position::new(rect.min.x, rect.min.y),
            dimension: Dimension::new(width, height),
        }
    }
}

impl From<carbide_rusttype::Rect<i32>> for Rect {
    fn from(rect: carbide_rusttype::Rect<i32>) -> Self {
        let width = rect.max.x - rect.min.x;
        let height = rect.max.y - rect.min.y;
        Rect {
            position: Position::new(rect.min.x as f64, rect.min.y as f64),
            dimension: Dimension::new(width as f64, height as f64),
        }
    }
}

impl Add<Position> for Rect {
    type Output = Rect;

    fn add(mut self, rhs: Position) -> Self::Output {
        self.position = self.position + rhs;
        self
    }
}

impl Div<Scalar> for Rect {
    type Output = Rect;

    fn div(mut self, rhs: f64) -> Self::Output {
        self.position = self.position / rhs;
        self.dimension = self.dimension / rhs;
        self
    }
}

impl Mul<Scalar> for Rect {
    type Output = Rect;

    fn mul(mut self, rhs: f64) -> Self::Output {
        self.position = self.position * rhs;
        self.dimension = self.dimension * rhs;
        self
    }
}
