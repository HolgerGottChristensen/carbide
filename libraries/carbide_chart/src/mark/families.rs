use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::ReadState;
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, StackId};
use crate::dataset::OptionsDataset;
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::mark::{DatasetBarMark, DatasetLineMark, DatasetPieMark, DatasetScatterMark};
use crate::scale::ScaleId;
use crate::sequence::{tuple_sequences, Sequence, SequenceItem};

/// A dataset a `BarController` can draw: a plain dataset, or one whose marks can all be drawn in
/// bar slots (`BarMark`). The bar controller reads the marks through `bar_marks`.
///
/// Implemented for every sequence of points, for datasets with options (`DatasetExt`) when the
/// dataset inside is one, for marked datasets (`Marked`) when the mark is a `BarMark`, and for
/// `Box<dyn BarDataSet>`, which mixes dataset types in a `Vec`.
///
/// ```
/// use carbide_chart::{BarController, Range, RangeDataSetExt};
///
/// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
/// BarController::new(temperatures.range());
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a bar controller",
    label = "not a bar dataset",
    note = "bar controllers draw plain datasets, and datasets whose marks are bar marks (`.range()`, `.error_bars()`, `.box_plot()`, `.candles()`, `.ohlc()`); bubbles are drawn by scatter controllers",
    note = "datasets of mixed types are boxed for one kind of controller: `Box<dyn BarDataSet>`",
)]
pub trait BarDataSet: DataSet {
    /// Calls `f` with each of the dataset's marks, in the order they were added. A plain dataset
    /// has none, and is drawn as bars.
    fn bar_marks(&self, _f: &mut dyn FnMut(&dyn DatasetBarMark)) {}
}

dyn_clone::clone_trait_object!(BarDataSet);

/// A dataset a `LineController` can draw: a plain dataset, or one whose marks can all be drawn by
/// line controllers (`LineMark`), such as a band (`.range()`) or error bars (`.error_bars()`). The line controller reads the marks
/// through `line_marks`.
///
/// Implemented for every sequence of points, for datasets with options (`DatasetExt`) when the
/// dataset inside is one, for marked datasets (`Marked`) when the mark is a `LineMark`, and for
/// `Box<dyn LineDataSet>`, which mixes dataset types in a `Vec`.
///
/// ```
/// use carbide_chart::{LineController, Range, RangeDataSetExt};
///
/// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
/// LineController::new(temperatures.range());
/// ```
///
/// A dataset with a mark lines can't draw doesn't compile:
///
/// ```compile_fail
/// use carbide_chart::{LineController, Ohlc, OhlcDataSetExt};
///
/// let prices = vec![Ohlc { x: 0.0, open: 10.0, high: 12.0, low: 9.0, close: 11.5 }];
/// LineController::new(prices.candles());
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a line controller",
    label = "not a line dataset",
    note = "line controllers draw plain datasets, and datasets whose marks are line marks (`.range()`, `.error_bars()`); box plots and candlesticks are drawn by bar controllers",
    note = "datasets of mixed types are boxed for one kind of controller: `Box<dyn LineDataSet>`",
)]
pub trait LineDataSet: DataSet {
    /// Calls `f` with each of the dataset's marks, in the order they were added. A plain dataset
    /// has none, and is drawn as a line.
    fn line_marks(&self, _f: &mut dyn FnMut(&dyn DatasetLineMark)) {}
}

dyn_clone::clone_trait_object!(LineDataSet);

