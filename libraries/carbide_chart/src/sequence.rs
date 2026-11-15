//! The chart's own sequence trait. Points, datasets and controllers are all sequences of the
//! level below: a `DataSet` is a sequence of `DataPoint`s, a `DataSetSequence` a sequence of
//! `DataSet`s, and a `ControllerSequence` a sequence of `DatasetController`s.
//!
//! Unlike carbide's widget `Sequence`, this one only iterates, through `&self`, and never hands
//! out `&mut` to an item. That is what lets a state be a sequence: `LocalState<Vec<Position>>`
//! is a dataset, and `LocalState<Vec<LocalState<Vec<Position>>>>` a sequence of datasets.

use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
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

/// An ordered sequence of `T`s, visited in order with their index.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a sequence of `{T}`",
    label = "not a sequence of `{T}`",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
)]
pub trait Sequence<T: ?Sized>: Debug + Clone + 'static {
    fn foreach(&self, f: &mut dyn FnMut(usize, &T));

    /// Syncs the states in the sequence with the environment: the sequence itself when it is a
    /// state, and its items. Returns whether anything may have changed. The chart syncs its
    /// controllers before every update, render and event, as carbide syncs a widget's states, so
    /// reads afterwards see the current values without syncing.
    fn sync(&mut self, env: &mut Environment) -> bool;

    fn count(&self) -> usize {
        let mut count = 0;
        self.foreach(&mut |_, _| count += 1);
        count
    }
}

/// A value that can be visited as a `T`, at `index` in its sequence. Containers (collections,
/// states holding a collection, tuples) are sequences of anything that is an item, so they are
/// written once for points, datasets and controllers alike.
///
/// The index is passed in so an item can depend on its position: a plain `Scalar` is the point
/// `(index, value)`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be an item in a sequence of `{T}`",
    label = "not an item of a sequence of `{T}`",
    note = "the items are data points or plain `Scalar` y values in a dataset, datasets in a controller, and controllers in a chart",
)]
pub trait SequenceItem<T: ?Sized>: Debug + Clone + 'static {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &T));

    /// Syncs the states in the item with the environment, and returns whether anything may have
    /// changed. By default there is nothing to sync: a data point or a plain `Scalar` holds no
    /// states. Datasets and controllers sync theirs.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }
}

/// A container of items: what a sequence is made of.
///
/// Every collection is a `Sequence` of anything its items are (`SequenceItem`). Implemented for
/// `Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>` and `&'static [_]`, and for every
/// `CollectionState`, so `LocalState<VecDeque<Position>>` is a dataset and
/// `GlobalState<Rc<[Vec<Scalar>]>>` a sequence of datasets. Implementing it for a type of your own
/// makes that type, and every state holding it, a sequence.
///
/// The items are visited through a callback rather than returned as an iterator: a state can only
/// lend its value through a borrow guard that lives inside the call.
///
/// It is implemented once per collection type, not through a blanket implementation over
/// `IntoIterator`: a blanket over a trait from another crate would conflict with the tuple
/// implementations. As `Collection` belongs to this crate, the blanket implementations over it
/// don't conflict with anything.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a collection of items",
    label = "not a collection",
    note = "collections are `Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>` and `&'static [_]`, any state holding one (`CollectionState`), and any type implementing `Collection`",
)]
pub trait Collection: Debug + Clone + 'static {
    type Item;

    /// Calls `f` with each item and its index, in order.
    fn visit_items(&self, f: &mut dyn FnMut(usize, &Self::Item));

    fn len(&self) -> usize;

    /// Syncs the collection itself with the environment, and returns whether it may have
    /// changed. A plain container has nothing to sync; a state holding a collection syncs the
    /// state.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }

    /// Calls `f` with each item, mutably, so the sequence can sync the items. By default no items
    /// are visited: a container that shares its items (`Rc<[_]>`, `Arc<[_]>`, `&'static [_]`)
    /// or only lends them for reading can't hand out `&mut`, so items inside it aren't synced.
    fn visit_items_mut(&mut self, _f: &mut dyn FnMut(usize, &mut Self::Item)) {}
}

/// Implements `Collection` for a container whose items are visited through `.iter()`. With
/// `mut`, the container owns its items, so they are also visited mutably, through `.iter_mut()`,
/// for syncing.
macro_rules! collection {
    (@impl [$($generics:tt)*] $collection:ty { $($extra:tt)* }) => {
        impl<I: Debug + Clone + 'static, $($generics)*> Collection for $collection {
            type Item = I;

            fn visit_items(&self, f: &mut dyn FnMut(usize, &I)) {
                for (index, item) in self.iter().enumerate() {
                    f(index, item);
                }
            }

            fn len(&self) -> usize {
                <[I]>::len(&self[..])
            }

            $($extra)*
        }
    };
    (mut [$($generics:tt)*] $collection:ty) => {
        collection!(@impl [$($generics)*] $collection {
            fn visit_items_mut(&mut self, f: &mut dyn FnMut(usize, &mut I)) {
                for (index, item) in self.iter_mut().enumerate() {
                    f(index, item);
                }
            }
        });
    };
    ([$($generics:tt)*] $collection:ty) => {
        collection!(@impl [$($generics)*] $collection {});
    };
}

