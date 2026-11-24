use carbide::state::{IntoReadState, IntoState};
use crate::{DataColor, DataSet, StackId};
use crate::dataset::OptionsDataset;
use crate::element::{Fill, Interpolation, SpanGaps};
use crate::scale::ScaleId;
use crate::Mark;
use crate::mark::{IntoBoxed, Marked, WithoutPrimaryMark};
use crate::{Overlay, Primary};


pub trait DatasetExt: DataSet + Clone + Sized {
    fn color<C: IntoReadState<DataColor>>(self, color: C) -> OptionsDataset<Self, C::Output, Interpolation, Fill, SpanGaps, ScaleId, ScaleId, StackId, String> {
        OptionsDataset::new(self).color(color)
    }

    /// Draws the line between the points with `interpolation` instead of straight lines: smooth,
    /// monotone or stepped (see `Interpolation`).
    fn interpolation<S: IntoReadState<Interpolation>>(self, interpolation: S) -> OptionsDataset<Self, DataColor, S::Output, Fill, SpanGaps, ScaleId, ScaleId, StackId, String> {
        OptionsDataset::new(self).interpolation(interpolation)
    }

    /// Fills the area between the line and `fill`: zero, a value, or the bottom or top of the
    /// chart area, making a line chart an area chart (see `Fill`). Bar controllers ignore it.
    fn fill<F: IntoReadState<Fill>>(self, fill: F) -> OptionsDataset<Self, DataColor, Interpolation, F::Output, SpanGaps, ScaleId, ScaleId, StackId, String> {
        OptionsDataset::new(self).fill(fill)
    }

    /// Whether the line breaks or joins across gaps: missing values, or with `SpanGaps::Within`
    /// wide stretches along x (see `SpanGaps`). Bar controllers ignore it.
    fn span_gaps<G: IntoReadState<SpanGaps>>(self, span_gaps: G) -> OptionsDataset<Self, DataColor, Interpolation, Fill, G::Output, ScaleId, ScaleId, StackId, String> {
        OptionsDataset::new(self).span_gaps(span_gaps)
    }

    /// Draws the dataset on the horizontal scale `x_scale` instead of `ScaleId::X`.
    fn x_scale<XS: IntoReadState<ScaleId>>(self, x_scale: XS) -> OptionsDataset<Self, DataColor, Interpolation, Fill, SpanGaps, XS::Output, ScaleId, StackId, String> {
        OptionsDataset::new(self).x_scale(x_scale)
    }

    /// Draws the dataset on the vertical scale `y_scale` instead of `ScaleId::Y`.
    fn y_scale<YS: IntoReadState<ScaleId>>(self, y_scale: YS) -> OptionsDataset<Self, DataColor, Interpolation, Fill, SpanGaps, ScaleId, YS::Output, StackId, String> {
        OptionsDataset::new(self).y_scale(y_scale)
    }

    /// Stacks the dataset on the datasets before it in the same controller with the same
    /// `stack` (see `StackId`). Bars stack on top of each other instead of beside each other;
    /// lines are drawn at the running total, and with `Fill::Stack` fill down to the line below.
    /// Scatter controllers don't stack, and ignore it.
    fn stack<ST: IntoReadState<StackId>>(self, stack: ST) -> OptionsDataset<Self, DataColor, Interpolation, Fill, SpanGaps, ScaleId, ScaleId, ST::Output, String> {
        OptionsDataset::new(self).stack(stack)
    }

    /// Names the dataset, a text or a state of one. The name is shown in the chart's legend (see
    /// `Chart::legend`), next to a marker in the dataset's colour; a dataset without a name isn't
    /// in the legend.
    fn label<L: IntoReadState<String>>(self, label: L) -> OptionsDataset<Self, DataColor, Interpolation, Fill, SpanGaps, ScaleId, ScaleId, StackId, L::Output> {
        OptionsDataset::new(self).label(label)
    }

    /// Shows the dataset, or hides it with `false`; a writable state of a `bool`, which a click on
    /// the dataset's legend entry flips (see `Chart::legend_toggles`), so the app can read it, or
    /// set it from controls of its own. A hidden dataset is left out of the scales' fit, its stack
    /// and what is drawn, keeping its colour and its place among the others; its legend entry is
    /// dimmed. Without it, a dataset with options keeps a visibility state of its own, shown to
    /// start with.
    fn visible<V: IntoState<bool>>(self, visible: V) -> OptionsDataset<Self, DataColor, Interpolation, Fill, SpanGaps, ScaleId, ScaleId, StackId, String> {
        OptionsDataset::new(self).visible(visible)
    }

    /// Draws the dataset as `mark` instead of its controller's default glyph: a mark whose role is
    /// `Primary` (see `Mark`), such as a mark of your own, or a built-in one. A dataset has at most
    /// one such mark, and any number of overlays (`overlay`); a second doesn't compile.
    ///
    /// ```
    /// use carbide_chart::{BarController, DatasetExt, Range, RangeMark};
    ///
    /// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
    /// // The same as `temperatures.range()`.
    /// BarController::new(temperatures.draw_as(RangeMark::default()));
    /// ```
    ///
    /// ```compile_fail,E0277
    /// use carbide_chart::{BarController, DatasetExt, Range, RangeDataSetExt, RangeMark};
    ///
    /// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
    /// BarController::new(temperatures.range().draw_as(RangeMark::default()));
    /// ```
    fn draw_as<M: Mark<Role = Primary>>(self, mark: M) -> Marked<Self, M> where Self: WithoutPrimaryMark {
        Marked::new(self, mark)
    }

    /// Draws `mark` on top of the dataset's glyph: a mark whose role is `Overlay` (see `Mark`),
    /// such as a mark of your own, or a built-in one. A dataset can have any number.
    ///
    /// ```
    /// use carbide_chart::{BarController, DatasetExt, ErrorBars, Range};
    ///
    /// let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }];
    /// // The same as `temperatures.error_bars()`.
    /// BarController::new(temperatures.overlay(ErrorBars::default()));
    /// ```
    fn overlay<M: Mark<Role = Overlay>>(self, mark: M) -> Marked<Self, M> {
        Marked::new(self, mark)
    }

    /// Boxes the dataset, so datasets of different types can share a `Vec` or a state of a
    /// `Vec`, as the datasets of one controller. The box is for one kind of controller:
    /// `Box<dyn BarDataSet>` for bar controllers, `Box<dyn LineDataSet>` for line controllers,
    /// `Box<dyn ScatterDataSet>` for scatter controllers. The controller can't infer which, so
    /// name it once, on one of the boxes or on the `Vec`'s type, and the boxes next to it follow:
    /// `vec![bars.boxed::<dyn BarDataSet>(), more_bars.boxed()]`.
    fn boxed<B: ?Sized>(self) -> Box<B> where Self: IntoBoxed<B> {
        self.into_boxed()
    }
}

impl<T> DatasetExt for T where T: DataSet + Clone + Sized {}
