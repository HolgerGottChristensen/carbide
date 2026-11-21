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
//! can be drawn in that family (`BarMark`, `ScatterMark`). So a mark given to a controller that can't draw it is
//! a compile error, not a dataset that silently draws nothing.
//!
//! The marks that read more than an x and a y are extension methods on datasets of points that
//! have those values: `RangeDataSetExt` on datasets of `RangePoint`s, and `SizedDataSetExt` on
//! datasets of `SizedPoint`s.

mod bar_mark;
mod families;
mod marked;
mod range_mark;
mod error_bars;
mod scatter_mark;
mod bubble_mark;

pub use bar_mark::{BarCanvas, BarMark, BarSlot, BarSlots, DatasetBarMark};
pub use families::{BarDataSet, IntoBoxed, LineDataSet, ScatterDataSet};
pub(crate) use families::AsDataSet;
pub use marked::Marked;
pub use range_mark::RangeMark;
pub use error_bars::ErrorBars;
pub use scatter_mark::{DatasetScatterMark, ScatterCanvas, ScatterMark, SizeScale};
pub use bubble_mark::BubbleMark;

use crate::DataSet;
use crate::dataset::{RangePoint, SizedPoint};
use crate::sequence::Sequence;

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
    /// Draws each point as a floating bar from its low to its high value, instead of a bar
    /// growing from zero. The value scale's bounds cover the ranges, without zero. Datasets drawn
    /// as ranges don't stack: each takes its own share of the slot.
    fn range(self) -> Marked<Self, RangeMark> {
        Marked::new(self, RangeMark)
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

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use crate::{BarController, BarDataSet, DataColor, DatasetExt, Estimate, Range, RangeDataSetExt, StackId};
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
