use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use crate::{DataPoint, DataSet, DataSetOptions};
use crate::dataset::{RangePoint, SizedPoint};
use crate::mark::bar_mark::BoundBarMark;
use crate::mark::line_mark::BoundLineMark;
use crate::mark::scatter_mark::BoundScatterMark;
use crate::mark::{BarDataSet, BarMark, DatasetBarMark, DatasetLineMark, DatasetScatterMark, LineDataSet, LineMark, Mark, ScatterDataSet, ScatterMark};
use crate::sequence::Sequence;

/// A dataset with a mark, made by a mark's extension method, such as `RangeDataSetExt::range`. It
/// draws like the dataset inside it, plus the mark, and can be drawn by the controllers whose
/// family the mark belongs to: by a bar controller when the mark is a `BarMark`, by a line
/// controller when it is a `LineMark`, and by a scatter controller when it is a `ScatterMark`. A
/// mark that is several, such as `RangeMark`, can be drawn by each of them.
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

    /// The mark, to read its options.
    pub fn mark(&self) -> &M {
        &self.mark
    }

    /// The same dataset with its mark replaced by what `f` makes of it, to change a mark's
    /// options, and with them its type.
    pub(crate) fn map_mark<M2>(self, f: impl FnOnce(M) -> M2) -> Marked<D, M2> {
        Marked { inner: self.inner, mark: f(self.mark) }
    }
}

impl<D: DataSet + Clone, M: Mark> DataSet for Marked<D, M> {
    fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint)) {
        self.inner.points(f);
    }

    fn min(&self) -> Option<(Scalar, Scalar)> {
        self.inner.min()
    }

    fn max(&self) -> Option<(Scalar, Scalar)> {
        self.inner.max()
    }

    /// Syncs the dataset inside, and the mark's options.
    fn sync(&mut self, env: &mut Environment) -> bool {
        let changed = self.inner.sync(env);
        self.mark.sync(env) | changed
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

/// A marked dataset of sized points is still a sequence of them, so another sized mark can be
/// added to it.
impl<D: Sequence<dyn SizedPoint>, M: Clone + Debug + 'static> Sequence<dyn SizedPoint> for Marked<D, M> {
    fn foreach(&self, f: &mut dyn FnMut(usize, &(dyn SizedPoint + 'static))) {
        self.inner.foreach(f);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        Sequence::<dyn SizedPoint>::sync(&mut self.inner, env)
    }
}

/// Drawn by bar controllers when the dataset inside is, and the mark can be drawn in bar slots.
impl<D: BarDataSet + Clone, M: BarMark<D>> BarDataSet for Marked<D, M> {
    fn bar_marks(&self, f: &mut dyn FnMut(&dyn DatasetBarMark)) {
        self.inner.bar_marks(f);
        f(&BoundBarMark { mark: &self.mark, data: &self.inner });
    }
}

/// Drawn by scatter controllers when the dataset inside is, and the mark can be drawn by them.
impl<D: ScatterDataSet + Clone, M: ScatterMark<D>> ScatterDataSet for Marked<D, M> {
    fn scatter_marks(&self, f: &mut dyn FnMut(&dyn DatasetScatterMark)) {
        self.inner.scatter_marks(f);
        f(&BoundScatterMark { mark: &self.mark, data: &self.inner });
    }
}

/// Drawn by line controllers when the dataset inside is, and the mark can be drawn by them.
impl<D: LineDataSet + Clone, M: LineMark<D>> LineDataSet for Marked<D, M> {
    fn line_marks(&self, f: &mut dyn FnMut(&dyn DatasetLineMark)) {
        self.inner.line_marks(f);
        f(&BoundLineMark { mark: &self.mark, data: &self.inner });
    }
}
