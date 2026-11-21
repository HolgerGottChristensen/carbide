use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use crate::{DataPoint, DataSet, DataSetOptions};
use crate::dataset::RangePoint;
use crate::mark::bar_mark::BoundBarMark;
use crate::mark::{BarDataSet, BarMark, DatasetBarMark};
use crate::sequence::Sequence;

/// A dataset with a mark, made by a mark's extension method, such as `RangeDataSetExt::range`. It
/// draws like the dataset inside it, plus the mark, and can be drawn by the controllers whose
/// family the mark belongs to: by a bar controller when the mark is a `BarMark`.
///
/// It is still a sequence of whatever points the dataset inside is, so marks and options can be
/// added in any order: `data.error_bars().color(..)` and `data.color(..).error_bars()` both work.
#[derive(Clone, Debug)]
pub struct Marked<D, M> {
    inner: D,
    mark: M,
}

impl<D, M> Marked<D, M> {
    pub(crate) fn new(inner: D, mark: M) -> Marked<D, M> {
        Marked { inner, mark }
    }
}

impl<D: DataSet + Clone, M: Clone + Debug + 'static> DataSet for Marked<D, M> {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint)) {
        self.inner.points(f);
    }

    fn min(&self) -> Option<(Scalar, Scalar)> {
        self.inner.min()
    }

    fn max(&self) -> Option<(Scalar, Scalar)> {
        self.inner.max()
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        self.inner.sync(env)
    }

    fn options(&self) -> DataSetOptions {
        self.inner.options()
    }
}

/// A marked dataset of range points is still a sequence of them, so another range mark can be
/// added to it.
impl<D: Sequence<dyn RangePoint>, M: Clone + Debug + 'static> Sequence<dyn RangePoint> for Marked<D, M> {
    fn foreach(&self, f: &mut dyn FnMut(usize, &(dyn RangePoint + 'static))) {
        self.inner.foreach(f);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        Sequence::<dyn RangePoint>::sync(&mut self.inner, env)
    }
}

/// Drawn by bar controllers when the dataset inside is, and the mark can be drawn in bar slots.
impl<D: BarDataSet + Clone, M: BarMark<D>> BarDataSet for Marked<D, M> {
    fn bar_marks(&self, f: &mut dyn FnMut(&dyn DatasetBarMark)) {
        self.inner.bar_marks(f);
        f(&BoundBarMark { mark: &self.mark, data: &self.inner });
    }
}