collection!(mut [] Vec<I>);
collection!(mut [const N: usize] [I; N]);
collection!(mut [] Box<[I]>);
// Shares its items without copying them: cloning an `Rc` only copies a pointer.
collection!([] Rc<[I]>);
// Shares its items without copying them, across threads.
collection!([] Arc<[I]>);
// Static data, borrowed rather than copied.
collection!([] &'static [I]);

impl<I: Debug + Clone + 'static> Collection for VecDeque<I> {
    type Item = I;

    fn visit_items(&self, f: &mut dyn FnMut(usize, &I)) {
        for (index, item) in self.iter().enumerate() {
            f(index, item);
        }
    }

    fn len(&self) -> usize {
        VecDeque::len(self)
    }

    fn visit_items_mut(&mut self, f: &mut dyn FnMut(usize, &mut I)) {
        for (index, item) in self.iter_mut().enumerate() {
            f(index, item);
        }
    }
}

/// Every collection is a sequence of anything its items are.
impl<T: ?Sized + 'static, C: Collection> Sequence<T> for C where C::Item: SequenceItem<T> {
    fn foreach(&self, f: &mut dyn FnMut(usize, &T)) {
        self.visit_items(&mut |index, item| item.visit(index, f));
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = Collection::sync(self, env);
        self.visit_items_mut(&mut |_, item| changed |= <C::Item as SequenceItem<T>>::sync(item, env));
        changed
    }

    fn count(&self) -> usize {
        self.len()
    }
}

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

/// A state holding a collection is a collection of the same items.
impl<S: CollectionState> Collection for S {
    type Item = <S::Value as Collection>::Item;

    fn visit_items(&self, f: &mut dyn FnMut(usize, &Self::Item)) {
        self.with_collection(&mut |collection| collection.visit_items(f));
    }

    fn len(&self) -> usize {
        let mut len = 0;
        self.with_collection(&mut |collection| len = collection.len());
        len
    }

    /// Syncs the state, then the collection it holds, in case that is a state too.
    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = CollectionState::sync(self, env);
        self.with_collection_mut(&mut |collection| changed |= Collection::sync(collection, env));
        changed
    }

    fn visit_items_mut(&mut self, f: &mut dyn FnMut(usize, &mut Self::Item)) {
        self.with_collection_mut(&mut |collection| collection.visit_items_mut(f));
    }
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

/// Marks the `IntoSequence` implementation for a single item, which becomes a sequence of one.
#[doc(hidden)]
pub struct One;

/// Marks the `IntoSequence` implementation for a value that is already a sequence.
#[doc(hidden)]
pub struct Many;

/// What a constructor takes where it wants a sequence: a sequence, or a single item, which
/// becomes a sequence of one. So `LineController::new(data)` and `LineController::new((a, b))`
/// both work.
///
/// A single item can't simply be a `Sequence` of one itself: a `Vec` of datasets would then be
/// both a sequence of its datasets and, being a dataset when its items are points, a sequence of
/// one. The marker `M` keeps the two implementations apart; it is always inferred.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is neither a `{T}` nor a sequence of them",
    label = "expected a `{T}`, or a sequence of them",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
    note = "a tuple can mix item types; the other sequences hold items of one type, so box controllers of different types with `.boxed()` to mix them",
)]
pub trait IntoSequence<T: ?Sized, M> {
    type Sequence: Sequence<T>;

    fn into_sequence(self) -> Self::Sequence;
}

impl<T: ?Sized + 'static, S: Sequence<T>> IntoSequence<T, Many> for S {
    type Sequence = S;

    fn into_sequence(self) -> S {
        self
    }
}