/// A dataset a `ScatterController` can draw: a plain dataset, or one whose marks can all be drawn
/// by scatter controllers (`ScatterMark`), such as bubbles. The scatter controller reads the marks
/// through `scatter_marks`.
///
/// Scatter charts are a family of their own, apart from line charts: their points sit freely on
/// two numeric scales, in any order, with no index to group, stack or make horizontal, so the
/// marks that suit them differ. A band along the categories suits lines, a size per point suits
/// scatter points, and a mark that suits both implements both families.
///
/// Implemented for every sequence of points, for datasets with options (`DatasetExt`) when the
/// dataset inside is one, for marked datasets (`Marked`) when the mark is a `ScatterMark`, and for
/// `Box<dyn ScatterDataSet>`, which mixes dataset types in a `Vec`.
///
/// ```
/// use carbide_chart::{Estimate, ScatterController, RangeDataSetExt};
///
/// let measured = vec![Estimate { x: 0.0, y: 6.0, low: 5.0, high: 7.0 }];
/// ScatterController::new(measured.error_bars());
/// ```
///
/// A dataset with a mark scatter charts can't draw doesn't compile:
///
/// ```compile_fail
/// use carbide_chart::{Range, ScatterController, RangeDataSetExt};
///
/// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
/// ScatterController::new(temperatures.range());
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a scatter controller",
    label = "not a scatter dataset",
    note = "scatter controllers draw plain datasets, and datasets whose marks are scatter marks (`.bubbles()`, `.error_bars()`); `.range()` is drawn by bar and line controllers, box plots and candlesticks by bar controllers",
    note = "datasets of mixed types are boxed for one kind of controller: `Box<dyn ScatterDataSet>`",
)]
pub trait ScatterDataSet: DataSet {
    /// Calls `f` with each of the dataset's marks, in the order they were added. A plain dataset
    /// has none, and is drawn as points.
    fn scatter_marks(&self, _f: &mut dyn FnMut(&dyn DatasetScatterMark)) {}
}

dyn_clone::clone_trait_object!(ScatterDataSet);

/// A dataset a `PieController` can draw: a plain dataset, each point a part, or one whose marks
/// can all be drawn by pie controllers (`PieMark`), such as slice radii (`.radii()`). The pie
/// controller reads the marks through `pie_marks`.
///
/// Implemented for every sequence of points, for datasets with options (`DatasetExt`) when the
/// dataset inside is one, for marked datasets (`Marked`) when the mark is a `PieMark`, and for
/// `Box<dyn PieDataSet>`, which mixes dataset types in a `Vec`.
///
/// ```
/// use carbide_chart::{PieController, SizedDataSetExt};
///
/// PieController::new(vec![("Red", 12.0, 3.0), ("Blue", 19.0, 1.5)].radii());
/// ```
///
/// A dataset with a mark pies can't draw doesn't compile:
///
/// ```compile_fail
/// use carbide_chart::{PieController, Range, RangeDataSetExt};
///
/// PieController::new(vec![Range { x: 0.0, low: 4.0, high: 12.0 }].range());
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be drawn by a pie controller",
    label = "not a pie dataset",
    note = "pie controllers draw plain datasets, and datasets whose marks are pie marks (`.radii()`)",
    note = "datasets of mixed types are boxed for one kind of controller: `Box<dyn PieDataSet>`",
)]
pub trait PieDataSet: DataSet {
    /// Calls `f` with each of the dataset's marks, in the order they were added. A plain dataset
    /// has none, and its slices reach the rim.
    fn pie_marks(&self, _f: &mut dyn FnMut(&dyn DatasetPieMark)) {}
}

dyn_clone::clone_trait_object!(PieDataSet);

impl<S: Sequence<dyn DataPoint>> PieDataSet for S {}

impl<T, C, S, F, G, XS, YS, ST, L> PieDataSet for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    T: PieDataSet + Clone,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    fn pie_marks(&self, f: &mut dyn FnMut(&dyn DatasetPieMark)) {
        self.inner().pie_marks(f);
    }
}

impl<S: Sequence<dyn DataPoint>> BarDataSet for S {}

impl<S: Sequence<dyn DataPoint>> LineDataSet for S {}

impl<S: Sequence<dyn DataPoint>> ScatterDataSet for S {}

impl<T, C, S, F, G, XS, YS, ST, L> BarDataSet for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    T: BarDataSet + Clone,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    fn bar_marks(&self, f: &mut dyn FnMut(&dyn DatasetBarMark)) {
        self.inner().bar_marks(f);
    }
}

impl<T, C, S, F, G, XS, YS, ST, L> LineDataSet for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    T: LineDataSet + Clone,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    fn line_marks(&self, f: &mut dyn FnMut(&dyn DatasetLineMark)) {
        self.inner().line_marks(f);
    }
}

