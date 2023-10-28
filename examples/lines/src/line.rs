use crate::intersect;
use carbide::draw::{Position, Rect, Scalar};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub start: Position,
    pub end: Position,
}

impl Line {
    pub fn new(start: Position, end: Position) -> Line {
        Line { start, end }
    }

    pub fn flip(&mut self) {
        let temp = self.start;
        self.start = self.end;
        self.end = temp;
    }

    pub fn flipped(&self) -> Line {
        Line::new(self.end, self.start)
    }

    pub fn half(&self) -> Line {
        Line {
            start: self.start,
            end: (self.end - self.start) / 2.0 + self.start,
        }
    }

    /// Return the angle in degrees
    pub fn angle(&self) -> Scalar {
        f64::to_degrees(f64::atan2(
            self.end.y() - self.start.y(),
            self.end.x() - self.start.x(),
        ))
    }

    pub fn intersect(&self, other: &Line) -> Option<Position> {
        intersect(self.start, self.end, other.start, other.end)
    }

    pub fn len(&self) -> Scalar {
        self.end.dist(&self.start)
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

    pub fn dist_inf_line_to_point(&self, point: Position) -> Scalar {
        let dx = self.start.x() - self.end.x();
        let dy = self.start.y() - self.end.y();

        let length = (dx * dx + dy * dy).sqrt();

        let dx = dx / length;
        let dy = dy / length;

        (dy * (point.x() - self.start.x()) - dx * (point.y() - self.start.y())).abs()
    }

    pub fn closest_point_on_line_infinite(&self, point: Position) -> Position {
        let ap = point - self.start;
        let ab = self.end - self.start;

        let t = (ap.dot(&ab) / ab.dot(&ab));

        let res = self.start + (ab * t);

        res
    }

    pub fn closest_point_on_line(&self, point: Position) -> Option<Position> {
        let ap = point - self.start;
        let ab = self.end - self.start;

        let t = (ap.dot(&ab) / ab.dot(&ab));

        if t < 0.0 || t > 1.0 {
            None
        } else {
            Some(self.start + (ab * t))
        }
    }

    pub fn extend(&self, bounding_box: Rect) -> Line {
        if self.start.y() == self.end.y() {
            return Line::new(
                Position::new(bounding_box.bottom(), self.start.y()),
                Position::new(bounding_box.top(), self.start.y()),
            );
        }

        if self.start.x() == self.end.x() {
            return Line::new(
                Position::new(self.start.x(), bounding_box.left()),
                Position::new(self.start.x(), bounding_box.right()),
            );
        }

        let bottom_line = Line::new(
            Position::new(bounding_box.left(), bounding_box.bottom()),
            Position::new(bounding_box.right(), bounding_box.bottom()),
        );

        let top_line = Line::new(
            Position::new(bounding_box.left(), bounding_box.top()),
            Position::new(bounding_box.right(), bounding_box.top()),
        );

        let p1 = self.intersect(&bottom_line).unwrap();
        let p2 = self.intersect(&top_line).unwrap();

        let res1 = Line::new(p1, p2);

        let left_line = Line::new(
            Position::new(bounding_box.left(), bounding_box.bottom()),
            Position::new(bounding_box.left(), bounding_box.top()),
        );

        let right_line = Line::new(
            Position::new(bounding_box.right(), bounding_box.bottom()),
            Position::new(bounding_box.right(), bounding_box.top()),
        );

        let p3 = self.intersect(&left_line).unwrap();
        let p4 = self.intersect(&right_line).unwrap();

        let res2 = Line::new(p3, p4);

        if res1.len() <= res2.len() {
            res1
        } else {
            res2
        }
    }
}
