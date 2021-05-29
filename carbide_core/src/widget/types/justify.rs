/// A type used for referring to typographic alignment of `Text`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Justify {
    /// Align text to the start of the bounding `Rect`'s *x* axis.
    Left,
    /// Symmetrically align text along the *y* axis.
    Center,
    /// Align text to the end of the bounding `Rect`'s *x* axis.
    Right,
    // /// Align wrapped text to both the start and end of the bounding `Rect`s *x* axis.
    // ///
    // /// Extra space is added between words in order to achieve this alignment.
    // TODO: Fill,
}
