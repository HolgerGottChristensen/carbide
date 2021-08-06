use uuid::Uuid;

// Which keys can be selected enum
use crate::prelude::EnvironmentColor;
use crate::prelude::EnvironmentFontSize;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum StateKey {
    String(String),
    Uuid(Uuid),
    Color(EnvironmentColor),
    FontSize(EnvironmentFontSize),
}