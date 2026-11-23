use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::ReadState;
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, StackId};
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::scale::ScaleId;
use crate::dataset::{BoxPoint, OhlcPoint, OptionsDataset, RangePoint, SizedPoint};
use crate::mark::bar_mark::BoundBarMark;
use crate::mark::line_mark::BoundLineMark;
use crate::mark::scatter_mark::BoundScatterMark;
use crate::mark::{BarDataSet, BarMark, DatasetBarMark, DatasetLineMark, DatasetPieMark, DatasetScatterMark, LineDataSet, LineMark, Mark, Overlay, PieDataSet, PieMark, ScatterDataSet, ScatterMark};
use crate::mark::pie_mark::BoundPieMark;
use crate::sequence::Sequence;

/// A dataset with a mark, made by a mark's extension method, such as `RangeDataSetExt::range`, or
/// by `DatasetExt::draw_as` and `DatasetExt::overlay` for marks of your own. It
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
    /// options, and with them its type. A mark of your own can offer option builders after it is
    /// attached through it, in an extension trait on `Marked<D, YourMark>`, as the built-in marks'
    /// options read (`data.error_bars().whisker_color(..)`):
    ///
    /// ```
    /// use carbide_chart::{DatasetExt, ErrorBars, Mark, Marked, Overlay};
    ///
    /// /// A mark with one option.
    /// #[derive(Clone, Debug, Default)]
    /// struct Halo { width: f64 }
    ///
    /// impl Mark for Halo {
    ///     type Role = Overlay;
    /// }
    ///
    /// trait HaloExt {
    ///     fn width(self, width: f64) -> Self;
    /// }
    ///
    /// impl<D> HaloExt for Marked<D, Halo> {
    ///     fn width(self, width: f64) -> Self {
    ///         self.map_mark(|halo| Halo { width, ..halo })
    ///     }
    /// }
    ///
    /// let data = vec![1.0, 2.0].overlay(Halo::default()).width(3.0);
    /// assert_eq!(data.mark().width, 3.0);
    /// ```
    pub fn map_mark<M2>(self, f: impl FnOnce(M) -> M2) -> Marked<D, M2> {
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

    fn set_visible(&self, visible: bool) -> bool {
        self.inner.set_visible(visible)
    }
}

/// A marked dataset is still a sequence of whatever kind of point the dataset inside is, so
/// another mark for that kind can be added to it.
macro_rules! point_sequences {
    ($($point:ident),+) => {
        $(
            impl<D: Sequence<dyn $point>, M: Clone + Debug + 'static> Sequence<dyn $point> for Marked<D, M> {
                fn foreach(&self, f: &mut dyn FnMut(usize, &(dyn $point + 'static))) {
                    self.inner.foreach(f);
                }

                fn sync(&mut self, env: &mut Environment) -> bool {
                    Sequence::<dyn $point>::sync(&mut self.inner, env)
                }
            }
        )+
    };
}

point_sequences!(RangePoint, SizedPoint, BoxPoint, OhlcPoint);

/// Drawn by bar controllers when the dataset inside is, and the mark can be drawn in bar slots.
impl<D: BarDataSet + Clone, M: BarMark<D>> BarDataSet for Marked<D, M> {
    fn bar_marks(&self, f: &mut dyn FnMut(&dyn DatasetBarMark)) {
        self.inner.bar_marks(f);
        f(&BoundBarMark { mark: &self.mark, data: &self.inner });
    }
}

/// Drawn by pie controllers when the dataset inside is, and the mark can be drawn by them.
impl<D: PieDataSet + Clone, M: PieMark<D>> PieDataSet for Marked<D, M> {
    fn pie_marks(&self, f: &mut dyn FnMut(&dyn DatasetPieMark)) {
        self.inner.pie_marks(f);
        f(&BoundPieMark { mark: &self.mark, data: &self.inner });
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

/// A dataset without a mark that replaces its glyph (`MarkRole::Primary`), so one can be added:
/// the methods that add a replacing mark (`.range()`, `.box_plot()`, `.candles()`, `.ohlc()`,
/// `.bubbles()`) require it. A dataset has at most one replacing mark, and any number of overlays
/// (`.error_bars()`), so a second replacing mark is a compile error rather than a mark that is
/// silently not drawn.
///
/// Implemented for every sequence of points, for datasets with options (`DatasetExt`) when the
/// dataset inside is one, and for marked datasets (`Marked`) when the mark is an overlay.
///
/// ```
/// use carbide_chart::{BarController, Estimate, RangeDataSetExt};
///
/// let measured = vec![Estimate { x: 0.0, y: 6.0, low: 5.0, high: 7.0 }];
/// BarController::new(measured.error_bars().range());
/// ```
///
/// ```compile_fail
/// use carbide_chart::{BarController, Range, RangeDataSetExt};
///
/// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
/// BarController::new(temperatures.range().range());
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` already has a mark that replaces its glyph",
    label = "already has a replacing mark",
    note = "a dataset has at most one replacing mark (`.range()`, `.box_plot()`, `.candles()`, `.ohlc()`, `.bubbles()`, `.radii()`, or `.draw_as(..)` with a mark of your own), and any number of overlays (`.error_bars()`, `.overlay(..)`)",
)]
pub trait WithoutPrimaryMark {}

/// A plain dataset has no marks.
impl<S: Sequence<dyn DataPoint>> WithoutPrimaryMark for S {}

/// A marked dataset still has no replacing mark when its mark is an overlay.
impl<D: WithoutPrimaryMark, M: Mark<Role = Overlay>> WithoutPrimaryMark for Marked<D, M> {}

/// Options keep what the dataset inside has.
impl<T, C, S, F, G, XS, YS, ST, L> WithoutPrimaryMark for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    T: WithoutPrimaryMark,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{}
