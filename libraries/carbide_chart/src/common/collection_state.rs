use std::fmt::Debug;
use carbide::cell::ValueRefMut;
use carbide::environment::Environment;
use carbide::state::StateSync;
use carbide::state::{
    AnyReadState, AnyState, CachedReadState, CachedState, EnvMap1, EnvMap10, EnvMap11, EnvMap12, EnvMap2, EnvMap3, EnvMap4, EnvMap5,
    EnvMap6, EnvMap7, EnvMap8, EnvMap9, FieldState, GlobalState, LocalState, RMap1, RMap10, RMap11,
    RMap12, RMap2, RMap3, RMap4, RMap5, RMap6, RMap7, RMap8, RMap9, RWMap1, RWMap10, RWMap11, RWMap12,
    RWMap2, RWMap3, RWMap4, RWMap5, RWMap6, RWMap7, RWMap8, RWMap9, RWOMap1, RWOMap10, RWOMap11,
    RWOMap12, RWOMap2, RWOMap3, RWOMap4, RWOMap5, RWOMap6, RWOMap7, RWOMap8, RWOMap9, State,
    ReadState, StateContract, ValueState,
};

use crate::Collection;

/// A state holding a collection. It is a collection of the same items, read through its current
/// value, so the items and their number can change after the chart is built.
///
/// Implemented for carbide's states holding any collection: `LocalState`, `GlobalState`,
/// `ValueState`, `FieldState`, the mapped states (`read_map`, `read_map_env`, `map`, `map_owned`),
/// the cached states (`CachedReadState`, `CachedState`) and `Box<dyn AnyReadState>` /
/// `Box<dyn AnyState>`, through which any other carbide state fits
/// with `.as_dyn_read()`. Implementing it for a state type of your own makes that state a
/// collection, and so a sequence.
///
/// It is implemented once per state type. A blanket implementation over carbide's `ReadState` is
/// not possible: a plain `Vec` is a constant `ReadState` in carbide, so it would be both a state
/// and a collection, and the compiler must assume carbide could make tuples states one day (see
/// "Sequences for every state type" in `docs/multiple-controllers-and-scales.md`).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a state holding a collection",
    label = "not a state of a collection",
    note = "states are `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, and read states boxed with `.as_dyn_read()`, each holding a collection",
)]
pub trait CollectionState: Debug + Clone + 'static {
    type Value: Collection;

    /// Calls `f` with the collection the state holds now. A callback rather than a returned
    /// reference, so any storage works: a carbide state lends its value through a `ValueRef`, a
    /// `RefCell` through a `Ref`, a `Mutex` through a guard.
    fn with_collection(&self, f: &mut dyn FnMut(&Self::Value));

    /// Syncs the state with the environment, and returns whether it may have changed. By default
    /// there's nothing to sync, and the answer is `true`: without a way to tell, the state may
    /// always have changed.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        true
    }

    /// Calls `f` with the collection, mutably, so the items in it can be synced. By default it
    /// isn't called: a state that only lends its value for reading keeps its items unsynced.
    fn with_collection_mut(&mut self, _f: &mut dyn FnMut(&mut Self::Value)) {}
}


/// Implements `CollectionState` for a carbide state holding any collection `C`: read through
/// `AnyReadState::value_dyn`, synced through `StateSync::sync`. With `mut`, the state stores its
/// value plainly and lends it mutably through `AnyState::value_dyn_mut`, so the items in it are
/// synced too.
///
/// carbide's read-write maps (`map`, `map_owned`) and `Box<dyn AnyState>`, which could hold one,
/// are not marked `mut`: borrowing a read-write map's value mutably writes it back into the
/// states it maps from when the borrow ends, which syncing must not do.
macro_rules! collection_state {
    (@impl [$($generics:tt)*] $state:ty [$($bounds:tt)*] { $($extra:tt)* }) => {
        impl<C: Collection, $($generics)*> CollectionState for $state where $($bounds)* {
            type Value = C;

            fn with_collection(&self, f: &mut dyn FnMut(&C)) {
                f(&self.value_dyn());
            }

            fn sync(&mut self, env: &mut Environment) -> bool {
                StateSync::sync(self, env)
            }

            $($extra)*
        }
    };
    (mut [$($generics:tt)*] $state:ty $(where $($bounds:tt)*)?) => {
        collection_state!(@impl [$($generics)*] $state [$($($bounds)*)?] {
            fn with_collection_mut(&mut self, f: &mut dyn FnMut(&mut C)) {
                f(&mut self.value_dyn_mut());
            }
        });
    };
    ([$($generics:tt)*] $state:ty $(where $($bounds:tt)*)?) => {
        collection_state!(@impl [$($generics)*] $state [$($($bounds)*)?] {});
    };
}

