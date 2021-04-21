use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ShapeStyle {
    Default,
    Fill,
    Stroke,
    FillAndStroke,
}

impl ShapeStyle {
    pub fn add_style(&self, style: ShapeStyle) -> ShapeStyle {
        match (self, style) {
            (_, ShapeStyle::FillAndStroke) |
            (ShapeStyle::FillAndStroke, _) => ShapeStyle::FillAndStroke,

            (ShapeStyle::Default, ShapeStyle::Default) => ShapeStyle::Default,
            (ShapeStyle::Default, ShapeStyle::Fill) => ShapeStyle::Fill,
            (ShapeStyle::Default, ShapeStyle::Stroke) => ShapeStyle::Stroke,

            (ShapeStyle::Fill, ShapeStyle::Stroke) => ShapeStyle::FillAndStroke,
            (ShapeStyle::Stroke, ShapeStyle::Fill) => ShapeStyle::FillAndStroke,

            (ShapeStyle::Stroke, _) => ShapeStyle::Stroke,
            (ShapeStyle::Fill, _) => ShapeStyle::Fill,
        }
    }
}

impl Add for ShapeStyle {
    type Output = ShapeStyle;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_style(rhs)
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