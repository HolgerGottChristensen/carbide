use serde::{Deserialize, Serialize};

use carbide_core::state::TState;

pub type CheckBoxState = TState<CheckBoxValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CheckBoxValue {
    True,
    Intermediate,
    False,
}

impl Default for CheckBoxValue {
    fn default() -> Self {
        CheckBoxValue::False
    }
}
