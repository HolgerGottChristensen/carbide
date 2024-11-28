use carbide::impl_state_value;
use carbide_derive::StateValue;

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
    /// Move X and Y
    None,
}