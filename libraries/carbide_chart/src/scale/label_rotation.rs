use carbide::draw::Angle;
use carbide::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};
use carbide_derive::StateValue;

/// How far a scale's tick labels are turned: by a fixed angle, or only when they don't fit.
///
/// Angles turn the way the canvas does: a negative angle turns a label counter-clockwise on
/// screen, so it rises to the right, and `Fixed(Angle::Degrees(-45.0))` is the usual slanted
/// label. A label turns around the point of it that sits at the end of its tick mark (see
/// `LinearScale::label_alignment`).
///
/// An `Angle` or a number of degrees converts to a fixed rotation, so `.label_rotation(-45.0)`
/// turns every label by 45°.
#[derive(Copy, Clone, Debug, PartialEq, StateValue)]
pub enum LabelRotation {
    /// Every label turned by the angle, on any axis.
    Fixed(Angle),
    /// Flat when the labels fit along the axis, and turned by `max` when they don't. Only
    /// horizontal axes turn; on a vertical axis the labels stay flat, as turning them doesn't
    /// make room between them.
    Auto {
        max: Angle,
    },
}

impl LabelRotation {
    /// Flat, until the labels don't fit; then turned by 45°, rising to the right.
    pub const AUTO: LabelRotation = LabelRotation::Auto { max: Angle::Degrees(-45.0) };

    /// Flat labels.
    pub const FLAT: LabelRotation = LabelRotation::Fixed(Angle::Degrees(0.0));
}

impl Default for LabelRotation {
    fn default() -> Self {
        LabelRotation::AUTO
    }
}

impl ConvertIntoRead<LabelRotation> for Angle {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&Angle) -> LabelRotation, Angle, LabelRotation, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |angle| LabelRotation::Fixed(*angle))
    }
}

impl ConvertIntoRead<LabelRotation> for f64 {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&f64) -> LabelRotation, f64, LabelRotation, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |degrees| LabelRotation::Fixed(Angle::Degrees(*degrees)))
    }
}
