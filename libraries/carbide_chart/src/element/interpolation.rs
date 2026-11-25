use carbide::draw::Scalar;
use carbide::impl_state_value;
use crate::element::Stepped;

/// How a line is drawn between a dataset's points, set with `DatasetExt::interpolation`. One
/// option holds every mode, so a line can't be stepped and smooth at once.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Interpolation {
    /// Straight lines from point to point. The default.
    #[default]
    Linear,
    /// A smooth curve through the points, a cubic spline whose `tension` sets how far it bends:
    /// 0 draws straight lines, and 0.4 gives a gentle curve. The curve can overshoot the data
    /// between points, so a series that is flat or only rising can dip or bulge; use `Monotone`
    /// where that matters.
    Smooth {
        tension: Scalar,
    },
    /// A smooth curve that never overshoots between points: where the data rises, the line
    /// rises, and peaks stay at the points. Monotone cubic interpolation (Fritsch–Carlson). It
    /// assumes the points are in x order.
    Monotone,
    /// A cardinal spline: a smooth curve through the points whose tangent at each point is
    /// parallel to the line from the previous point to the next, as for `Smooth`, but with arms
    /// of the same length on both sides. `tension` sets how tight the curve is: 0 is a uniform
    /// Catmull-Rom spline, 1 draws straight lines. Like `Smooth`, it can overshoot the data
    /// between points.
    Cardinal {
        tension: Scalar,
    },
    /// A Catmull-Rom spline: a smooth curve through the points, its tangents set by the distances
    /// between them. `alpha` sets how much the distances count: 0 is the uniform spline, the same
    /// curve as a cardinal spline with a tension of 0; 0.5, the centripetal spline, never forms a
    /// loop or a cusp within a segment, however unevenly the points are spaced; 1 is the chordal
    /// spline, which hugs the points more tightly.
    CatmullRom {
        alpha: Scalar,
    },
    /// Horizontal and vertical steps between points.
    Stepped(Stepped),
}

impl From<Stepped> for Interpolation {
    fn from(stepped: Stepped) -> Self {
        Interpolation::Stepped(stepped)
    }
}

impl_state_value!(Interpolation);
