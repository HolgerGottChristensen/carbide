use crate::render::primitive::Primitive;

/// A trait that allows the user to remain generic over render yielding `Primitive`s.
///
/// This trait is implemented for both the `Primitives` and `WalkOwnedPrimitives` render.
pub trait PrimitiveWalker {
    /// Yield the next `Primitive` in order of depth, bottom to top.
    fn next_primitive(&mut self) -> Option<Primitive>;
}

