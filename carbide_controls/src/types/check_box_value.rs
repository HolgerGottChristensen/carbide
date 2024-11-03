use carbide::state::ValueRefMut;
use carbide_core::impl_state_value;
use carbide_core::state::{AnyReadState, AnyState, ConvertInto, ConvertIntoRead, Map1, RMap1, RWMap1};

#[derive(Clone, Debug, PartialEq)]
pub enum CheckBoxValue {
    True,
    Mixed,
    False,
}

impl Default for CheckBoxValue {
    fn default() -> Self {
        CheckBoxValue::False
    }
}

impl_state_value!(CheckBoxValue);

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
                 CheckBoxValue::Mixed | CheckBoxValue::False => *val = false,
             }
        })
    }
}