

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StrokeDashCap {
    None, // Also known as Butt
    Round,
    Square,
    TriangleIn,
    TriangleOut,
}