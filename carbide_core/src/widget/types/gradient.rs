use crate::draw::{Position, Color, Alignment};

#[derive(Debug, Clone, PartialEq)]
pub enum GradientPosition {
    Absolute(Position),
    Relative(f64, f64),
    Alignment(Alignment),
}

/// The different types of gradients in carbide.
#[derive(Debug, Clone, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
    Diamond,
    Conic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GradientRepeat {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub colors: Vec<Color>,
    pub ratios: Vec<f32>,

    pub gradient_type: GradientType,
    pub gradient_repeat: GradientRepeat,

    pub start: GradientPosition,
    pub end: GradientPosition,
}

impl Gradient {
    /// Creates a linear gradient from the start to the end. The colors are split up equally.
    /// If you don't want the equal split, see [Gradient::linear_ratios()].
    /// If the points are on top of each other, the last color in the list of color will show.
    /// This is probably not intended and should not be used to fill a shape. Use a normal color
    /// instead.
    pub fn linear(
        colors: Vec<Color>,
        start: impl Into<GradientPosition>,
        end: impl Into<GradientPosition>,
    ) -> Self {
        let number_of_colors = (colors.len() - 1) as f32;
        let ratios = colors
            .iter()
            .enumerate()
            .map(|a| a.0 as f32 / number_of_colors)
            .collect::<Vec<_>>();
        Self {
            colors,
            ratios,
            gradient_type: GradientType::Linear,
            gradient_repeat: GradientRepeat::Clamp,
            start: start.into(),
            end: end.into(),
        }
    }

    /// Creates a radial gradient with the center point and the outer edge. The first color is at the
    /// center and the last color is at the edge. The colors are equally split between the center and end.
    /// If this is not intended and you want to decide the ratios, see [Gradient::radial_ratios()].
    /// If the points are on top of each other, the last color in the list of color will show.
    /// This is probably not intended and should not be used to fill a shape. Use a normal color
    /// instead.
    pub fn radial(
        colors: Vec<Color>,
        center: impl Into<GradientPosition>,
        edge: impl Into<GradientPosition>,
    ) -> Self {
        let number_of_colors = (colors.len() - 1) as f32;
        let ratios = colors
            .iter()
            .enumerate()
            .map(|a| a.0 as f32 / number_of_colors)
            .collect::<Vec<_>>();
        Self {
            colors,
            ratios,
            gradient_type: GradientType::Radial,
            gradient_repeat: GradientRepeat::Clamp,
            start: center.into(),
            end: edge.into(),
        }
    }

    /// Creates a diamond gradient with the center point and the end point at the corner of the diamond.
    /// The first color is at the center and the last color is at the corner of the diamond. The colors
    /// are equally split between the center and corner. If this is not intended, take a look at
    /// [Gradient::diamond_ratios()]. If the points are on top of each other, the last color in the
    /// list will be shown. This is probably not intended and should not be used to fill a shape.
    /// Use a normal color instead.
    pub fn diamond(
        colors: Vec<Color>,
        center: impl Into<GradientPosition>,
        end: impl Into<GradientPosition>,
    ) -> Self {
        let number_of_colors = (colors.len() - 1) as f32;
        let ratios = colors
            .iter()
            .enumerate()
            .map(|a| a.0 as f32 / number_of_colors)
            .collect::<Vec<_>>();
        Self {
            colors,
            ratios,
            gradient_type: GradientType::Diamond,
            gradient_repeat: GradientRepeat::Clamp,
            start: center.into(),
            end: end.into(),
        }
    }

