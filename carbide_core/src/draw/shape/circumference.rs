use crate::{Point, Scalar};
use crate::OldRect;

/// An iterator yielding the edges of an `Oval` (or some section of an `Oval`) as a circumference
/// represented as a series of edges.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Circumference {
    index: usize,
    num_points: usize,
    pub(crate) point: Point,
    rad_step: Scalar,
    rad_offset: Scalar,
    half_w: Scalar,
    half_h: Scalar,
}

impl Circumference {
    fn new_inner(rect: OldRect, num_points: usize, rad_step: Scalar) -> Self {
        let (x, y, w, h) = rect.x_y_w_h();
        Circumference {
            index: 0,
            num_points,
            point: [x, y],
            half_w: w * 0.5,
            half_h: h * 0.5,
            rad_step,
            rad_offset: 0.0,
        }
    }

    /// An iterator yielding the `Oval`'s edges as a circumference represented as a series of points.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new(rect: OldRect, mut resolution: usize) -> Self {
        resolution = std::cmp::max(resolution, 1);
        use std::f64::consts::PI;
        let radians = 2.0 * PI;
        Self::new_section(rect, resolution, radians)
    }

    /// Produces a new iterator that yields only a section of the `Oval`'s circumference, where the
    /// section is described via its angle in radians.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new_section(rect: OldRect, resolution: usize, radians: Scalar) -> Self {
        Self::new_inner(rect, resolution + 1, radians / resolution as Scalar)
    }
}

/// An iterator yielding triangles that describe an oval or some section of an oval.
#[derive(Clone)]
pub struct Triangles {
    // The last circumference point yielded by the `CircumferenceOffset` iterator.
    pub last: Point,
    // The circumference points used to yield yielded by the `CircumferenceOffset` iterator.
    pub points: Circumference,
}

impl Circumference {
    /// Produces a new iterator that yields only a section of the `Oval`'s circumference, where the
    /// section is described via its angle in radians.
    pub fn section(mut self, radians: Scalar) -> Self {
        let resolution = self.num_points - 1;
        self.rad_step = radians / resolution as Scalar;
        self
    }

    /// Rotates the position at which the iterator starts yielding points by the given radians.
    ///
    /// This is particularly useful for yielding a different section of the circumference when
    /// using `circumference_section`
    pub fn offset_radians(mut self, radians: Scalar) -> Self {
        self.rad_offset = radians;
        self
    }

    /// Produces an `Iterator` yielding `Triangle`s.
    ///
    /// Triangles are created by joining each edge yielded by the inner `Circumference` to the
    /// middle of the `Oval`.
    pub fn triangles(mut self) -> Triangles {
        let last = self.next().unwrap_or(self.point);
        Triangles { last, points: self }
    }
}

impl Iterator for Circumference {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference {
            ref mut index,
            num_points,
            point,
            rad_step,
            rad_offset,
            half_w,
            half_h,
        } = *self;
        if *index >= num_points {
            return None;
        }
        let x = point[0] + half_w * (rad_offset + rad_step * *index as Scalar).cos();
        let y = point[1] + half_h * (rad_offset + rad_step * *index as Scalar).sin();
        *index += 1;
        Some([x, y])
    }
}
