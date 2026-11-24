use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use carbide::environment::Environment;
use crate::CollectionState;

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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::sync::Arc;
    use carbide::draw::{Position, Scalar};
    use carbide::state::{LocalState, State};
    use crate::{DataSet, Sequence, Value};

    /// The points of a dataset, as `(x, y)`. None of the datasets here have missing points.
    fn points<D: DataSet>(dataset: &D) -> Vec<(Scalar, Scalar)> {
        let mut points = vec![];
        dataset.points(&mut |_, point| points.push((point.x().and_then(Value::number).expect("no missing x"), point.y().and_then(Value::number).expect("no missing y"))));
        points
    }

    fn dataset_count<S: Sequence<dyn DataSet>>(datasets: &S) -> usize {
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

    /// A collection type of your own, implementing `Collection`, is a sequence, and so is every
    /// state holding it.
    #[test]
    fn custom_collections_are_sequences() {
        #[derive(Clone, Debug)]
        struct Readings { values: Vec<Scalar> }

        impl crate::Collection for Readings {
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
}
