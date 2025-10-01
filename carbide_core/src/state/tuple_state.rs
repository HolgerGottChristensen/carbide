use std::rc::Rc;

use crate::environment::Environment;
use crate::state::{
    StateSync, ReadState, State, Functor, IntoReadState, Fn2,
    StateContract, ValueRef, ValueRefMut, AnyReadState, AnyState, InnerState, ValueCell,
};

macro_rules! tuple_state {
    ($struct_name:ident, $map_name:ident, $read_map_name:ident, $env_map_name:ident, $map_name_owned:ident, $($name:ident : $type:ident -> $type2:ident),*) => {

        /// The struct used to create mappings between states. Its methods delegates to the
        /// correct map implementation.
        pub struct $struct_name;

        impl $struct_name {
            #[allow(unused_parens)]
            pub fn read_map<$($type: StateContract),*, TO: StateContract, $($type2: AnyReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static>($($name: $type2),*, map: MAP) -> $read_map_name<MAP, $($type),*, TO, $($type2),*> {
                $read_map_name {
                    $(
                        $name,
                    )*
                    map,
                }
            }

            #[allow(unused_parens)]
            pub fn read_map_env<$($type: StateContract),*, TO: StateContract + Default, $($type2: AnyReadState<T=$type> + Clone + 'static),*, MAP: Fn(&mut Environment, $(&$type),*) -> TO + Clone + 'static>($($name: $type2),*, map: MAP) -> $env_map_name<MAP, $($type),*, TO, $($type2),*> {
                $env_map_name {
                    $(
                        $name,
                    )*
                    map,
                    value: Default::default(),
                }
            }


            #[allow(unused_parens)]
            pub fn map<$($type: StateContract),*, TO: StateContract, $($type2: AnyState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static, REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static>($($name: $type2),*, map: MAP, replace: REPLACE) -> $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> {
                $map_name {
                    $(
                        $name,
                    )*
                    map,
                    replace,
                }
            }

            #[allow(unused_parens)]
            pub fn map_owned<$($type: StateContract),*, TO: StateContract, $($type2: AnyState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*, &mut TO) + Clone + 'static, REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static>($($name: $type2),*, map: MAP, replace: REPLACE, default: TO) -> $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> {
                $map_name_owned {
                    $(
                        $name,
                    )*
                    value: Rc::new(ValueCell::new(default)),
                    map,
                    replace,
                }
            }
        }


        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> where
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static,
            REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        {
            $(
                $name: $type2,
            )*
            map: MAP,
            replace: REPLACE,
        }

        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> where
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*, &mut TO) + Clone + 'static,
            REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        {
            $(
                $name: $type2,
            )*
            value: InnerState<TO>,
            map: MAP,
            replace: REPLACE,
        }

        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $read_map_name<MAP, $($type),*, TO, $($type2),*> where
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static
        {
            $(
                $name: $type2,
            )*
            map: MAP,
        }

        #[derive(Clone)]
        #[allow(unused_parens)]
        pub struct $env_map_name<MAP, $($type),*, TO, $($type2),*> where
            $($type: StateContract),*,
            TO: StateContract + Default,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn(&mut Environment, $(&$type),*) -> TO + Clone + 'static
        {
            $(
                $name: $type2,
            )*
            map: MAP,
            value: TO,
        }

        impl<
            V: StateContract,
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static,
            REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > Functor<V> for $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> where $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*>: IntoReadState<V> {
            // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
            type Output<G: StateContract, F: Fn2<V, G>> = RMap1<F, V, G, <$map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> as IntoReadState<V>>::Output>;

            fn map<U: StateContract, F: Fn2<V, U>>(self, f: F) -> Self::Output<U, F> {
                Map1::read_map(self.into_read_state(), f)
            }
        }

        impl<
            V: StateContract,
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*, &mut TO) + Clone + 'static,
            REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > Functor<V> for $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> where $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*>: IntoReadState<V> {
            // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
            type Output<G: StateContract, F: Fn2<V, G>> = RMap1<F, V, G, <$map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> as IntoReadState<V>>::Output>;

            fn map<U: StateContract, F: Fn2<V, U>>(self, f: F) -> Self::Output<U, F> {
                Map1::read_map(self.into_read_state(), f)
            }
        }

        impl<
            V: StateContract,
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static
        > Functor<V> for $read_map_name<MAP, $($type),*, TO, $($type2),*> where $read_map_name<MAP, $($type),*, TO, $($type2),*>: IntoReadState<V> {
            // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
            type Output<G: StateContract, F: Fn2<V, G>> = RMap1<F, V, G, <$read_map_name<MAP, $($type),*, TO, $($type2),*> as IntoReadState<V>>::Output>;

            fn map<U: StateContract, F: Fn2<V, U>>(self, f: F) -> Self::Output<U, F> {
                Map1::read_map(self.into_read_state(), f)
            }
        }

        impl<
            V: StateContract,
            $($type: StateContract),*,
            TO: StateContract + Default,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn(&mut Environment, $(&$type),*) -> TO + Clone + 'static
        > Functor<V> for $env_map_name<MAP, $($type),*, TO, $($type2),*> where $env_map_name<MAP, $($type),*, TO, $($type2),*>: IntoReadState<V> {
            // Can be simplified once this is stabilized: https://github.com/rust-lang/rust/issues/63063
            type Output<G: StateContract, F: Fn2<V, G>> = RMap1<F, V, G, <$env_map_name<MAP, $($type),*, TO, $($type2),*> as IntoReadState<V>>::Output>;

            fn map<U: StateContract, F: Fn2<V, U>>(self, f: F) -> Self::Output<U, F> {
                Map1::read_map(self.into_read_state(), f)
            }
        }


        /// Implement NewStateSync for the RMap
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static
        > StateSync for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                updated
            }
        }

        /// Implement NewStateSync for the RWMap
        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static,
            REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > StateSync for $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                updated
            }
        }

        /// Implement NewStateSync for the RWOMap
        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*, &mut TO) + Clone + 'static,
            REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > StateSync for $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                let mut updated = false;

                $(
                    updated |= self.$name.sync(env);
                )*

                updated
            }
        }

        /// Implement NewStateSync for the EnvMap
        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract + Default,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn(&mut Environment, $(&$type),*) -> TO + Clone + 'static
        > StateSync for $env_map_name<MAP, $($type),*, TO, $($type2),*> {
            fn sync(&mut self, env: &mut Environment) -> bool {
                $(
                    self.$name.sync(env);
                )*

                self.value = (self.map)(env, $(&*self.$name.value()),*);

                true // We could check if the value changed with PartialEq
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static
        > AnyReadState for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            type T = TO;
            fn value_dyn(&self) -> ValueRef<'_, TO> {
                let val = (self.map)($(&*self.$name.value()),*);
                ValueRef::Owned(val)
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static,
            REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > AnyReadState for $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            type T = TO;
            fn value_dyn(&self) -> ValueRef<'_, TO> {
                let val = (self.map)($(&*self.$name.value()),*);
                ValueRef::Owned(val)
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*, &mut TO) + Clone + 'static,
            REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > AnyReadState for $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            type T = TO;
            fn value_dyn(&self) -> ValueRef<'_, TO> {
                (self.map)($(&*self.$name.value()),*, &mut *self.value.borrow_mut());
                self.value.borrow()
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract + Default,
            $($type2: AnyReadState<T=$type> + Clone + 'static),*,
            MAP: Fn(&mut Environment, $(&$type),*) -> TO + Clone + 'static
        > AnyReadState for $env_map_name<MAP, $($type),*, TO, $($type2),*> {
            type T = TO;
            fn value_dyn(&self) -> ValueRef<'_, TO> {
                ValueRef::Borrow(&self.value)
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static,
            REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > AnyState for $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            fn value_dyn_mut(&mut self) -> ValueRefMut<'_, TO> {
                // Get the current value
                let val = (self.map)($(&*self.$name.value()),*);

                // Clone self to get static lifetime
                let mut setter_self = self.clone();

                // Call set_value_dyn when ValueRefMut is dropped
                let setter = move |new: TO| {
                    setter_self.set_value_dyn(new);
                };

                ValueRefMut::TupleState(Some(Box::new(setter)), Some(val))
            }

            /// Set value will only update its containing state if the map_rev is specified.
            #[allow(unused_parens)]
            fn set_value_dyn(&mut self, value: TO) {
                (self.replace)(value, $(self.$name.value_mut()),*);
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*, &mut TO) + Clone + 'static,
            REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > AnyState for $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            fn value_dyn_mut(&mut self) -> ValueRefMut<'_, TO> {
                // Get the current value
                (self.map)($(&*self.$name.value()),*, &mut self.value.borrow_mut());

                // Clone self to get static lifetime
                let mut setter_self = self.clone();

                // Call set_value_dyn when ValueRefMut is dropped
                let setter = move |new: TO| {
                    setter_self.set_value_dyn(new);
                };

                ValueRefMut::TupleState(Some(Box::new(setter)), Some(self.value.borrow().clone()))
            }

            /// Set value will only update its containing state if the map_rev is specified.
            #[allow(unused_parens)]
            fn set_value_dyn(&mut self, value: TO) {
                *self.value.borrow_mut() = value;
                (self.replace)(&*self.value.borrow(), $(self.$name.value_mut()),*);
            }
        }

        #[allow(unused_parens)]
        impl<$($type: StateContract),*, TO: StateContract, $($type2: AnyReadState<T=$type> + Clone + 'static),*, MAP: Fn($(&$type),*) -> TO + Clone + 'static> core::fmt::Debug for $read_map_name<MAP, $($type),*, TO, $($type2),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($read_map_name))
                    $(
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }

        #[allow(unused_parens)]
        impl<$($type: StateContract),*, TO: StateContract + Default, $($type2: AnyReadState<T=$type> + Clone + 'static),*, MAP: Fn(&mut Environment, $(&$type),*) -> TO + Clone + 'static> core::fmt::Debug for $env_map_name<MAP, $($type),*, TO, $($type2),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($env_map_name))
                    $(
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*) -> TO + Clone + 'static,
            REPLACE: Fn(TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > core::fmt::Debug for $map_name<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($map_name))
                    $(
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }

        #[allow(unused_parens)]
        impl<
            $($type: StateContract),*,
            TO: StateContract,
            $($type2: AnyState<T=$type> + Clone + 'static),*,
            MAP: Fn($(&$type),*, &mut TO) + Clone + 'static,
            REPLACE: Fn(&TO, $(ValueRefMut<$type>),*) + Clone + 'static,
        > core::fmt::Debug for $map_name_owned<MAP, REPLACE, $($type),*, TO, $($type2),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($map_name_owned))
                    $(
                    .field(stringify!($name), &*self.$name.value())
                    )*
                    .finish()
            }
        }
    };
}

