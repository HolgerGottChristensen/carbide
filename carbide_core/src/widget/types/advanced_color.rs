use carbide_core::Color;
use carbide_core::widget::Gradient;

#[derive(Debug, Clone, PartialEq)]
pub enum AdvancedColor {
    Color(Color),
    SingleGradient(Gradient),
    MultiGradient(Vec<Gradient>),
}

impl Default for AdvancedColor {
    fn default() -> Self {
        AdvancedColor::Color(Color::default())
    }
}

impl From<Color> for AdvancedColor {
    fn from(c: Color) -> Self {
        AdvancedColor::Color(c)
    }
}
