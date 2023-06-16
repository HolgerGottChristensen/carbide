// Which keys can be selected enum
use crate::environment::EnvironmentColor;
use crate::environment::EnvironmentFontSize;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum EnvironmentStateKey {
    String(String),
    Color(EnvironmentColor),
    FontSize(EnvironmentFontSize),
}
