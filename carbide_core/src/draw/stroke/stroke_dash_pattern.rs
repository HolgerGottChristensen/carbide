use crate::draw::stroke::{StrokeDashCap, StrokeDashMode};

#[derive(Clone, Debug, PartialEq)]
pub struct StrokeDashPattern {
    pub pattern: Vec<f64>,
    pub offset: f64,
    pub start_cap: StrokeDashCap,
    pub end_cap: StrokeDashCap,
    pub dash_type: StrokeDashMode,
}