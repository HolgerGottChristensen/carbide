/// Describes the "Handedness" of a given coordinate system. Affects math done
/// in the space.
///
/// While a weird term, if you make your thumb X, your pointer Y,
/// and your middle finger Z, the handedness can be determined by which hand can
/// contort to represent the coordinate system.
///
/// For example
/// +X right, +Y up, +Z _into_ the screen is left handed.
/// +X right, +Y up, +Z _out of_ the screen is right handed.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Handedness {
    Left,
    Right,
}

/*impl From<Handedness> for FrontFace {
    fn from(value: Handedness) -> Self {
        match value {
            Handedness::Left => Self::Cw,
            Handedness::Right => Self::Ccw,
        }
    }
}*/

impl Default for Handedness {
    fn default() -> Self {
        Self::Left
    }
}