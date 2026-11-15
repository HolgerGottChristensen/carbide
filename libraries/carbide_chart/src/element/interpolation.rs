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
    /// Horizontal and vertical steps between points.
    Stepped(Stepped),
}

impl From<Stepped> for Interpolation {
    fn from(stepped: Stepped) -> Self {
        Interpolation::Stepped(stepped)
    }
}

impl_state_value!(Interpolation);
