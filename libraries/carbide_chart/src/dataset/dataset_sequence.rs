use crate::{DataSet, DataValue};
use crate::sequence::{tuple_sequences, Sequence};

/// The datasets of one controller: any `Sequence` of datasets that share their point types.
///
/// Implemented for, with `D` any dataset that is `Clone`:
/// - tuples `(D1,)` to `(D1, …, D12)`, each of its own dataset type;
/// - `Vec<D>`;
/// - `LocalState<Vec<D>>`;
/// - `Box<dyn AnyReadState<T=Vec<D>>>`. Any other state of a `Vec` of datasets (a mapped state,
///   `GlobalState`, `FieldState`, `ValueState`) is a sequence once boxed with `.as_dyn_read()`.
///
/// With a `Vec` or a state, the number of datasets can change at runtime:
/// `LocalState<Vec<LocalState<Vec<Position>>>>` is a sequence of datasets. A type of your own
/// becomes one by implementing `Sequence<dyn DataSet<X, Y, Z>>`.
///
/// A single dataset is not a `DataSetSequence`, but controllers take an `IntoSequence`, which
/// turns a single dataset into the 1-tuple `(D,)`, so it can be passed on its own.
pub trait DataSetSequence<X: DataValue, Y: DataValue, Z: DataValue>: Sequence<dyn DataSet<X, Y, Z>> {}

impl<X: DataValue, Y: DataValue, Z: DataValue, S: Sequence<dyn DataSet<X, Y, Z>>> DataSetSequence<X, Y, Z> for S {}

tuple_sequences!([X: DataValue, Y: DataValue, Z: DataValue,] dyn DataSet<X, Y, Z>);