collection_state!(mut [] LocalState<C>);
collection_state!(mut [] GlobalState<C>);
collection_state!(mut [] ValueState<C>);
collection_state!(mut [S: State<T=FROM>, FROM: StateContract] FieldState<S, FROM, C>);
// Any read state, boxed with `.as_dyn_read()`: the way in for a carbide state not listed here.
collection_state!([] Box<dyn AnyReadState<T=C>>);
collection_state!([] Box<dyn AnyState<T=C>>);
// Cached states compute their value when synced and read the cached result afterwards, so a
// mapping wrapped in one computes once per sync instead of on every read.
collection_state!([S: ReadState<T=C>] CachedReadState<C, S>);
collection_state!([S: State<T=C>] CachedState<C, S>);

/// Implements `CollectionState` for carbide's mapped states of one arity, mapping to any
/// collection `C`: read maps (`Map1::read_map`), environment maps (`read_map_env`) and
/// read-write maps (`map`, `map_owned`). The bounds repeat the ones on carbide's structs.
macro_rules! map_states {
    ($read_map:ident, $env_map:ident, $map:ident, $owned_map:ident; $($value:ident $source:ident),+) => {
        collection_state!(
            [MAP, $($value: StateContract, $source: AnyReadState<T=$value> + Clone + 'static),+]
            $read_map<MAP, $($value),+, C, $($source),+>
            where MAP: Fn($(&$value),+) -> C + Clone + 'static
        );
        collection_state!(
            [MAP, $($value: StateContract, $source: AnyReadState<T=$value> + Clone + 'static),+]
            $env_map<MAP, $($value),+, C, $($source),+>
            where
                C: Default,
                MAP: Fn(&mut Environment, $(&$value),+) -> C + Clone + 'static
        );
        collection_state!(
            [MAP, REPLACE, $($value: StateContract, $source: AnyState<T=$value> + Clone + 'static),+]
            $map<MAP, REPLACE, $($value),+, C, $($source),+>
            where
                MAP: Fn($(&$value),+) -> C + Clone + 'static,
                REPLACE: Fn(C, $(ValueRefMut<$value>),+) + Clone + 'static
        );
        collection_state!(
            [MAP, REPLACE, $($value: StateContract, $source: AnyState<T=$value> + Clone + 'static),+]
            $owned_map<MAP, REPLACE, $($value),+, C, $($source),+>
            where
                MAP: Fn($(&$value),+, &mut C) + Clone + 'static,
                REPLACE: Fn(&C, $(ValueRefMut<$value>),+) + Clone + 'static
        );
    };
}

map_states!(RMap1, EnvMap1, RWMap1, RWOMap1; T1 S1);
map_states!(RMap2, EnvMap2, RWMap2, RWOMap2; T1 S1, T2 S2);
map_states!(RMap3, EnvMap3, RWMap3, RWOMap3; T1 S1, T2 S2, T3 S3);
map_states!(RMap4, EnvMap4, RWMap4, RWOMap4; T1 S1, T2 S2, T3 S3, T4 S4);
map_states!(RMap5, EnvMap5, RWMap5, RWOMap5; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5);
map_states!(RMap6, EnvMap6, RWMap6, RWOMap6; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6);
map_states!(RMap7, EnvMap7, RWMap7, RWOMap7; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6, T7 S7);
map_states!(RMap8, EnvMap8, RWMap8, RWOMap8; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6, T7 S7, T8 S8);
map_states!(RMap9, EnvMap9, RWMap9, RWOMap9; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6, T7 S7, T8 S8, T9 S9);
map_states!(RMap10, EnvMap10, RWMap10, RWOMap10; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6, T7 S7, T8 S8, T9 S9, T10 S10);
map_states!(RMap11, EnvMap11, RWMap11, RWOMap11; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6, T7 S7, T8 S8, T9 S9, T10 S10, T11 S11);
map_states!(RMap12, EnvMap12, RWMap12, RWOMap12; T1 S1, T2 S2, T3 S3, T4 S4, T5 S5, T6 S6, T7 S7, T8 S8, T9 S9, T10 S10, T11 S11, T12 S12);

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::sync::Arc;
    use carbide::draw::{Position, Scalar};
    use carbide::environment::Environment;
    use carbide::state::{AnyState, FieldState, GlobalState, LocalState, Map1, ReadStateExt, State, StateExtNew, ValueState};
    use crate::{DataSet, DataSetSequence, Value};

    /// The points of a dataset, as `(x, y)`. None of the datasets here have missing points.
    fn points<D: DataSet>(dataset: &D) -> Vec<(Scalar, Scalar)> {
        let mut points = vec![];
        dataset.points(&mut |_, point| points.push((point.x().and_then(Value::number).expect("no missing x"), point.y().and_then(Value::number).expect("no missing y"))));
        points
    }

    fn dataset_count<S: DataSetSequence>(datasets: &S) -> usize {
        let mut count = 0;
        datasets.foreach(&mut |_, _| count += 1);
        assert_eq!(count, datasets.count(), "`count` should agree with `foreach`");
        count
    }

    const VALUES: [Scalar; 3] = [3.0, 5.0, 4.0];
    const INDEXED: [(Scalar, Scalar); 3] = [(0.0, 3.0), (1.0, 5.0), (2.0, 4.0)];
    static STATIC_VALUES: [Scalar; 3] = VALUES;

    /// Every state of a `Vec` is a dataset, read through its current value, so a change to the
    /// state shows on the next read.
    #[test]
    fn states_are_sequences() {
        let mut local = LocalState::new(VALUES.to_vec());
        assert_eq!(points(&local), INDEXED);
        *local.value_mut() = vec![1.0];
        assert_eq!(points(&local), vec![(0.0, 1.0)]);

        let mut global = GlobalState::new(VALUES.to_vec());
        assert_eq!(points(&global), INDEXED);
        *global.value_mut() = vec![1.0];
        assert_eq!(points(&global), vec![(0.0, 1.0)]);

        assert_eq!(points(&ValueState::new(VALUES.to_vec())), INDEXED);

        #[derive(Clone, Debug)]
        struct Model { readings: Vec<Scalar> }
        let model = LocalState::new(Model { readings: VALUES.to_vec() });
        let field = FieldState::new(model.clone(), |model: &Model| &model.readings, |model: &mut Model| &mut model.readings);
        assert_eq!(points(&field), INDEXED);

        let boxed_read = local.clone().as_dyn_read();
        assert_eq!(points(&boxed_read), vec![(0.0, 1.0)]);
        let boxed: Box<dyn AnyState<T=Vec<Scalar>>> = local.as_dyn();
        assert_eq!(points(&boxed), vec![(0.0, 1.0)]);

        let datasets = GlobalState::new(vec![VALUES.to_vec(), VALUES.to_vec()]);
        assert_eq!(dataset_count(&datasets), 2);
    }

    /// Every state holds any collection, so every combination is a sequence.
    #[test]
    fn states_of_every_collection_are_sequences() {
        let mut deque = LocalState::new(VecDeque::from(VALUES.to_vec()));
        assert_eq!(points(&deque), INDEXED);
        *deque.value_mut() = VecDeque::from(vec![1.0]);
        assert_eq!(points(&deque), vec![(0.0, 1.0)]);

        assert_eq!(points(&LocalState::new(VALUES)), INDEXED);
        assert_eq!(points(&GlobalState::new(Rc::<[Scalar]>::from(VALUES))), INDEXED);
        assert_eq!(points(&ValueState::new(Arc::<[Scalar]>::from(VALUES))), INDEXED);
        assert_eq!(points(&LocalState::new(Box::<[Scalar]>::from(VALUES))), INDEXED);
        assert_eq!(points(&LocalState::new(&STATIC_VALUES[..])), INDEXED);

        let model = LocalState::new(VALUES.to_vec());
        let shared = Map1::read_map(model.clone(), |values: &Vec<Scalar>| Rc::<[Scalar]>::from(values.as_slice()));
        assert_eq!(points(&shared), INDEXED, "a mapped state can map to any collection");

        let datasets = LocalState::new(VecDeque::from(vec![VALUES.to_vec(), VALUES.to_vec()]));
        assert_eq!(dataset_count(&datasets), 2);
    }

    /// A state type of your own, implementing `CollectionState` generically over the collection,
    /// works with every collection: the standard ones, one of your own, and collections of
    /// datasets. States of states work too.
    #[test]
    fn custom_states_are_sequences() {
        use std::cell::RefCell;
        use crate::{Collection, CollectionState};

        /// A shared, swappable value, standing in for a user's own state type. It isn't a carbide
        /// state: it lends its value through a `RefCell` borrow.
        #[derive(Clone, Debug)]
        struct Shared<C>(Rc<RefCell<C>>);

        impl<C> Shared<C> {
            fn new(value: C) -> Shared<C> {
                Shared(Rc::new(RefCell::new(value)))
            }

            fn set(&self, value: C) {
                *self.0.borrow_mut() = value;
            }
        }

        impl<C: Collection> CollectionState for Shared<C> {
            type Value = C;

            fn with_collection(&self, f: &mut dyn FnMut(&C)) {
                f(&self.0.borrow());
            }
        }

        /// A collection type of your own, used inside the state of your own.
        #[derive(Clone, Debug)]
        struct Readings(Vec<Scalar>);

        impl Collection for Readings {
            type Item = Scalar;

            fn visit_items(&self, f: &mut dyn FnMut(usize, &Scalar)) {
                for (index, value) in self.0.iter().enumerate() {
                    f(index, value);
                }
            }

            fn len(&self) -> usize {
                self.0.len()
            }
        }

        let vec = Shared::new(VALUES.to_vec());
        assert_eq!(points(&vec), INDEXED);
        vec.set(vec![1.0]);
        assert_eq!(points(&vec), vec![(0.0, 1.0)], "the state should be read through its current value");

        assert_eq!(points(&Shared::new(VecDeque::from(VALUES.to_vec()))), INDEXED);
        assert_eq!(points(&Shared::new(Rc::<[Scalar]>::from(VALUES))), INDEXED);
        assert_eq!(points(&Shared::new(VALUES)), INDEXED);
        assert_eq!(points(&Shared::new(vec![Position::new(1.0, 2.0)])), vec![(1.0, 2.0)]);

        let readings = Shared::new(Readings(VALUES.to_vec()));
        assert_eq!(points(&readings), INDEXED);
        readings.set(Readings(vec![1.0]));
        assert_eq!(points(&readings), vec![(0.0, 1.0)]);

        let datasets = Shared::new(VecDeque::from(vec![VALUES.to_vec(), VALUES.to_vec()]));
        assert_eq!(dataset_count(&datasets), 2);
        datasets.set(VecDeque::from(vec![VALUES.to_vec()]));
        assert_eq!(dataset_count(&datasets), 1, "datasets can be removed through the state");

        let nested = LocalState::new(LocalState::new(VALUES.to_vec()));
        assert_eq!(points(&nested), INDEXED);
    }

    /// A mapping wrapped in a cached state computes once per sync, however often the dataset is
    /// read afterwards. Unwrapped, it computes on every read.
    #[test]
    fn cached_states_compute_once_per_sync() {
        use std::cell::Cell;
        use carbide::state::CachedReadState;

        let model = LocalState::new(VALUES.to_vec());
        let computed = Rc::new(Cell::new(0));
        let counter = computed.clone();
        let mapped = Map1::read_map(model, move |values: &Vec<Scalar>| {
            counter.set(counter.get() + 1);
            values.clone()
        });

        let mut cached = CachedReadState::new(mapped.clone());
        DataSet::sync(&mut cached, &mut Environment::new());
        computed.set(0);

        assert_eq!(points(&cached), INDEXED);
        assert!(cached.min().is_some() && cached.max().is_some());
        assert_eq!(computed.get(), 0, "reads of a synced cached state shouldn't recompute");

        DataSet::sync(&mut cached, &mut Environment::new());
        assert_eq!(computed.get(), 1, "a sync should compute the mapping once");

        computed.set(0);
        let _ = (points(&mapped), mapped.min(), mapped.max());
        assert_eq!(computed.get(), 3, "unwrapped, every read recomputes");
    }

    /// carbide's mapped states are datasets. Read and read-write maps compute their value when
    /// read; an environment map only when synced, which the dataset's `sync` does.
    #[test]
    fn mapped_states_are_sequences() {
        let model = LocalState::new(VALUES.to_vec());
        let double = |values: &Vec<Scalar>| values.iter().map(|value| value * 2.0).collect::<Vec<Scalar>>();
        let doubled = vec![(0.0, 6.0), (1.0, 10.0), (2.0, 8.0)];

        assert_eq!(points(&Map1::read_map(model.clone(), double)), doubled);

        let mapped = Map1::map(model.clone(), double, |_doubled: Vec<Scalar>, _values| {});
        assert_eq!(points(&mapped), doubled);

        let owned = Map1::map_owned(model.clone(), move |values: &Vec<Scalar>, out: &mut Vec<Scalar>| *out = double(values), |_: &Vec<Scalar>, _values| {}, vec![]);
        assert_eq!(points(&owned), doubled);

        let mut env_mapped = Map1::read_map_env(model.clone(), move |_env: &mut Environment, values: &Vec<Scalar>| double(values));
        assert_eq!(points(&env_mapped), vec![], "an environment map has no value before it is synced");
        assert!(DataSet::sync(&mut env_mapped, &mut Environment::new()), "syncing a state reports that it may have changed");
        assert_eq!(points(&env_mapped), doubled, "the dataset's sync syncs the environment map");

        let two_models = LocalState::new(vec![1.0]);
        let joined = carbide::state::Map2::read_map(model, two_models, |a: &Vec<Scalar>, b: &Vec<Scalar>| [a.clone(), b.clone()].concat());
        assert_eq!(points(&joined).len(), 4, "maps of several states are sequences too");
    }
}
