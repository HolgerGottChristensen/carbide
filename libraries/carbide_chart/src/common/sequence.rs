//! The chart's own sequence trait. Points, datasets and controllers are all sequences of the
//! level below: a `DataSet` is a sequence of `DataPoint`s, a `DataSetSequence` a sequence of
//! `DataSet`s, and a `ControllerSequence` a sequence of `DatasetController`s.
//!
//! Unlike carbide's widget `Sequence`, reading one goes only through `&self`, handing each item
//! to a callback, and never asks for an item by index. That is what lets a state be a sequence:
//! a state lends its value only through a borrow guard that lives inside the call, so
//! `LocalState<Vec<Position>>` is a dataset, and `LocalState<Vec<LocalState<Vec<Position>>>>` a
//! sequence of datasets. Only `sync` reaches the items mutably, through the containers that can
//! lend them so (`Collection::visit_items_mut`, `CollectionState::with_collection_mut`).

use std::fmt::Debug;
use carbide::environment::Environment;
use crate::{Collection, DataSet, DatasetController, SequenceItem};

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


/// Implements `Sequence<$item>` for tuples of up to 12 items, and `IntoSequence<$item, One>` for
/// a single item, as a 1-tuple. Tuples are implemented per item kind, not for every `T`: a tuple
/// of points would make `(f64, f64)` both a point and a dataset.
macro_rules! tuple_sequences {
    ([$($generics:tt)*] $item:ty) => {
        impl<$($generics)* I: $crate::SequenceItem<$item>> $crate::IntoSequence<$item, $crate::One> for I {
            type Sequence = (I,);

            fn into_sequence(self) -> (I,) {
                (self,)
            }
        }

        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J) (9 K));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J) (9 K) (10 L));
        $crate::common::tuple_sequences!(@tuples [$($generics)*] $item; (0 A) (1 B) (2 C) (3 D) (4 E) (5 F) (6 G) (7 H) (8 J) (9 K) (10 L) (11 M));
    };
    (@tuples [$($generics:tt)*] $item:ty; $(($index:tt $name:ident))+) => {
        impl<$($generics)* $($name: $crate::SequenceItem<$item>),+> $crate::Sequence<$item> for ($($name,)+) {
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
                    changed |= <$name as $crate::SequenceItem<$item>>::sync(&mut self.$index, env);
                )+
                changed
            }
        }
    };
}

pub(crate) use tuple_sequences;

// Tuples of datasets and of controllers, each item of its own type.
tuple_sequences!([] dyn DataSet);
tuple_sequences!([] dyn DatasetController);
