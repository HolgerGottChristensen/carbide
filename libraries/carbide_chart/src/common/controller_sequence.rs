use crate::{DatasetController, Sequence};

/// The controllers of a chart: any `Sequence` of controllers. They are visited in sequence
/// order, which is also the order they are drawn in.
///
/// Implemented for, with `C` any controller that is `Clone` (see `Sequence`):
/// - tuples `(C1,)` to `(C1, …, C12)`, each of its own controller type;
/// - collections (`Collection`): `Vec<C>`, `[C; N]`, `VecDeque<C>`, `Box<[C]>`, `Rc<[C]>`,
///   `Arc<[C]>`, `&'static [C]`, and any type of your own implementing `Collection`;
/// - states holding any of those collections, as for `DataSetSequence`.
///
/// Every item of a collection or state has the same type. To mix controller types there, box
/// them with `DatasetController::boxed`: `vec![bars.boxed(), lines.boxed()]`.
///
/// A type of your own becomes one by implementing `Sequence<dyn DatasetController>`. A single
/// controller is not a `ControllerSequence`, but `Chart::new` takes an `IntoSequence`, which turns
/// it into the 1-tuple `(C,)`, so it can be passed on its own.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a sequence of controllers",
    label = "not a sequence of controllers",
    note = "sequences are tuples of up to 12 items; collections (`Vec`, arrays, `VecDeque`, `Box<[_]>`, `Rc<[_]>`, `Arc<[_]>`, `&'static [_]`, or any type implementing `Collection`); and states holding a collection: `LocalState`, `GlobalState`, `ValueState`, `FieldState`, mapped states, or any read state boxed with `.as_dyn_read()`",
    note = "a single controller is accepted by `Chart::new`, but is not a `ControllerSequence` itself",
)]
pub trait ControllerSequence: Sequence<dyn DatasetController> {}

impl<S: Sequence<dyn DatasetController>> ControllerSequence for S {}
