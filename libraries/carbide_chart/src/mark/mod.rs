//! Marks: what a dataset draws in its place in a controller. A controller decides the layout (a
//! bar controller the slots along the index scale, a line controller the continuous runs), and a
//! mark decides what is drawn there. Without a mark, a dataset draws as its controller's default:
//! a bar, or a line.
//!
//! A mark either replaces that default glyph (`MarkRole::Primary`, such as `RangeDataSetExt::range`,
//! a floating bar), or is drawn on top of it (`MarkRole::Overlay`, such as
//! `RangeDataSetExt::error_bars`). A dataset has at most one replacing mark and any number of
//! overlays.
//!
//! Which controllers a dataset can be drawn by is checked when compiling. Each layout family has
//! a dataset trait (`BarDataSet`, `LineDataSet`, `ScatterDataSet`), which its controllers require. A plain dataset
//! implements all of them; a dataset with a mark implements a family's trait only when the mark
//! can be drawn in that family (`BarMark`, `LineMark`, `ScatterMark`). So a mark given to a controller that can't draw it is
//! a compile error, not a dataset that silently draws nothing.
//!
//! The marks that read more than an x and a y are extension methods on datasets of points that
//! have those values: `RangeDataSetExt` on datasets of `RangePoint`s, and `SizedDataSetExt` on
//! datasets of `SizedPoint`s, `BoxDataSetExt` on datasets of `BoxPoint`s, and `OhlcDataSetExt` on
//! datasets of `OhlcPoint`s.

mod bar_mark;
mod families;
mod marked;
mod range_mark;
mod error_bars;
mod scatter_mark;
mod line_mark;
mod box_mark;
mod candle_mark;
mod bubble_mark;

pub use bar_mark::{BarCanvas, BarMark, BarSlot, BarSlots, DatasetBarMark};
pub use families::{BarDataSet, IntoBoxed, LineDataSet, ScatterDataSet};
pub(crate) use families::AsDataSet;
pub use marked::Marked;
pub use range_mark::RangeMark;
pub use error_bars::ErrorBars;
pub use scatter_mark::{DatasetScatterMark, ScatterCanvas, ScatterMark, SizeScale};
pub use bubble_mark::BubbleMark;
pub use line_mark::{DatasetLineMark, LineCanvas, LineMark};
pub use box_mark::BoxMark;
pub use candle_mark::{CandleGlyph, CandleMark};

use std::fmt::Debug;
use carbide::environment::Environment;
use crate::DataSet;
use crate::dataset::{BoxPoint, OhlcPoint, RangePoint, SizedPoint};
use crate::sequence::Sequence;

/// What every mark is, whichever controllers draw it (`BarMark`, `LineMark`, `ScatterMark`): a
/// value that can be cloned, printed, and synced. A mark's options can be states, as a dataset's
/// can, and its dataset syncs them before every update, render and event (see `DataSet::sync`).
pub trait Mark: Clone + Debug + 'static {
    /// Syncs the states in the mark's options with the environment, and returns whether anything
    /// may have changed. The default syncs nothing: a mark without states has nothing to sync.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }
}

/// Whether a mark replaces the glyph its controller draws by default, or is drawn on top of it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MarkRole {
    /// Drawn instead of the controller's default glyph: a floating bar instead of a bar. The
    /// controller then leaves the dataset's own values out of the scale bounds, and the mark
    /// reports its values instead.
    Primary,
    /// Drawn on top of the dataset's glyph, after every dataset of the controller is drawn: error
    /// bars over bars.
    Overlay,
}

/// Marks for datasets of `RangePoint`s: points with a low and a high value.
pub trait RangeDataSetExt: DataSet + Sequence<dyn RangePoint> + Sized {
    /// Draws the ranges from each point's low to its high value. In a bar controller, each point
    /// is a floating bar instead of a bar growing from zero; in a line controller, the dataset is a
    /// band between its lows and its highs instead of a line. The value scale's bounds cover the
    /// ranges, without zero. Datasets drawn as ranges don't stack: in a bar controller each takes
    /// its own share of the slot.
    fn range(self) -> Marked<Self, RangeMark> {
        Marked::new(self, RangeMark::default())
    }

