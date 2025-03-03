use std::ops::{Add, AddAssign};
use crate::widget::StrokeStyle;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShapeStyle {
    Default,
    Fill,
    Stroke { line_width: f64 },
    FillAndStroke { line_width: f64 },
}

impl Add for ShapeStyle {
    type Output = ShapeStyle;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ShapeStyle::Default, _) => rhs,
            (_, ShapeStyle::Default) => self,
            
            (ShapeStyle::FillAndStroke { .. }, ShapeStyle::FillAndStroke { line_width }) => ShapeStyle::FillAndStroke { line_width },
            (ShapeStyle::FillAndStroke { line_width }, ShapeStyle::Fill) => ShapeStyle::FillAndStroke { line_width },
            (ShapeStyle::FillAndStroke { .. }, ShapeStyle::Stroke { line_width }) => ShapeStyle::FillAndStroke { line_width },

            (ShapeStyle::Stroke { line_width }, ShapeStyle::Fill) => ShapeStyle::FillAndStroke { line_width },
            (ShapeStyle::Stroke { .. }, ShapeStyle::Stroke { line_width }) => ShapeStyle::Stroke { line_width },
            (ShapeStyle::Stroke { .. }, ShapeStyle::FillAndStroke { line_width }) => ShapeStyle::FillAndStroke { line_width },
            
            (ShapeStyle::Fill, ShapeStyle::Stroke { line_width }) => ShapeStyle::FillAndStroke { line_width },
            (ShapeStyle::Fill, ShapeStyle::Fill) => ShapeStyle::Fill,
            (ShapeStyle::Fill, ShapeStyle::FillAndStroke { line_width }) => ShapeStyle::FillAndStroke { line_width },
        }
    }
}

impl AddAssign for ShapeStyle {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs
    }
}

impl Default for ShapeStyle {
    fn default() -> Self {
        ShapeStyle::Default
    }
}