    /// Created a conic gradient with the center point at the center and the direction of the end point.
    /// The first color is on the left side of the line defined by the center and the direction and
    /// the colors are moving in the clockwise direction. The colors are split equally. If you need
    /// to customize this, see [Gradient::conic_ratios()]. If the points are on top of each other
    /// the line will point right.
    pub fn conic(
        colors: Vec<Color>,
        center: impl Into<GradientPosition>,
        end: impl Into<GradientPosition>,
    ) -> Self {
        let number_of_colors = (colors.len() - 1) as f32;
        let ratios = colors
            .iter()
            .enumerate()
            .map(|a| a.0 as f32 / number_of_colors)
            .collect::<Vec<_>>();
        Self {
            colors,
            ratios,
            gradient_type: GradientType::Conic,
            gradient_repeat: GradientRepeat::Clamp,
            start: center.into(),
            end: end.into(),
        }
    }

    pub fn linear_ratios(
        colors: Vec<(Color, f32)>,
        start: impl Into<GradientPosition>,
        end: impl Into<GradientPosition>,
    ) -> Self {
        let cols = colors.iter().map(|(c, _)| *c).collect();
        let ratios = colors.iter().map(|(_, r)| *r).collect();
        Self {
            colors: cols,
            ratios,
            gradient_type: GradientType::Linear,
            gradient_repeat: GradientRepeat::Clamp,
            start: start.into(),
            end: end.into(),
        }
    }

    pub fn radial_ratios(
        colors: Vec<(Color, f32)>,
        center: impl Into<GradientPosition>,
        edge: impl Into<GradientPosition>,
    ) -> Self {
        let cols = colors.iter().map(|(c, _)| *c).collect();
        let ratios = colors.iter().map(|(_, r)| *r).collect();
        Self {
            colors: cols,
            ratios,
            gradient_type: GradientType::Radial,
            gradient_repeat: GradientRepeat::Clamp,
            start: center.into(),
            end: edge.into(),
        }
    }

    pub fn diamond_ratios(
        colors: Vec<(Color, f32)>,
        center: impl Into<GradientPosition>,
        end: impl Into<GradientPosition>,
    ) -> Self {
        let cols = colors.iter().map(|(c, _)| *c).collect();
        let ratios = colors.iter().map(|(_, r)| *r).collect();
        Self {
            colors: cols,
            ratios,
            gradient_type: GradientType::Diamond,
            gradient_repeat: GradientRepeat::Clamp,
            start: center.into(),
            end: end.into(),
        }
    }

    pub fn conic_ratios(
        colors: Vec<(Color, f32)>,
        center: impl Into<GradientPosition>,
        end: impl Into<GradientPosition>,
    ) -> Self {
        let cols = colors.iter().map(|(c, _)| *c).collect();
        let ratios = colors.iter().map(|(_, r)| *r).collect();
        Self {
            colors: cols,
            ratios,
            gradient_type: GradientType::Conic,
            gradient_repeat: GradientRepeat::Clamp,
            start: center.into(),
            end: end.into(),
        }
    }

    /// This is the default mode for all gradients. Outside of the gradient the color is clamped
    /// to the start color on one end and the end color at the end.
    pub fn clamp(mut self) -> Self {
        self.gradient_repeat = GradientRepeat::Clamp;
        self
    }

    /// Mirror the gradient at the ends. This means for example a gradient from red to blue will be
    /// red -> blue blue -> red red -> blue blue -> red ...
    pub fn mirror(mut self) -> Self {
        self.gradient_repeat = GradientRepeat::Mirror;
        self
    }

    /// Repeat the gradient at the end points. This means for example a gradient from red to blue
    /// will be red -> blue red -> blue red -> blue red -> blue ...
    pub fn repeat(mut self) -> Self {
        self.gradient_repeat = GradientRepeat::Repeat;
        self
    }
}

impl Into<GradientPosition> for Alignment {
    fn into(self) -> GradientPosition {
        GradientPosition::Alignment(self)
    }
}

impl Into<GradientPosition> for Position {
    fn into(self) -> GradientPosition {
        GradientPosition::Absolute(self)
    }
}

impl Into<GradientPosition> for (f64, f64) {
    fn into(self) -> GradientPosition {
        GradientPosition::Relative(self.0, self.1)
    }
}
