use crate::environment::Environment;
use crate::state::{
    CacheRState, CacheTState, NewStateSync, ReadState, ReadWidgetState, RState, State,
    StateContract, TState, ValueRef, ValueRefMut, WidgetState
};

macro_rules! tuple_state {
    ($struct_name:ident, $map_name:ident, $read_map_name:ident, $($name:ident : $type:ident -> $type2:ident),*) => {

        /// The struct used to create mappings between states. Its methods delegates to the
        /// correct map implementation.
        pub struct $struct_name;

        impl $struct_name {
            pub fn read_map<$($type: StateContract),*, TO: StateContract, $($type2: ReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static>($($name: $type2),*, map: MAP) -> $read_map_name<MAP, $($type),*, TO, $($type2),*> {
                $read_map_name {
                    $(
                        $name,
                    )*
                    map,
                }
            }

            pub fn read_map_cached<$($type: StateContract),*, TO: StateContract>($($name: impl Into<RState<$type>>),*, map: fn($($name: &$type),*) -> TO) -> RState<TO> {
                let n = $read_map_name {
                    $(
                        $name: $name.into(),
                    )*
                    map,
                };
                CacheRState::new(ReadWidgetState::new(Box::new(n)))
            }

            #[allow(unused_parens)]
            pub fn map<$($type: StateContract),*, TO: StateContract>($($name: impl Into<TState<$type>>),*, map: fn($($name: &$type),*) -> TO, replace: fn(TO, $($name: &$type),*) -> ($(Option<$type>),*)) -> TState<TO> {
                let n = $map_name {
                    $(
                        $name: $name.into(),
                    )*
                    map,
                    replace,
                };
                WidgetState::new(Box::new(n))
            }

            #[allow(unused_parens)]
            pub fn map_cached<$($type: StateContract),*, TO: StateContract>($($name: impl Into<TState<$type>>),*, map: fn($($name: &$type),*) -> TO, replace: fn(TO, $($name: &$type),*) -> ($(Option<$type>),*)) -> TState<TO> {
                let n = $map_name {
                    $(
                        $name: $name.into(),
                    )*
                    map,
                    replace,
                };
                CacheTState::new(WidgetState::new(Box::new(n)))
            }
        }


        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $map_name<$($type),*, TO> where $($type: StateContract),*, TO: StateContract {
            $(
                $name: TState<$type>,
            )*
            map: fn($($name: &$type),*) -> TO,
            replace: fn(TO, $($name: &$type),*) -> ($(Option<$type>),*),
        }

        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $read_map_name<MAP, $($type),*, TO, $($type2),*> where $($type: StateContract),*, TO: StateContract, $($type2: ReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static {
            $(
                $name: $type2,
            )*
            map: MAP,
        }

        /// Implement NewStateSync for the RMap
        impl<$($type: StateContract),*, TO: StateContract, $($type2: ReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static> NewStateSync for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                updated
            }
        }

        /// Implement NewStateSync for the RWMap
        impl<$($type: StateContract),*, TO: StateContract> NewStateSync for $map_name<$($type),*, TO> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                updated
            }
        }

        impl<$($type: StateContract),*, TO: StateContract, $($type2: ReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static> ReadState for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            type T = TO;
            fn value(&self) -> ValueRef<TO> {
                let val = (self.map)($(&*self.$name.value()),*);
                ValueRef::Owned(val)
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> ReadState for $map_name<$($type),*, TO> {
            type T = TO;
            fn value(&self) -> ValueRef<TO> {
                let val = (self.map)($(&*self.$name.value()),*);
                ValueRef::Owned(val)
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> State for $map_name<$($type),*, TO> {
            fn value_mut(&mut self) -> ValueRefMut<TO> {
                panic!("You can not set the value of a map state this way. Please use the set_state macro instead")
            }

            /// Set value will only update its containing state if the map_rev is specified.
            #[allow(unused_parens)]
            fn set_value(&mut self, value: TO) {
                let ($($name),*) = (self.replace)(value, $(&*self.$name.value()),*);

                $(
                    if let Some($name) = $name {
                        self.$name.set_value($name);
                    }
                )*
            }

            fn update_dependent(&mut self) {}
        }

        impl<$($type: StateContract),*, TO: StateContract, $($type2: ReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static> core::fmt::Debug for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($read_map_name))
                    $(
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }

        impl<$($type: StateContract),*, TO: StateContract> core::fmt::Debug for $map_name<$($type),*, TO> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($map_name))
                    $(
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }


        /// Implement NewStateSync for the RMap
        impl<$($type: StateContract),*, TO: StateContract, $($type2: ReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static> carbide_core::state::IntoReadState<TO> for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            type Output = Self;

            fn into_read_state(self) -> Self::Output {
                self
            }
        }
    };
}

tuple_state!(Map1,  RWMap1,  RMap1,  s1: T1 -> T1State);
tuple_state!(Map2,  RWMap2,  RMap2,  s1: T1 -> T1State, s2: T2 -> T2State);
tuple_state!(Map3,  RWMap3,  RMap3,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State);
tuple_state!(Map4,  RWMap4,  RMap4,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State);
tuple_state!(Map5,  RWMap5,  RMap5,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State);
tuple_state!(Map6,  RWMap6,  RMap6,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State);
tuple_state!(Map7,  RWMap7,  RMap7,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State);
tuple_state!(Map8,  RWMap8,  RMap8,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State);
tuple_state!(Map9,  RWMap9,  RMap9,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State);
tuple_state!(Map10, RWMap10, RMap10, s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State, s10: T10 -> T10State);
tuple_state!(Map11, RWMap11, RMap11, s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State, s10: T10 -> T10State, s11: T11 -> T11State);
tuple_state!(Map12, RWMap12, RMap12, s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State, s10: T10 -> T10State, s11: T11 -> T11State, s12: T12 -> T12State);
