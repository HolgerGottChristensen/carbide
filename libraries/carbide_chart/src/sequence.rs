//! The chart's own sequence trait. Points, datasets and controllers are all sequences of the
//! level below: a `DataSet` is a sequence of `DataPoint`s, a `DataSetSequence` a sequence of
//! `DataSet`s, and a `ControllerSequence` a sequence of `DatasetController`s.
//!
//! Unlike carbide's widget `Sequence`, this one only iterates, through `&self`, and never hands
//! out `&mut` to an item. That is what lets a state be a sequence: `LocalState<Vec<Position>>`
//! is a dataset, and `LocalState<Vec<LocalState<Vec<Position>>>>` a sequence of datasets.

use std::fmt::Debug;
use carbide::state::{AnyReadState, LocalState, ReadState};

/// An ordered sequence of `T`s, visited in order with their index.
pub trait Sequence<T: ?Sized>: Debug + Clone + 'static {
    fn foreach(&self, f: &mut dyn FnMut(usize, &T));

    fn count(&self) -> usize {
        let mut count = 0;
        self.foreach(&mut |_, _| count += 1);
        count
    }
}

/// A value that can be visited as a `T`, at `index` in its sequence. Containers (`Vec`, states
/// of a `Vec`, tuples) are sequences of anything that is an item, so they are written once for
/// points, datasets and controllers alike.
///
/// The index is passed in so an item can depend on its position: a plain `Scalar` is the point
/// `(index, value)`.
pub trait SequenceItem<T: ?Sized>: Debug + Clone + 'static {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &T));
}

impl<T: ?Sized + 'static, I: SequenceItem<T>> Sequence<T> for Vec<I> {
    fn foreach(&self, f: &mut dyn FnMut(usize, &T)) {
        for (index, item) in self.iter().enumerate() {
            item.visit(index, f);
        }
    }

    fn count(&self) -> usize {
        self.len()
    }
}

impl<T: ?Sized + 'static, I: SequenceItem<T>> Sequence<T> for LocalState<Vec<I>> {
    fn foreach(&self, f: &mut dyn FnMut(usize, &T)) {
        for (index, item) in self.value().iter().enumerate() {
            item.visit(index, f);
        }
    }

    fn count(&self) -> usize {
        self.value().len()
    }
}

/// Any other read state, such as a mapped one, boxed with `.as_dyn_read()`.
impl<T: ?Sized + 'static, I: SequenceItem<T>> Sequence<T> for Box<dyn AnyReadState<T=Vec<I>>> {
    fn foreach(&self, f: &mut dyn FnMut(usize, &T)) {
        for (index, item) in self.value().iter().enumerate() {
            item.visit(index, f);
        }
    }

    fn count(&self) -> usize {
        self.value().len()
    }
}

/// Marks the `IntoSequence` implementation for a single item, which becomes a sequence of one.
pub struct One;

/// Marks the `IntoSequence` implementation for a value that is already a sequence.
pub struct Many;

/// What a constructor takes where it wants a sequence: a sequence, or a single item, which
/// becomes a sequence of one. So `LineController::new(data)` and `LineController::new((a, b))`
/// both work.
///
/// A single item can't simply be a `Sequence` of one itself: a `Vec` of datasets would then be
/// both a sequence of its datasets and, being a dataset when its items are points, a sequence of
/// one. The marker `M` keeps the two implementations apart, as Bevy's `IntoSystem<Marker>`; it is
/// always inferred.
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
        }
    };
}

pub(crate) use tuple_sequences;
