/// Marks the `IntoSequence` implementation for a single item, which becomes a sequence of one.
#[doc(hidden)]
pub struct One;

/// Marks the `IntoSequence` implementation for a value that is already a sequence.
#[doc(hidden)]
pub struct Many;
