use carbide::state::ValueRefMut;
use carbide_core::impl_read_state;
use carbide_core::state::{AnyReadState, AnyState, ConvertInto, ConvertIntoRead, Map1, RMap1, RWMap1};

// use carbide_core::environment::Environment;
// use carbide_core::state::{MapOwnedState, NewStateSync, ReadState, RState, State, TState, ValueRef, ValueRefMut};
//
// #[derive(Clone, Debug)]
// pub struct CheckBoxState(TState<CheckBoxValue>);
//
#[derive(Clone, Debug, PartialEq)]
pub enum CheckBoxValue {
    True,
    Indeterminate,
    False,
}

impl Default for CheckBoxValue {
    fn default() -> Self {
        CheckBoxValue::False
    }
}

impl_read_state!(CheckBoxValue);

// ---------------------------------------------------
//  Conversion implementations
// ---------------------------------------------------

impl ConvertIntoRead<CheckBoxValue> for bool {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&bool)->CheckBoxValue, bool, CheckBoxValue, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |value| {
            if *value {
                CheckBoxValue::True
            } else {
                CheckBoxValue::False
            }
        })
    }
}

impl ConvertInto<CheckBoxValue> for bool {
    type Output<G: AnyState<T=Self> + Clone> = RWMap1<fn(&bool)->CheckBoxValue, fn(CheckBoxValue, ValueRefMut<bool>), bool, CheckBoxValue, G>;

    fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::map(f, |value| {
            if *value { CheckBoxValue::True } else { CheckBoxValue::False }
        }, |new, mut val| {
            match new {
                 CheckBoxValue::True => *val = true,
                 CheckBoxValue::Indeterminate | CheckBoxValue::False => *val = false,
             }
        })
    }
}


/*impl<T> IntoReadStateHelper<T, bool, CheckBoxValue> for T where T: AnyReadState<T=bool> + Clone {
    type Output = RMap1<fn(&bool)->CheckBoxValue, bool, CheckBoxValue, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map(self, |value| {
            if *value {
                CheckBoxValue::True
            } else {
                CheckBoxValue::False
            }
        })
    }
}*/

//
// impl NewStateSync for CheckBoxState {
//     fn sync(&mut self, env: &mut Environment) -> bool {
//         self.0.sync(env)
//     }
// }
//
// impl ReadState<CheckBoxValue> for CheckBoxState {
//     fn value(&self) -> ValueRef<CheckBoxValue> {
//         self.0.value()
//     }
// }
//
// impl State<CheckBoxValue> for CheckBoxState {
//     fn value_mut(&mut self) -> ValueRefMut<CheckBoxValue> {
//         self.0.value_mut()
//     }
//
//     fn set_value(&mut self, value: CheckBoxValue) {
//         self.0.set_value(value)
//     }
//
//     fn update_dependent(&mut self) {
//         self.0.update_dependent()
//     }
// }
//
// impl Into<CheckBoxState> for TState<CheckBoxValue> {
//     fn into(self) -> CheckBoxState {
//         CheckBoxState(self)
//     }
// }
//
// impl CheckBoxState {
//     fn from_bool(from: &bool, current: &CheckBoxValue, env: &Environment) -> CheckBoxValue {
//         if *from {
//             CheckBoxValue::True
//         } else {
//             CheckBoxValue::False
//         }
//     }
//
//     fn val_to_bool(to: &CheckBoxValue) -> Option<bool> {
//         match to {
//             CheckBoxValue::True => Some(true),
//             CheckBoxValue::Intermediate | CheckBoxValue::False => Some(false),
//         }
//     }
// }
//
// impl Into<CheckBoxState> for TState<bool> {
//     fn into(self) -> CheckBoxState {
//         CheckBoxState(
//             MapOwnedState::new_with_default_and_rev(
//                 self,
//                 CheckBoxState::from_bool,
//                 CheckBoxState::val_to_bool,
//                 CheckBoxValue::False,
//             )
//             .into(),
//         )
//     }
// }
//
// impl Into<TState<CheckBoxValue>> for CheckBoxState {
//     fn into(self) -> TState<CheckBoxValue> {
//         self.0
//     }
// }
//
// impl Into<RState<CheckBoxValue>> for CheckBoxState {
//     fn into(self) -> RState<CheckBoxValue> {
//         self.0.into()
//     }
// }
