use crate::MarkRole;

/// A mark's `MarkRole` as a type, for `Mark::Role`: `Primary` or `Overlay`.
pub trait Role: sealed::Sealed + 'static {
    /// The role as a value, for the controllers reading marks through trait objects.
    const ROLE: MarkRole;
}

/// Keeps `Role` to the crate's own roles: only `Primary` and `Overlay` implement `Sealed`.
pub(crate) mod sealed {
    pub trait Sealed {}
}