tuple_state!(Map1,  RWMap1,  RMap1,  EnvMap1,  RWOMap1,  s1: T1 -> T1State);
tuple_state!(Map2,  RWMap2,  RMap2,  EnvMap2,  RWOMap2,  s1: T1 -> T1State, s2: T2 -> T2State);
tuple_state!(Map3,  RWMap3,  RMap3,  EnvMap3,  RWOMap3,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State);
tuple_state!(Map4,  RWMap4,  RMap4,  EnvMap4,  RWOMap4,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State);
tuple_state!(Map5,  RWMap5,  RMap5,  EnvMap5,  RWOMap5,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State);
tuple_state!(Map6,  RWMap6,  RMap6,  EnvMap6,  RWOMap6,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State);
tuple_state!(Map7,  RWMap7,  RMap7,  EnvMap7,  RWOMap7,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State);
tuple_state!(Map8,  RWMap8,  RMap8,  EnvMap8,  RWOMap8,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State);
tuple_state!(Map9,  RWMap9,  RMap9,  EnvMap9,  RWOMap9,  s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State);
tuple_state!(Map10, RWMap10, RMap10, EnvMap10, RWOMap10, s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State, s10: T10 -> T10State);
tuple_state!(Map11, RWMap11, RMap11, EnvMap11, RWOMap11, s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State, s10: T10 -> T10State, s11: T11 -> T11State);
tuple_state!(Map12, RWMap12, RMap12, EnvMap12, RWOMap12, s1: T1 -> T1State, s2: T2 -> T2State, s3: T3 -> T3State, s4: T4 -> T4State, s5: T5 -> T5State, s6: T6 -> T6State, s7: T7 -> T7State, s8: T8 -> T8State, s9: T9 -> T9State, s10: T10 -> T10State, s11: T11 -> T11State, s12: T12 -> T12State);
