
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeDashPattern {
    pub pattern: Vec<f64>,
    pub offset: f64,
    pub start_cap: StrokeDashCap,
    pub end_cap: StrokeDashCap,
    pub dash_type: StrokeDashMode,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StrokeDashCap {
    None, // Also known as Butt
    Round,
    Square,
    TriangleIn,
    TriangleOut,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StrokeDashMode {
    /// Fast will only check if the triangle making up the line is in a dash
    Fast,
    /// Pretty will check if triangles that are parts of joins contains dashes.
    /// On thin lines this is not visible.
    Pretty
}