use crate::{Many, Sequence};

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
    note = "a tuple can mix item types; the other sequences hold items of one type, so box items of different types with `.boxed()` to mix them",
    note = "a dataset with a mark is a `dyn BarDataSet`, `dyn LineDataSet` or `dyn ScatterDataSet` only when the mark can be drawn by that kind of controller: `.range()` and `.error_bars()` are drawn by bar and line controllers, `.box_plot()`, `.candles()` and `.ohlc()` by bar controllers, and `.bubbles()` by scatter controllers",
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
