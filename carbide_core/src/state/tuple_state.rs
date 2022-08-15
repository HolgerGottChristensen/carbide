use crate::prelude::Environment;
use crate::state::{
    InnerState, NewStateSync, RState, ReadState, ReadWidgetState, State, StateContract, TState,
    ValueCell, ValueRef, ValueRefMut, WidgetState,
};

macro_rules! tuple_state {
    ($struct_name:ident, $($name:ident : $type:ident),*) => {
        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $struct_name<$($type),*, TO> where $($type: StateContract),*, TO: StateContract {
            $(
                $name: TState<$type>,
            )*
            inner_value: InnerState<Option<TO>>,
            map: fn($($name: &$type),*) -> TO,
            replace: Option<fn(TO, $($name: &$type),*) -> ($(Option<$type>),*)>,
        }

        impl<$($type: StateContract),*, TO: StateContract> NewStateSync for $struct_name<$($type),*, TO> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                let borrowed = &mut *self.inner_value.borrow_mut();

                if let Some(inner) = borrowed {
                    if updated {
                        *inner = (self.map)($(&*self.$name.value()),*);
                    }
                } else {
                    let val = (self.map)($(&*self.$name.value()),*);
                    *borrowed = Some(val);
                }

                updated
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> ReadState<TO> for $struct_name<$($type),*, TO> {
            fn value(&self) -> ValueRef<TO> {
                ValueRef::map(self.inner_value.borrow(), |v| {v.as_ref().expect("Tried to get value without having synced first. Maps are not initialized before the first sync")})
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> State<TO> for $struct_name<$($type),*, TO> {
            fn value_mut(&mut self) -> ValueRefMut<TO> {
                panic!("You can not set the value of a map state this way. Please use the set_state macro instead")
            }

            /// Set value will only update its containing state if the map_rev is specified.
            #[allow(unused_parens)]
            fn set_value(&mut self, value: TO) {
                if let Some(replace) = self.replace {
                    let ($($name),*) = (replace)(value, $(&*self.$name.value()),*);

                    $(
                        if let Some($name) = $name {
                            self.$name.set_value($name);
                        }
                    )*

                    let val = (self.map)($(&*self.$name.value()),*);

                    let borrowed = &mut *self.inner_value.borrow_mut();

                    if let Some(inner) = borrowed {
                        *inner = val;
                    } else {
                        *borrowed = Some(val);
                    }

                } else {
                    panic!("This should not be reachable.")
                }
            }

            fn update_dependent(&mut self) {}
        }

        impl<$($type: StateContract),*, TO: StateContract> core::fmt::Debug for $struct_name<$($type),*, TO> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($struct_name))
                    $(
                    .field("initialized", &self.inner_value.borrow().is_some())
                    .field("times_shared", &InnerState::strong_count(&self.inner_value))
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> $struct_name<$($type),*, TO> {
            pub fn read_map($($name: impl Into<RState<$type>>),*, map: fn($($name: &$type),*) -> TO) -> RState<TO> {
                $(
                    let $name = $name.into().ignore_writes();
                )*

                let inner_value = InnerState::new(ValueCell::new(None));

                let n = Self {
                    $(
                        $name,
                    )*
                    map,
                    inner_value,
                    replace: None,
                };
                ReadWidgetState::new(Box::new(n))
            }

            pub fn read_map_test($($name: TState<$type>),*, map: fn($($name: &$type),*) -> TO) -> RState<TO> {
                let inner_value = InnerState::new(ValueCell::new(None));

                let n = Self {
                    $(
                        $name,
                    )*
                    map,
                    inner_value,
                    replace: None,
                };
                ReadWidgetState::new(Box::new(n))
            }

            #[allow(unused_parens)]
            pub fn map($($name: impl Into<TState<$type>>),*, map: fn($($name: &$type),*) -> TO, replace: fn(TO, $($name: &$type),*) -> ($(Option<$type>),*)) -> TState<TO> {
                $(
                    let $name = $name.into();
                )*

                let inner_value = InnerState::new(ValueCell::new(None));

                let n = Self {
                    $(
                        $name,
                    )*
                    map,
                    inner_value,
                    replace: Some(replace),
                };
                WidgetState::new(Box::new(n))
            }
        }
    };
}

tuple_state!(Map1, s1: T1);
tuple_state!(Map2, s1: T1, s2: T2);
tuple_state!(Map3, s1: T1, s2: T2, s3: T3);
tuple_state!(Map4, s1: T1, s2: T2, s3: T3, s4: T4);
tuple_state!(Map5, s1: T1, s2: T2, s3: T3, s4: T4, s5: T5);
tuple_state!(Map6, s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6);
tuple_state!(Map7, s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7);
tuple_state!(
    Map8,
    s1: T1,
    s2: T2,
    s3: T3,
    s4: T4,
    s5: T5,
    s6: T6,
    s7: T7,
    s8: T8
);
tuple_state!(
    Map9,
    s1: T1,
    s2: T2,
    s3: T3,
    s4: T4,
    s5: T5,
    s6: T6,
    s7: T7,
    s8: T8,
    s9: T9
);
tuple_state!(
    Map10,
    s1: T1,
    s2: T2,
    s3: T3,
    s4: T4,
    s5: T5,
    s6: T6,
    s7: T7,
    s8: T8,
    s9: T9,
    s10: T10
);
tuple_state!(
    Map11,
    s1: T1,
    s2: T2,
    s3: T3,
    s4: T4,
    s5: T5,
    s6: T6,
    s7: T7,
    s8: T8,
    s9: T9,
    s10: T10,
    s11: T11
);
tuple_state!(
    Map12,
    s1: T1,
    s2: T2,
    s3: T3,
    s4: T4,
    s5: T5,
    s6: T6,
    s7: T7,
    s8: T8,
    s9: T9,
    s10: T10,
    s11: T11,
    s12: T12
);
