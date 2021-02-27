

// Which keys can be selected enum

use crate::state::environment_color::EnvironmentColor;
use crate::state::environment_font_size::EnvironmentFontSize;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum StateKey {
    String(String),
    Color(EnvironmentColor),
    FontSize(EnvironmentFontSize)
}