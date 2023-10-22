use std::fmt::Debug;
use carbide_core::state::AnyState;

use crate::environment::Environment;
use crate::state::{AnyReadState, IntoState, NewStateSync, StateContract};
use crate::state::{ReadState, State};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};

/// # FieldState
/// The FieldState is a state mapping that should be used to retrieve a field of a struct or enum
/// as a new state. This is done efficiently because we have two mapping functions that maps the
/// references to the original struct, to the references to the field.
///
/// You should almost never need to use this state directly. Instead you should use the `lens!`
/// macro as this will do all the dirty work of creating the mapping functions provided you
/// specify which field it should access.
///
/// When [NewStateSync::sync()] events are received, it will delegate if further to the
/// parent state, making sure the parent is up to date. FieldState does not need to handle sync
/// itself.
#[derive(Clone)]
pub struct FieldState<S, FROM, TO>
where
    S: AnyState<T=FROM> + Clone + 'static,
    FROM: StateContract,
    TO: StateContract,
{
    /// The state containing the original data to get the field from
    state: S,
    /// The immutable reference mapping that can return a reference to the field when provided a
    /// reference to the struct
    map: for<'r, 's> fn(&'r FROM) -> &'r TO,
    /// The mutable reference mapping that can return a mutable reference to the field when
    /// provided with a mutable reference to the struct.
    map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
}

impl<S: AnyState<T=FROM> + Clone + 'static, FROM: StateContract, TO: StateContract> FieldState<S, FROM, TO> {
    pub fn new(
        state: S,
        map: for<'r, 's> fn(&'r FROM) -> &'r TO,
        map_mut: for<'r, 's> fn(&'r mut FROM) -> &'r mut TO,
    ) -> FieldState<S, FROM, TO> {
        FieldState {
            state,
            map,
            map_mut,
        }
    }
}

impl<S: AnyState<T=FROM> + Clone + 'static, FROM: StateContract, TO: StateContract> NewStateSync for FieldState<S, FROM, TO> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.state.sync(env)
    }
}

impl<S: AnyState<T=FROM> + Clone + 'static, FROM: StateContract, TO: StateContract> AnyReadState for FieldState<S, FROM, TO> {
    type T = TO;
    fn value_dyn(&self) -> ValueRef<TO> {
        let map = self.map;
        ValueRef::map(self.state.value(), |a| map(a))
    }
}

impl<S: AnyState<T=FROM> + Clone + 'static, FROM: StateContract, TO: StateContract> AnyState for FieldState<S, FROM, TO> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<TO> {
        let map_mut = self.map_mut;
        ValueRefMut::map(self.state.value_mut(), move |a| map_mut(a))
    }

    fn set_value_dyn(&mut self, value: TO) {
        let map_mut = self.map_mut;
        *ValueRefMut::map(self.state.value_mut(), move |a| map_mut(a)) = value;
    }
}

