


/// The fill rule defines how to determine what is inside and what is outside of the shape.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FillRule {
    EvenOdd,
    NonZero,
}