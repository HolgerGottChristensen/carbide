//! The chart's own sequence trait. Points, datasets and controllers are all sequences of the
//! level below: a `DataSet` is a sequence of `DataPoint`s, a controller holds a
//! `Sequence<dyn DataSet>` (or of its family, such as `dyn BarDataSet`), and a chart a
//! `Sequence<dyn DatasetController>`.
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

/// An ordered sequence of `T`s, visited in order with their index, such as a dataset's points, a
/// controller's datasets or a chart's controllers.
///
/// Implemented for, with each item anything that is a `T` (see `SequenceItem`):
/// - tuples of 1 to 12 items, each of its own type;
/// - collections (`Collection`): `Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`,
///   `&'static [_]`, and any type of your own implementing `Collection`;
/// - states holding any of those collections (`CollectionState`): `LocalState`, `GlobalState`,
///   `ValueState`, `FieldState`, carbide's mapped states (`read_map`, `read_map_env`, `map`,
///   `map_owned`), its cached states (`CachedReadState`, `CachedState`), and
///   `Box<dyn AnyReadState>` / `Box<dyn AnyState>`. Any other carbide state is a sequence once
///   boxed with `.as_dyn_read()`, and a state type of your own once it implements
///   `CollectionState`.
///
/// Every item of a collection or state has the same type, while each item of a tuple has its own.
/// With a `Vec` or a state, the number of items can change at runtime.
///
/// A single item is not a sequence of one; where a single item should do, take an `IntoSequence`,
/// which turns it into the 1-tuple `(item,)`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a sequence of `{T}`",
    label = "not a sequence of `{T}`",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
)]
pub trait Sequence<T: ?Sized>: Debug + Clone + 'static {
    /// Internal iteration of each item and index in the sequence
    fn foreach(&self, f: &mut dyn FnMut(usize, &T));

    /// Syncs the states in the sequence with the environment: the sequence itself when it is a
    /// state, and its items. Returns whether anything may have changed. The chart syncs its
    /// controllers before every update, render and event, as carbide syncs a widget's states, so
    /// reads afterwards see the current values without syncing.
    fn sync(&mut self, env: &mut Environment) -> bool;

    /// The total number of items in the sequence. By default it iterates and counts.
    /// Overridden for collections that provide faster count/len methods.
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
