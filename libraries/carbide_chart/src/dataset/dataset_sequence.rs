use crate::{DataSet, DataValue};
use crate::sequence::{tuple_sequences, Sequence};

/// The datasets of one controller: any `Sequence` of datasets that share their point types.
///
/// Implemented for, with `D` any dataset that is `Clone` (see `sequence.rs`):
/// - tuples `(D1,)` to `(D1, …, D12)`, each of its own dataset type;
/// - collections (`Collection`): `Vec<D>`, `[D; N]`, `VecDeque<D>`, `Box<[D]>`, `Rc<[D]>`,
///   `Arc<[D]>`, `&'static [D]`, and any type of your own implementing `Collection`;
/// - states holding any of those collections: `LocalState`, `GlobalState`, `ValueState`,
///   `FieldState`, carbide's mapped states (`read_map`, `read_map_env`, `map`, `map_owned`), and
///   `Box<dyn AnyReadState>` / `Box<dyn AnyState>`. Any other state is a sequence once boxed with
///   `.as_dyn_read()`.
///
/// Every item of a collection or state has the same type. To mix dataset types there, box them
/// with `DatasetExt::boxed`: `vec![points.boxed(), values.boxed()]`. The point types stay part
/// of the boxed type, so the datasets must still share `X`, `Y` and `Z`.
///
/// With a `Vec` or a state, the number of datasets can change at runtime:
/// `LocalState<Vec<LocalState<Vec<Position>>>>` is a sequence of datasets. A type of your own
/// becomes one by implementing `Sequence<dyn DataSet<X, Y, Z>>`.
///
/// A single dataset is not a `DataSetSequence`, but controllers take an `IntoSequence`, which
/// turns a single dataset into the 1-tuple `(D,)`, so it can be passed on its own.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a sequence of datasets",
    label = "not a sequence of datasets",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
    note = "a single dataset is accepted where a constructor takes an `IntoSequence`, but is not a `DataSetSequence` itself",
)]
pub trait DataSetSequence<X: DataValue, Y: DataValue, Z: DataValue>: Sequence<dyn DataSet<X, Y, Z>> {}

impl<X: DataValue, Y: DataValue, Z: DataValue, S: Sequence<dyn DataSet<X, Y, Z>>> DataSetSequence<X, Y, Z> for S {}

tuple_sequences!([X: DataValue, Y: DataValue, Z: DataValue,] dyn DataSet<X, Y, Z>);