impl<T, C, S, F, G, XS, YS, ST, L> ScatterDataSet for OptionsDataset<T, C, S, F, G, XS, YS, ST, L>
where
    T: ScatterDataSet + Clone,
    C: ReadState<T=DataColor>,
    S: ReadState<T=Interpolation>,
    F: ReadState<T=Fill>,
    G: ReadState<T=SpanGaps>,
    XS: ReadState<T=ScaleId>,
    YS: ReadState<T=ScaleId>,
    ST: ReadState<T=StackId>,
    L: ReadState<T=String>,
{
    fn scatter_marks(&self, f: &mut dyn FnMut(&dyn DatasetScatterMark)) {
        self.inner().scatter_marks(f);
    }
}

/// Forwards every `DataSet` method of a boxed dataset of one family to the dataset inside.
macro_rules! boxed_dataset {
    ($family:ident { $($extra:tt)* }) => {
        impl DataSet for Box<dyn $family> {
            fn points(&self, f: &mut dyn FnMut(usize, &dyn DataPoint)) {
                (**self).points(f);
            }

            fn min(&self) -> Option<(Scalar, Scalar)> {
                (**self).min()
            }

            fn max(&self) -> Option<(Scalar, Scalar)> {
                (**self).max()
            }

            fn sync(&mut self, env: &mut Environment) -> bool {
                (**self).sync(env)
            }

            fn options(&self) -> DataSetOptions {
                (**self).options()
            }
        }

        impl $family for Box<dyn $family> {
            $($extra)*
        }

        /// A dataset of this family is an item of a sequence of them.
        impl<D: $family + Clone> SequenceItem<dyn $family> for D {
            fn visit(&self, index: usize, f: &mut dyn FnMut(usize, &(dyn $family + 'static))) {
                f(index, self);
            }

            fn sync(&mut self, env: &mut Environment) -> bool {
                DataSet::sync(self, env)
            }
        }

        impl AsDataSet for dyn $family {
            fn as_dataset(&self) -> &dyn DataSet {
                self
            }
        }

        impl<D: $family + Clone> IntoBoxed<dyn $family> for D {
            fn into_boxed(self) -> Box<dyn $family> {
                Box::new(self)
            }
        }

        tuple_sequences!([] dyn $family);
    };
}

boxed_dataset!(BarDataSet {
    fn bar_marks(&self, f: &mut dyn FnMut(&dyn DatasetBarMark)) {
        (**self).bar_marks(f);
    }
});

boxed_dataset!(LineDataSet {
    fn line_marks(&self, f: &mut dyn FnMut(&dyn DatasetLineMark)) {
        (**self).line_marks(f);
    }
});

boxed_dataset!(PieDataSet {
    fn pie_marks(&self, f: &mut dyn FnMut(&dyn DatasetPieMark)) {
        (**self).pie_marks(f);
    }
});

boxed_dataset!(ScatterDataSet {
    fn scatter_marks(&self, f: &mut dyn FnMut(&dyn DatasetScatterMark)) {
        (**self).scatter_marks(f);
    }
});

/// A dataset as the plain `DataSet` it is, whichever family a controller reads it as, so drawing
/// code shared between families takes any of them.
pub(crate) trait AsDataSet {
    fn as_dataset(&self) -> &dyn DataSet;
}

impl AsDataSet for dyn DataSet {
    fn as_dataset(&self) -> &dyn DataSet {
        self
    }
}

/// A dataset boxed as one family's trait object, `Box<dyn BarDataSet>`, `Box<dyn LineDataSet>` or
/// `Box<dyn ScatterDataSet>`, or as a plain `Box<dyn DataSet>` (see `DatasetExt::boxed`).
pub trait IntoBoxed<B: ?Sized> {
    fn into_boxed(self) -> Box<B>;
}

impl<D: DataSet + Clone> IntoBoxed<dyn DataSet> for D {
    fn into_boxed(self) -> Box<dyn DataSet> {
        Box::new(self)
    }
}
