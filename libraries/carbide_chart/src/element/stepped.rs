use carbide_derive::StateValue;

/// Where a stepped line turns between two points (see `Interpolation::Stepped`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, StateValue)]
pub enum Stepped {
    /// Move Y then X
    Before,
    /// Move X then Y
    After,
    /// Move X half way then Y
    Middle,
    /// Move Y half way then X
    MiddleVertical,
}
