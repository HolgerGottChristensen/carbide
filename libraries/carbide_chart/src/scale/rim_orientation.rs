use carbide_derive::StateValue;

/// How the labels around a polar chart's rim are turned: flat, following the circle, or pointing
/// out from its centre. A label that would read upside down is turned half a turn, so every label
/// reads from left to right or from bottom to top. Set on the angle scale (`rim_orientation`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, StateValue)]
pub enum RimOrientation {
    /// Flat, each label hanging away from the circle by the edge or corner facing it.
    #[default]
    Flat,
    /// Along the circle, centred on its angle, with the edge facing the circle at the rim.
    Along,
    /// Out from the centre, starting at the rim, or ending at it on the left half, so the text
    /// doesn't read upside down.
    Outward,
}