    /// Draws a whisker from each point's low to its high value on top of its bar, with a cap at
    /// each end. The value scale's bounds cover the whiskers.
    fn error_bars(self) -> Marked<Self, ErrorBars> {
        Marked::new(self, ErrorBars::default())
    }
}

impl<D: DataSet + Sequence<dyn RangePoint>> RangeDataSetExt for D {}

/// Marks for datasets of `SizedPoint`s: points with a size.
pub trait SizedDataSetExt: DataSet + Sequence<dyn SizedPoint> + Sized {
    /// Draws each point as a bubble whose area is proportional to its size, instead of a point.
    /// The scatter controller maps the sizes of all its datasets through one size scale, so
    /// bubbles compare across datasets (see `ScatterController::radius_range`).
    ///
    /// ```
    /// use carbide_chart::{Bubble, ScatterController, SizedDataSetExt};
    ///
    /// let cities = vec![Bubble { x: 3.0, y: 70.0, size: 1.2 }, Bubble { x: 5.0, y: 75.0, size: 0.4 }];
    /// ScatterController::new(cities.bubbles());
    /// ```
    ///
    /// Only scatter controllers draw bubbles:
    ///
    /// ```compile_fail
    /// use carbide_chart::{BarController, SizedDataSetExt};
    ///
    /// BarController::new(vec![(0.0, 4.0, 2.0)].bubbles());
    /// ```
    fn bubbles(self) -> Marked<Self, BubbleMark> {
        Marked::new(self, BubbleMark::default())
    }
}

impl<D: DataSet + Sequence<dyn SizedPoint>> SizedDataSetExt for D {}

/// Marks for datasets of `BoxPoint`s: points with a minimum, quartiles, a median and a maximum.
pub trait BoxDataSetExt: DataSet + Sequence<dyn BoxPoint> + Sized {
    /// Draws each point as a box plot in a bar controller's slot, instead of a bar: a box from the
    /// first to the third quartile, the median across it, and whiskers out to the minimum and the
    /// maximum (see `BoxMark`). The value scale's bounds cover the whiskers, without zero. Box
    /// plots don't stack: each dataset takes its own share of the slot, side by side.
    ///
    /// ```
    /// use carbide_chart::{BarController, BoxDataSetExt, BoxStats};
    ///
    /// let waits = vec![BoxStats { x: 0.0, min: 2.0, q1: 4.0, median: 5.0, q3: 7.0, max: 11.0 }];
    /// BarController::new(waits.box_plot());
    /// ```
    ///
    /// Only bar controllers draw box plots:
    ///
    /// ```compile_fail
    /// use carbide_chart::{BoxDataSetExt, BoxStats, LineController};
    ///
    /// let waits = vec![BoxStats { x: 0.0, min: 2.0, q1: 4.0, median: 5.0, q3: 7.0, max: 11.0 }];
    /// LineController::new(waits.box_plot());
    /// ```
    fn box_plot(self) -> Marked<Self, BoxMark> {
        Marked::new(self, BoxMark::default())
    }
}

impl<D: DataSet + Sequence<dyn BoxPoint>> BoxDataSetExt for D {}

/// Marks for datasets of `OhlcPoint`s: points with an open, a high, a low and a close.
pub trait OhlcDataSetExt: DataSet + Sequence<dyn OhlcPoint> + Sized {
    /// Draws each point as a candlestick in a bar controller's slot, instead of a bar: a body from
    /// the open to the close and a wick from the low to the high, in the rising or the falling
    /// colour (see `CandleMark`). The value scale's bounds cover the wicks, without zero.
    ///
    /// ```
    /// use carbide_chart::{BarController, Ohlc, OhlcDataSetExt};
    ///
    /// let prices = vec![Ohlc { x: 0.0, open: 10.0, high: 12.0, low: 9.0, close: 11.5 }];
    /// BarController::new(prices.candles());
    /// ```
    fn candles(self) -> Marked<Self, CandleMark> {
        Marked::new(self, CandleMark::new(CandleGlyph::Candlestick))
    }

