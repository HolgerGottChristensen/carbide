use carbide_core::draw::Position;
use carbide_core::Scalar;
use crate::intersect;

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Position,
    pub end: Position,
}

impl Line {
    pub fn new(start: Position, end: Position) -> Line {
        Line {
            start,
            end
        }
    }

    pub fn flip(&mut self) {
        let temp = self.start;
        self.start = self.end;
        self.end = temp;
    }

    pub fn half(&self) -> Line {
        Line {
            start: self.start,
            end: (self.end - self.start) / 2.0 + self.start,
        }
    }

    /// Return the angle in degrees
    pub fn angle(&self) -> Scalar {
        f64::atan2(self.end.y() - self.start.y(), self.end.x() - self.start.x()) * 180.0 * std::f64::consts::PI
    }

    pub fn intersect(&self, other: &Line) -> Position {
        intersect(self.start, self.end, other.start, other.end).unwrap()
    }

    pub fn direction(&self) -> Position {
        self.end - self.start
    }

    pub fn normal_offset(&self, distance: Scalar) -> Line {
        let dir = self.direction().orthogonal().normalized() * distance;
        Line {
            start: self.start + dir,
            end: self.end + dir,
        }
    }
}