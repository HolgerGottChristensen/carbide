use crate::controller::DatasetController;
use crate::sequence::{tuple_sequences, Sequence, SequenceItem};

/// The controllers of a chart: any `Sequence` of controllers. They are visited in sequence
/// order, which is also the order they are drawn in.
///
/// Implemented for, with `C` any controller that is `Clone`:
/// - tuples `(C1,)` to `(C1, …, C12)`, each of its own controller type;
/// - `Vec<C>`;
/// - `LocalState<Vec<C>>`;
/// - `Box<dyn AnyReadState<T=Vec<C>>>`. Any other state of a `Vec` of controllers is a sequence
///   once boxed with `.as_dyn_read()`.
///
/// A type of your own becomes one by implementing `Sequence<dyn DatasetController>`. A single
/// controller is not a `ControllerSequence`, but `Chart::new` takes an `IntoSequence`, which turns
/// it into the 1-tuple `(C,)`, so it can be passed on its own.
pub trait ControllerSequence: Sequence<dyn DatasetController> {}

impl<S: Sequence<dyn DatasetController>> ControllerSequence for S {}

/// A controller is an item of a sequence of controllers.
impl<C: DatasetController + Clone> SequenceItem<dyn DatasetController> for C {
    fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn DatasetController + 'static))) {
        f(index, self);
    }
}

tuple_sequences!([] dyn DatasetController);
