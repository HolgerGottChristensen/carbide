use crate::state::{InnerState, ReadState, StateContract, ReadWidgetState, NewStateSync, ValueRef, State, ValueRefMut, TState, RState, ValueCell, WidgetState};
use crate::prelude::Environment;

macro_rules! tuple_state {
    ($struct_name:ident, $($name:ident : $type:ident),*) => {
        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $struct_name<$($type),*, TO> where $($type: StateContract),*, TO: StateContract {
            $(
                $name: TState<$type>,
            )*
            inner_value: Option<InnerState<TO>>,
            map: fn($($name: &$type),*) -> TO,
            replace: Option<fn(TO, $($name: &$type),*) -> ($(Option<$type>),*)>,
        }

        impl<$($type: StateContract),*, TO: StateContract> NewStateSync for $struct_name<$($type),*, TO> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                if let Some(inner) = &mut self.inner_value {
                    if updated {
                    *inner.borrow_mut() = (self.map)($(&*self.$name.value()),*);
                }
                } else {
                    let val = (self.map)($(&*self.$name.value()),*);
                    self.inner_value = Some(InnerState::new(ValueCell::new(val)));
                }

                updated
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> ReadState<TO> for $struct_name<$($type),*, TO> {
            fn value(&self) -> ValueRef<TO> {
                self.inner_value.as_ref().expect("Tried to get value without having synced first. Maps are not initialized before the first sync").borrow()
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

                    if let Some(inner) = &mut self.inner_value {
                        *inner.borrow_mut() = val;
                    } else {
                        self.inner_value = Some(InnerState::new(ValueCell::new(val)));
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
                //let val = map($(&*$name.value()),*);

                let inner_value = None;//InnerState::new(ValueCell::new(val));

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
                //let val = map($(&*$name.value()),*);

                let inner_value = None;//InnerState::new(ValueCell::new(val));

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

tuple_state!(Map1,  s1: T1);
tuple_state!(Map2,  s1: T1, s2: T2);
tuple_state!(Map3,  s1: T1, s2: T2, s3: T3);
tuple_state!(Map4,  s1: T1, s2: T2, s3: T3, s4: T4);
tuple_state!(Map5,  s1: T1, s2: T2, s3: T3, s4: T4, s5: T5);
tuple_state!(Map6,  s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6);
tuple_state!(Map7,  s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7);
tuple_state!(Map8,  s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7, s8: T8);
tuple_state!(Map9,  s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7, s8: T8, s9: T9);
tuple_state!(Map10, s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7, s8: T8, s9: T9, s10: T10);
tuple_state!(Map11, s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7, s8: T8, s9: T9, s10: T10, s11: T11);
tuple_state!(Map12, s1: T1, s2: T2, s3: T3, s4: T4, s5: T5, s6: T6, s7: T7, s8: T8, s9: T9, s10: T10, s11: T11, s12: T12);