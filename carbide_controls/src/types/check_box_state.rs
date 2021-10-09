use serde::{Deserialize, Serialize};

use carbide_core::state::{BoolState, MapOwnedState, TState};

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

/*impl From<BoolState> for CheckBoxState {
    fn from(self) -> CheckBoxState {
        MapOwnedState::new_with_default_and_rev(
            self,
            |from: &bool, _:&_, _:&_| {
                if *from {
                    CheckBoxValue::True
                } else {
                    CheckBoxValue::False
                }
            },
            |to: &CheckBoxValue| {
                match to {
                    CheckBoxValue::True => {
                        true
                    }
                    CheckBoxValue::Intermediate | CheckBoxValue::False => {
                        false
                    }
                }
            },
            false
        ).into()
    }
}*/
