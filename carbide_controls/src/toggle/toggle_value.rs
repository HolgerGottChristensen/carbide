use carbide::state::ValueRefMut;
use carbide_core::impl_state_value;
use carbide_core::state::{AnyReadState, AnyState, ConvertInto, ConvertIntoRead, Map1, RMap1, RWMap1};

#[derive(Clone, Debug, PartialEq)]
pub enum ToggleValue {
    True,
    Mixed,
    False,
}

impl Default for ToggleValue {
    fn default() -> Self {
        ToggleValue::False
    }
}

impl_state_value!(ToggleValue);

// ---------------------------------------------------
//  Conversion implementations
// ---------------------------------------------------

impl ConvertIntoRead<ToggleValue> for bool {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&bool)-> ToggleValue, bool, ToggleValue, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |value| {
            if *value {
                ToggleValue::True
            } else {
                ToggleValue::False
            }
        })
    }
}

impl ConvertInto<ToggleValue> for bool {
    type Output<G: AnyState<T=Self> + Clone> = RWMap1<fn(&bool)-> ToggleValue, fn(ToggleValue, ValueRefMut<bool>), bool, ToggleValue, G>;

    fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::map(f, |value| {
            if *value { ToggleValue::True } else { ToggleValue::False }
        }, |new, mut val| {
            match new {
                 ToggleValue::True => *val = true,
                 ToggleValue::Mixed | ToggleValue::False => *val = false,
             }
        })
    }
}