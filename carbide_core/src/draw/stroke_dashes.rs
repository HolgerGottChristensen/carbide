
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeDashPattern {
    pub pattern: Vec<f64>,
    pub offset: f64,
    pub start_cap: StrokeDashCap,
    pub end_cap: StrokeDashCap,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StrokeDashCap {
    None,
    Square,
    Round,
    TriangleIn,
    TriangleOut,
}