impl<S: AnyState<T=FROM> + Clone + 'static, FROM: StateContract, TO: StateContract> Debug for FieldState<S, FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldState")
            .field("value", &*self.value())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use carbide_core::state::Map1;
    use crate::state::{FieldState, GlobalState, LocalState, ReadState, State};

    #[test]
    fn field_state1() {
        let mut base = LocalState::new((24, 42));

        let mut field = FieldState::new(base.clone(), |base| {&base.0}, |base| {&mut base.0});

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (24, 42));
        assert_eq!(*field.value(), 24);

        *field.value_mut() = 200;

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (200, 42));
        assert_eq!(*field.value(), 200);

        base.value_mut().0 = 32;

        assert_eq!(*base.value(), (32, 42));
        assert_eq!(*field.value(), 32);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        *base.value_mut() = (32, 33);

        assert_eq!(*base.value(), (32, 33));
        assert_eq!(*field.value(), 32);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        field.set_value(202);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (202, 33));
        assert_eq!(*field.value(), 202);

        base.set_value((42, 42));

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (42, 42));
        assert_eq!(*field.value(), 42);
    }

    #[test]
    #[should_panic(expected = "Already borrowed: ()")]
    fn field_state2() {
        let base = LocalState::new((24, 42));

        let mut field = FieldState::new(base.clone(), |base| {&base.0}, |base| {&mut base.0});

        assert_eq!(*base.value(), (24, 42));
        assert_eq!(*field.value(), 24);

        // Getting the value mut for the field in turn borrows the base value mutably.
        let mut val = field.value_mut();

        // This will panic since you are trying to borrow the base value immutable, but
        // it was already borrowed in "val".
        assert_eq!(*base.value(), (24, 42));

        // Makes sure the val is not dropped
        *val = 200;
    }

    #[test]
    fn field_state3() {
        let mut base = GlobalState::new((24, 42));

        let mut field = FieldState::new(base.clone(), |base| {&base.0}, |base| {&mut base.0});

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (24, 42));
        assert_eq!(*field.value(), 24);

        *field.value_mut() = 200;

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (200, 42));
        assert_eq!(*field.value(), 200);

        base.value_mut().0 = 32;

        assert_eq!(*base.value(), (32, 42));
        assert_eq!(*field.value(), 32);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        *base.value_mut() = (32, 33);

        assert_eq!(*base.value(), (32, 33));
        assert_eq!(*field.value(), 32);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        field.set_value(202);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (202, 33));
        assert_eq!(*field.value(), 202);

        base.set_value((42, 42));

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (42, 42));
        assert_eq!(*field.value(), 42);
    }

    #[test]
    fn field_state4() {
        let mut base = LocalState::new(((24, 35), 42));

        let mut field = FieldState::new(base.clone(), |base| {&base.0}, |base| {&mut base.0});
        let mut field2 = FieldState::new(field.clone(), |base| {&base.0}, |base| {&mut base.0});

        println!("Base: {:?}, Field: {:?}, Field2: {:?}", base.value(), field.value(), field2.value());

        assert_eq!(*base.value(), ((24, 35), 42));
        assert_eq!(*field.value(), (24, 35));
        assert_eq!(*field2.value(), 24);

        *field.value_mut() = (200, 430);

        println!("Base: {:?}, Field: {:?}, Field2: {:?}", base.value(), field.value(), field2.value());

        assert_eq!(*base.value(), ((200, 430), 42));
        assert_eq!(*field.value(), (200, 430));
        assert_eq!(*field2.value(), 200);

        base.value_mut().0 = (32, 33);

        assert_eq!(*base.value(), ((32, 33), 42));
        assert_eq!(*field.value(), (32, 33));
        assert_eq!(*field2.value(), 32);

        println!("Base: {:?}, Field: {:?}, Field2: {:?}", base.value(), field.value(), field2.value());

        *field2.value_mut() = 55;

        assert_eq!(*base.value(), ((55, 33), 42));
        assert_eq!(*field.value(), (55, 33));
        assert_eq!(*field2.value(), 55);

        println!("Base: {:?}, Field: {:?}, Field2: {:?}", base.value(), field.value(), field2.value());

        *base.value_mut() = ((32, 33), 34);

        assert_eq!(*base.value(), ((32, 33), 34));
        assert_eq!(*field.value(), (32, 33));
        assert_eq!(*field2.value(), 32);

        println!("Base: {:?}, Field: {:?}, Field2: {:?}", base.value(), field.value(), field2.value());
    }

    #[test]
    fn field_state5() {
        let mut base = LocalState::new(24);

        let mut mapped = Map1::map(
            base.clone(),
            |val| { *val + 2 },
            |new, old| { Some(new - 2) },
        );

        let mut field = FieldState::new(mapped.clone(), |base| { base }, |base| { base });

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 24);
        assert_eq!(*mapped.value(), 26);
        assert_eq!(*field.value(), 26);

        *base.value_mut() = 40;

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 40);
        assert_eq!(*mapped.value(), 42);
        assert_eq!(*field.value(), 42);

        *mapped.value_mut() = 20;

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 18);
        assert_eq!(*mapped.value(), 20);
        assert_eq!(*field.value(), 20);

        base.set_value(10);

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 10);
        assert_eq!(*mapped.value(), 12);
        assert_eq!(*field.value(), 12);

        mapped.set_value(10);

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 8);
        assert_eq!(*mapped.value(), 10);
        assert_eq!(*field.value(), 10);

        *field.value_mut() = 44;

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 42);
        assert_eq!(*mapped.value(), 44);
        assert_eq!(*field.value(), 44);

        field.set_value(420);

        println!("Base: {:?}, Mapped: {:?}, Field: {:?}", base.value(), mapped.value(), field.value());

        assert_eq!(*base.value(), 418);
        assert_eq!(*mapped.value(), 420);
        assert_eq!(*field.value(), 420);
    }

    #[test]
    fn field_state6() {
        let mut base = LocalState::new((24, 42));

        let mut field = lens!(base.0);
        let mut field2 = lens!(base.1);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (24, 42));
        assert_eq!(*field.value(), 24);
        assert_eq!(*field2.value(), 42);

        *field.value_mut() = 200;

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (200, 42));
        assert_eq!(*field.value(), 200);
        assert_eq!(*field2.value(), 42);

        base.value_mut().0 = 32;

        assert_eq!(*base.value(), (32, 42));
        assert_eq!(*field.value(), 32);
        assert_eq!(*field2.value(), 42);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        *base.value_mut() = (32, 33);

        assert_eq!(*base.value(), (32, 33));
        assert_eq!(*field.value(), 32);
        assert_eq!(*field2.value(), 33);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        field.set_value(202);

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (202, 33));
        assert_eq!(*field.value(), 202);
        assert_eq!(*field2.value(), 33);

        base.set_value((42, 42));

        println!("Base: {:?}, Field: {:?}", base.value(), field.value());

        assert_eq!(*base.value(), (42, 42));
        assert_eq!(*field.value(), 42);
        assert_eq!(*field2.value(), 42);
    }
}