use serde::{Serialize, Deserialize};
use carbide_core::state::State;

pub type CheckBoxState<GS> = Box<dyn State<CheckBoxValue, GS>>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CheckBoxValue {
    True,
    Intermediate,
    False
}

impl Default for CheckBoxValue {
    fn default() -> Self {
        CheckBoxValue::False
    }
}