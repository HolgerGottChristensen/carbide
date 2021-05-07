
#[derive(Debug, Clone, PartialEq)]
pub enum StrokeStyle {
    None,
    Solid {
        line_width: f64,
    }
}

impl StrokeStyle {
    pub fn get_line_width(&self) -> f64 {
        match self {
            StrokeStyle::None => 0.0,
            StrokeStyle::Solid { line_width } => *line_width,
        }
    }
}

impl Default for StrokeStyle {
    fn default() -> Self {
        StrokeStyle::Solid {line_width: 2.0}
    }
}