    /// Draws each point as an OHLC bar in a bar controller's slot, instead of a bar: a line from
    /// the low to the high, with a tick at the open on one side and at the close on the other, in
    /// the rising or the falling colour (see `CandleMark`).
    fn ohlc(self) -> Marked<Self, CandleMark> {
        Marked::new(self, CandleMark::new(CandleGlyph::Ohlc))
    }
}

impl<D: DataSet + Sequence<dyn OhlcPoint>> OhlcDataSetExt for D {}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use crate::{BarController, BarDataSet, DataColor, DataSet, DatasetExt, Estimate, Range, RangeDataSetExt, StackId};
    use crate::controller::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId, ScaleKind};

    fn bounds<C: DatasetController>(controller: C) -> ScaleBounds {
        let mut bounds = ScaleBounds::default();
        controller.collect_bounds(&mut bounds, &mut Environment::new());
        bounds
    }

    /// Floating bars report their ranges to the value scale instead of their bars, so zero isn't
    /// included, and still ask the index scale for the bars' category scale and offsets.
    #[test]
    fn range_marks_report_their_ranges_without_zero() {
        let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }, Range { x: 1.0, low: 7.0, high: 18.0 }];
        let bounds = bounds(BarController::new(temperatures.range()));

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((4.0, 18.0)), "no zero, and the lows as well as the highs");

        let x = bounds.get(&ScaleId::X);
        assert_eq!(x.range(), Some((0.0, 1.0)));
        assert_eq!(x.kind(), Some(ScaleKind::Category));
        assert!(x.offset() && x.grid_offset());
    }

    /// Error bars are drawn on top of the bars, so the bars keep their bounds, zero included, and
    /// the whiskers add theirs.
    #[test]
    fn error_bars_add_their_whiskers_to_the_bars_bounds() {
        let measured = vec![Estimate { x: 0.0, y: 10.0, low: 8.0, high: 13.0 }, Estimate { x: 1.0, y: 6.0, low: 5.0, high: 7.0 }];
        let bounds = bounds(BarController::new(measured.error_bars()));

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((0.0, 13.0)));
    }

    /// On stacked bars the whiskers move with the bars: they keep their distance from the point's
    /// value, around where its bar ends. Below zero, the bars and whiskers stack downwards.
    #[test]
    fn error_bars_follow_the_stack() {
        const COSTS: StackId = StackId::new("costs");
        let materials = vec![Estimate { x: 0.0, y: 10.0, low: 9.0, high: 11.0 }];
        let labour = vec![Estimate { x: 0.0, y: 5.0, low: 4.0, high: 7.0 }];

        let up = bounds(BarController::new((materials.error_bars().stack(COSTS), labour.error_bars().stack(COSTS))));
        assert_eq!(up.get(&ScaleId::Y).range(), Some((0.0, 17.0)), "labour's whisker runs from 14 to 17, around its bar's top at 15");

        let refunds = vec![Estimate { x: 0.0, y: -10.0, low: -11.0, high: -9.0 }];
        let returns = vec![Estimate { x: 0.0, y: -5.0, low: -8.0, high: -4.0 }];

        let down = bounds(BarController::new((refunds.error_bars().stack(COSTS), returns.error_bars().stack(COSTS))));
        assert_eq!(down.get(&ScaleId::Y).range(), Some((-18.0, 0.0)), "returns' whisker runs from -18 to -14, around its bar's end at -15");
    }

    /// A point whose value is missing has no bar, so it has no whisker either, and its low and
    /// high don't reach the scale.
    #[test]
    fn a_missing_value_has_no_whisker() {
        let measured: Vec<Estimate<f64, Option<f64>>> = vec![
            Estimate { x: 0.0, y: Some(10.0), low: Some(8.0), high: Some(12.0) },
            Estimate { x: 1.0, y: None, low: Some(30.0), high: Some(40.0) },
        ];

        let bounds = bounds(BarController::new(measured.error_bars()));
        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((0.0, 12.0)));
    }

    /// The same range dataset draws in a bar controller and in a line controller. In a line
    /// controller its band reports the lows and highs, without zero, and doesn't stack, even with a
    /// stack id; lying down, the band's values go on the x scale. Its legend entry is a square, as
    /// for a filled area.
    #[test]
    fn ranges_in_lines_report_their_bands() {
        use crate::{LegendMarker, LineController};

        const SPAN: StackId = StackId::new("span");
        let temperatures = vec![Range { x: 0.0, low: 4.0, high: 12.0 }, Range { x: 1.0, low: 7.0, high: 18.0 }];
        let average = vec![8.0, 12.0];

        let bars = bounds(BarController::new(temperatures.clone().range()));
        assert_eq!(bars.get(&ScaleId::Y).range(), Some((4.0, 18.0)));

        let lines = LineController::new((temperatures.clone().range().stack(SPAN).label("range"), average.stack(SPAN)));
        let line_bounds = bounds(lines.clone());
        assert_eq!(line_bounds.get(&ScaleId::Y).range(), Some((4.0, 18.0)), "the band doesn't stack under the average, which stays at 8 to 12");
        assert_eq!(line_bounds.get(&ScaleId::X).kind(), Some(ScaleKind::Category));

        let mut legend = vec![];
        lines.legend_items(&Environment::new(), &mut legend);
        assert!(matches!(legend.as_slice(), [item] if item.marker == LegendMarker::Square), "{legend:?}");

        let lying = bounds(LineController::new(temperatures.range()).horizontal());
        assert_eq!(lying.get(&ScaleId::X).range(), Some((4.0, 18.0)));
        assert_eq!(lying.get(&ScaleId::Y).range(), Some((0.0, 1.0)));
    }

    /// A mark's options can be states, synced with its dataset: syncing a dataset whose inverted
    /// colour is a state reports that it may have changed, and the mark reads the state's current
    /// colour. A mark without options has nothing to sync. (As everywhere in carbide, any option
    /// set, even to a plain value, reports that it may have changed, until states can tell.)
    #[test]
    fn marks_sync_their_option_states_with_the_dataset() {
        let ranges = vec![Range { x: 0.0, low: 6.0, high: 2.0 }];
        let mut env = Environment::new();

        let mut plain = ranges.clone().range();
        assert!(!plain.sync(&mut env), "neither the points nor the mark hold a state");

        let mut color = LocalState::new(DataColor::Palette(3));
        let mut stateful = ranges.range().inverted_color(color.clone());
        assert!(stateful.sync(&mut env), "the mark's state may have changed");
        assert!(matches!(stateful.mark().inverted_color(), Some(DataColor::Palette(3))));

        *color.value_mut() = DataColor::Palette(5);
        stateful.sync(&mut env);
        assert!(matches!(stateful.mark().inverted_color(), Some(DataColor::Palette(5))));
    }

    /// Box plots and candlesticks report their whole span to the value scale, from the minimum or
    /// low to the maximum or high, without zero, and ask the index scale for the bars' category
    /// scale and offsets. They don't stack, even with a stack id.
    #[test]
    fn box_plots_and_candles_report_their_whole_span() {
        use crate::{BoxDataSetExt, BoxStats, Ohlc, OhlcDataSetExt};

        const ALL: StackId = StackId::new("all");
        let waits = vec![
            BoxStats { x: 0.0, min: 2.0, q1: 4.0, median: 5.0, q3: 7.0, max: 11.0 },
            BoxStats { x: 1.0, min: 3.0, q1: 5.0, median: 6.0, q3: 9.0, max: 14.0 },
        ];
        let boxes = bounds(BarController::new((waits.clone().box_plot().stack(ALL), waits.box_plot().stack(ALL))));
        assert_eq!(boxes.get(&ScaleId::Y).range(), Some((2.0, 14.0)), "not stacked, and without zero");
        assert_eq!(boxes.get(&ScaleId::X).kind(), Some(ScaleKind::Category));

        let prices = vec![
            Ohlc { x: 0.0, open: 10.0, high: 12.0, low: 9.0, close: 11.5 },
            Ohlc { x: 1.0, open: 11.5, high: 13.0, low: 8.0, close: 9.5 },
        ];
        let candles = bounds(BarController::new(prices.clone().candles()));
        assert_eq!(candles.get(&ScaleId::Y).range(), Some((8.0, 13.0)));
        let bars = bounds(BarController::new(prices.clone().ohlc()));
        assert_eq!(bars.get(&ScaleId::Y).range(), Some((8.0, 13.0)));

        let closes = bounds(crate::LineController::new(prices));
        assert_eq!(closes.get(&ScaleId::Y).range(), Some((9.5, 11.5)), "without a mark, a line runs through the closes");
    }

    /// The rising and falling colours of candlesticks can be states, synced with the dataset and
    /// read as last synced. Without them, the mark leaves the colours to the `SeriesStyle`.
    #[test]
    fn candle_colours_are_states() {
        use crate::{CandleGlyph, Ohlc, OhlcDataSetExt};

        let prices = vec![Ohlc { x: 0.0, open: 10.0, high: 12.0, low: 9.0, close: 11.5 }];
        let mut env = Environment::new();

        let plain = prices.clone().ohlc();
        assert_eq!(plain.mark().glyph(), CandleGlyph::Ohlc);
        assert!(plain.mark().rising_color().is_none() && plain.mark().falling_color().is_none());

        let mut rising = LocalState::new(DataColor::Palette(1));
        let mut candles = prices.candles().rising_color(rising.clone()).falling_color(DataColor::Palette(4));
        assert!(candles.sync(&mut env));
        assert!(matches!((candles.mark().rising_color(), candles.mark().falling_color()), (Some(DataColor::Palette(1)), Some(DataColor::Palette(4)))));

        *rising.value_mut() = DataColor::Palette(6);
        candles.sync(&mut env);
        assert!(matches!(candles.mark().rising_color(), Some(DataColor::Palette(6))));
    }

    /// Datasets drawn as ranges don't stack, even with a stack id: each reports its own range.
    #[test]
    fn ranges_dont_stack() {
        const SPAN: StackId = StackId::new("span");
        let first = vec![Range { x: 0.0, low: 2.0, high: 5.0 }];
        let second = vec![Range { x: 0.0, low: 3.0, high: 6.0 }];

        let bounds = bounds(BarController::new((first.range().stack(SPAN), second.range().stack(SPAN))));

        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((2.0, 6.0)), "not 2 to 11, as stacked sums would reach");
    }

    /// Marks and options combine in either order, and the options still apply.
    #[test]
    fn marks_and_options_combine_in_any_order() {
        const RIGHT: ScaleId = ScaleId::new("right");
        let measured = vec![Estimate { x: 0.0, y: 10.0, low: 8.0, high: 13.0 }];

        let mark_first = bounds(BarController::new(measured.clone().error_bars().y_scale(RIGHT).color(DataColor::Palette(2))));
        let options_first = bounds(BarController::new(measured.clone().y_scale(RIGHT).label("measured").error_bars()));

        for bounds in [mark_first, options_first] {
            assert_eq!(bounds.get(&RIGHT).range(), Some((0.0, 13.0)), "the whiskers go on the dataset's own y scale");
            assert_eq!(bounds.get(&ScaleId::Y).range(), None);
        }

        let mut marks = 0;
        measured.y_scale(RIGHT).error_bars().bar_marks(&mut |_| marks += 1);
        assert_eq!(marks, 1);
    }

    /// Plain and marked datasets of different types mix in one tuple, and boxed as bar datasets
    /// in a `Vec`. The controller doesn't decide which kind of box `.boxed()` makes, so it is
    /// named once: on the first box, or on the `Vec`.
    #[test]
    fn marked_datasets_mix_in_tuples_and_boxes() {
        let plain = vec![3.0, 4.0];
        let ranges = vec![Range { x: 0.0, low: 1.0, high: 9.0 }, Range { x: 1.0, low: 2.0, high: 5.0 }];
        let measured = vec![Estimate { x: 0.0, y: 6.0, low: 5.0, high: 11.0 }];

        let tuple = bounds(BarController::new((plain.clone(), ranges.clone().range(), measured.clone().error_bars())));
        assert_eq!(tuple.get(&ScaleId::Y).range(), Some((0.0, 11.0)));

        let named_first = bounds(BarController::new(vec![plain.clone().boxed::<dyn BarDataSet>(), ranges.clone().range().boxed(), measured.clone().error_bars().boxed()]));
        assert_eq!(named_first.get(&ScaleId::Y).range(), Some((0.0, 11.0)));

        let datasets: Vec<Box<dyn BarDataSet>> = vec![plain.boxed(), ranges.range().boxed(), measured.error_bars().boxed()];
        assert_eq!(bounds(BarController::new(datasets)).get(&ScaleId::Y).range(), Some((0.0, 11.0)));
    }

    /// Bubbles cover the x and y of the points they replace; their sizes don't reach the scales.
    /// Tuples with a third value and `Bubble`s are both sized points, and options and the mark
    /// combine in either order.
    #[test]
    fn bubbles_are_bounded_by_their_points() {
        use crate::{Bubble, ScatterController, SizedDataSetExt};

        let tuples = vec![(1.0, 10.0, 500.0), (4.0, 2.0, 50.0)];
        let structs = vec![Bubble { x: 2.0, y: 7.0, size: 1.0 }];

        let bounds = bounds(ScatterController::new((tuples.clone().bubbles(), structs.label("cities").bubbles(), tuples.bubbles().color(DataColor::Palette(3)))));

        assert_eq!(bounds.get(&ScaleId::X).range(), Some((1.0, 4.0)));
        assert_eq!(bounds.get(&ScaleId::Y).range(), Some((2.0, 10.0)), "the sizes don't reach the y scale");
    }

    /// A point without a bubble, its size negative or missing, isn't drawn, so it isn't in the
    /// bounds either, unless a line runs through it.
    #[test]
    fn points_without_a_bubble_are_left_out_of_the_bounds() {
        use crate::{Bubble, ScatterController, SizedDataSetExt};

        let sizes: Vec<Bubble<f64, f64, Option<f64>>> = vec![
            Bubble { x: 1.0, y: 5.0, size: Some(10.0) },
            Bubble { x: 2.0, y: 9.0, size: Some(0.0) },
            Bubble { x: 3.0, y: 50.0, size: Some(-4.0) },
            Bubble { x: 4.0, y: -20.0, size: None },
        ];

        let without_line = bounds(ScatterController::new(sizes.clone().bubbles()));
        assert_eq!(without_line.get(&ScaleId::X).range(), Some((1.0, 2.0)), "a size of zero still has a bubble");
        assert_eq!(without_line.get(&ScaleId::Y).range(), Some((5.0, 9.0)));

        let with_line = bounds(ScatterController::new(sizes.bubbles()).show_line(true));
        assert_eq!(with_line.get(&ScaleId::X).range(), Some((1.0, 4.0)), "the line runs through every point");
        assert_eq!(with_line.get(&ScaleId::Y).range(), Some((-20.0, 50.0)));
    }

    /// A state of marked datasets is a sequence of bar datasets, so marked datasets can be added
    /// at runtime.
    #[test]
    fn a_state_of_marked_datasets_can_change() {
        let mut datasets = LocalState::new(vec![vec![Range { x: 0.0, low: 4.0, high: 6.0 }].range()]);
        let controller = BarController::new(datasets.clone());

        assert_eq!(bounds(controller.clone()).get(&ScaleId::Y).range(), Some((4.0, 6.0)));

        datasets.value_mut().push(vec![Range { x: 0.0, low: 1.0, high: 2.0 }].range());
        assert_eq!(bounds(controller).get(&ScaleId::Y).range(), Some((1.0, 6.0)));
    }
}
