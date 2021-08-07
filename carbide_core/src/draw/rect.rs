use std::ops::{Add, Div};

use crate::draw::dimension::Dimension;
use crate::draw::Position;
use crate::draw::Scalar;

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