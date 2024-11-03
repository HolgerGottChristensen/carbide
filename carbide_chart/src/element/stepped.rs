use carbide::impl_state_value;

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl_state_value!(Stepped);