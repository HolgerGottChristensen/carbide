use carbide::draw::Scalar;
use crate::dataset::SizedPoint;
use crate::mark::{Mark, MarkRole, Primary, Role};
use crate::sequence::Sequence;

/// A mark that can be drawn by a pie controller, for datasets of type `D`: implementing it is what
/// lets a dataset with this mark be drawn by a `PieController` (see `PieDataSet`). A pie's parts
/// are shares of a whole, so its marks say something about each part besides its share, such as
/// how far out its slice reaches (`RadiiMark`).
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a pie controller for `{D}`",
    label = "not a pie mark for this dataset",
    note = "pie marks are `RadiiMark` (`.radii()`) on datasets of `SizedPoint`s, and types of your own implementing `PieMark`",
)]
pub trait PieMark<D: ?Sized>: Mark {
    /// Calls `f` with each point's index and its size, which sets how far out its part's slice
    /// reaches (see `RadiiMark`). Points without a size aren't passed.
    fn sizes(&self, data: &D, f: &mut dyn FnMut(usize, Scalar));
}

/// A pie mark together with the dataset it draws, as a pie controller reads it from a dataset's
/// `PieDataSet::pie_marks`.
pub trait DatasetPieMark {
    fn role(&self) -> MarkRole;

    fn sizes(&self, f: &mut dyn FnMut(usize, Scalar));
}

pub(crate) struct BoundPieMark<'a, M, D> {
    pub(crate) mark: &'a M,
    pub(crate) data: &'a D,
}

impl<M: PieMark<D>, D> DatasetPieMark for BoundPieMark<'_, M, D> {
    fn role(&self) -> MarkRole {
        <M::Role as Role>::ROLE
    }

    fn sizes(&self, f: &mut dyn FnMut(usize, Scalar)) {
        self.mark.sizes(self.data, f);
    }
}

/// Sets how far out each part's slice reaches from its size (see `SizedDataSetExt::radii`): a
/// variable-radius pie. The controller measures every dataset's sizes against the largest of
/// them, so slices compare across rings: the largest reaches its ring's rim, and the others reach
/// out in proportion, linearly, from the ring's inner edge, as ag-charts' `radiusKey` does. A part
/// without a size reaches the rim; one with a size of zero or less has no slice. Along the sides,
/// on bars of 100%, the size sets each segment's width instead.
#[derive(Copy, Clone, Debug, Default)]
pub struct RadiiMark;

impl Mark for RadiiMark {
    type Role = Primary;
}

impl<D: Sequence<dyn SizedPoint>> PieMark<D> for RadiiMark {
    fn sizes(&self, data: &D, f: &mut dyn FnMut(usize, Scalar)) {
        data.foreach(&mut |index, point| {
            if let Some(size) = point.size() {
                f(index, size);
            }
        });
    }
}
