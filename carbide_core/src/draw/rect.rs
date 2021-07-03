use std::ops::{Add, Div};

use crate::{Point, Scalar};
use crate::draw::dimension::Dimension;
use crate::draw::Position;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct Rect {
    pub position: Position,
    pub dimension: Dimension,
}

impl Rect {
    pub fn l_r_b_t(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        (
            self.position.x,
            self.position.x + self.dimension.width,
            self.position.y,
            self.position.y + self.dimension.height,
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