/// Implements `Sequence<$item>` for tuples of up to 12 items, and `IntoSequence<$item, One>` for
/// a single item, as a 1-tuple. Tuples are implemented per item kind, not for every `T`: a tuple
/// of points would make `(f64, f64)` both a point and a dataset.
macro_rules! tuple_sequences {
    ([$($generics:tt)*] $item:ty) => {
        impl<$($generics)* I: $crate::sequence::SequenceItem<$item>> $crate::sequence::IntoSequence<$item, $crate::sequence::One> for I {
            type Sequence = (I,);

            fn into_sequence(self) -> (I,) {
                (self,)
            }
        }

        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J) (9 K));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J) (9 K) (10 L));
        $crate::sequence::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J) (9 K) (10 L) (11 M));
    };
    (@tuples [$($generics:tt)*] $item:ty; $(($index:tt $name:ident))+) => {
        impl<$($generics)* $($name: $crate::sequence::SequenceItem<$item>),+> $crate::sequence::Sequence<$item> for ($($name,)+) {
            fn foreach(&self, f: &mut dyn FnMut(usize, &$item)) {
                $(
                    self.$index.visit($index, f);
                )+
            }

            fn count(&self) -> usize {
                [$($index),+].len()
            }

            fn sync(&mut self, env: &mut carbide::environment::Environment) -> bool {
                let mut changed = false;
                $(
                    changed |= <$name as $crate::sequence::SequenceItem<$item>>::sync(&mut self.$index, env);
                )+
                changed
            }
        }
    };
}

pub(crate) use tuple_sequences;

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::sync::Arc;
    use carbide::draw::{Position, Scalar};
    use carbide::environment::Environment;
    use carbide::state::{AnyState, FieldState, GlobalState, LocalState, Map1, ReadStateExtNew, State, StateExtNew, ValueState};
    use crate::{DataSet, DataSetSequence};

    /// The points of a dataset, as `(x, y)`.
    fn points<D: DataSet<Scalar, Scalar, Scalar>>(dataset: &D) -> Vec<(Scalar, Scalar)> {
        let mut points = vec![];
        dataset.points(&mut |_, point| points.push((point.x(), point.y())));
        points
    }

    fn dataset_count<S: DataSetSequence<Scalar, Scalar, Scalar>>(datasets: &S) -> usize {
        let mut count = 0;
        datasets.foreach(&mut |_, _| count += 1);
        assert_eq!(count, datasets.count(), "`count` should agree with `foreach`");
        count
    }

    const VALUES: [Scalar; 3] = [3.0, 5.0, 4.0];
    const INDEXED: [(Scalar, Scalar); 3] = [(0.0, 3.0), (1.0, 5.0), (2.0, 4.0)];
    static STATIC_VALUES: [Scalar; 3] = VALUES;

    /// Every collection is a dataset of points (plain values at their index), and a sequence of
    /// datasets.
    #[test]
    fn collections_are_sequences() {
        assert_eq!(points(&VALUES.to_vec()), INDEXED);
        assert_eq!(points(&VALUES), INDEXED);
        assert_eq!(points(&VecDeque::from(VALUES.to_vec())), INDEXED);
        assert_eq!(points(&Box::<[Scalar]>::from(VALUES)), INDEXED);
        assert_eq!(points(&Rc::<[Scalar]>::from(VALUES)), INDEXED);
        assert_eq!(points(&Arc::<[Scalar]>::from(VALUES)), INDEXED);
        assert_eq!(points(&&STATIC_VALUES[..]), INDEXED);

        let positions: Rc<[Position]> = Rc::from(vec![Position::new(1.0, 2.0)]);
        assert_eq!(points(&positions), vec![(1.0, 2.0)]);

        assert_eq!(dataset_count(&[VALUES.to_vec(), VALUES.to_vec()]), 2);
        assert_eq!(dataset_count(&VecDeque::from(vec![VALUES.to_vec()])), 1);
        assert_eq!(dataset_count(&Rc::<[Vec<Scalar>]>::from(vec![VALUES.to_vec(); 3])), 3);
    }

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

    /// A collection type of your own, implementing `Collection`, is a sequence, and so is every
    /// state holding it.
    #[test]
    fn custom_collections_are_sequences() {
        #[derive(Clone, Debug)]
        struct Readings { values: Vec<Scalar> }

        impl super::Collection for Readings {
            type Item = Scalar;

            fn visit_items(&self, f: &mut dyn FnMut(usize, &Scalar)) {
                for (index, value) in self.values.iter().enumerate() {
                    f(index, value);
                }
            }

            fn len(&self) -> usize {
                self.values.len()
            }
        }

        let readings = Readings { values: VALUES.to_vec() };
        assert_eq!(points(&readings), INDEXED);

        let mut state = LocalState::new(readings);
        assert_eq!(points(&state), INDEXED);
        *state.value_mut() = Readings { values: vec![1.0] };
        assert_eq!(points(&state), vec![(0.0, 1.0)]);
    }

    /// A state type of your own, implementing `CollectionState` generically over the collection,
    /// works with every collection: the standard ones, one of your own, and collections of
    /// datasets. States of states work too.
    #[test]
    fn custom_states_are_sequences() {
        use std::cell::RefCell;
        use super::{Collection, CollectionState};

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
