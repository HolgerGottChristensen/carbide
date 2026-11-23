//! The chart elements of this crate, each drawn by a chart through the public API alone.

use std::cell::RefCell;
use std::rc::Rc;

use carbide::chart::{Axis, Chart, ChartStyleExt, DataColor, DatasetExt, LegendPosition, LineController, ScaleId, ScatterController, SeriesStyle, ZoomWindow};
use carbide::color::{Color, GREEN, ORANGE, PURPLE, RED};
use carbide::draw::{Dimension, Position};
use carbide::environment::Environment;
use carbide::widget::VStack;
use carbide::state::{LocalState, ReadState};
use carbide::testing::Harness;
use chart_extensions::{Kelvin, LogScale, LollipopController};

use support::Framed;

/// A line through readings in `Kelvin`, a coordinate type of this crate's own: the y scale is in
/// degrees Celsius, as `Kelvin` places itself, and the reading at absolute zero is missing, so the
/// line has a gap there.
#[test]
fn kelvin_readings_in_a_line() {
    let readings = vec![(0.0, Kelvin(285.2)), (1.0, Kelvin(287.9)), (2.0, Kelvin(290.1)), (3.0, Kelvin(0.0)), (4.0, Kelvin(288.4)), (5.0, Kelvin(284.6)), (6.0, Kelvin(283.0))];

    let chart = Chart::new(LineController::new(readings))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("kelvin_readings_in_a_line");
}

/// Points spanning five orders of magnitude on a `LogScale` set as the chart's y scale: each power
/// of ten takes the same height, with a tick at each, from 100 to 10 000 000.
#[test]
fn points_on_a_log_scale() {
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(ScatterController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical).title("Inhabitants"))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("points_on_a_log_scale");
}

/// Clicks at three heights the same distance apart on a chart with a `LogScale` as its y scale.
/// Pins down that the scale reads a position back through the inverse of its own mapping: equal
/// steps up the chart multiply the value by the same factor, as each power of ten takes the same
/// height, where a linear reading would add the same amount.
#[test]
fn values_read_back_from_a_log_scale() {
    let clicked: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(vec![]));
    let record = clicked.clone();
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(ScatterController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .on_click(move |click| {
            let value = click.pick.value(&ScaleId::Y).expect("the chart has a y scale").value;
            record.borrow_mut().push(value);
        })
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));
    harness.render();

    // Downwards from near the top of the chart area, 80 pixels apart.
    harness.click(Position::new(300.0, 80.0));
    harness.click(Position::new(300.0, 160.0));
    harness.click(Position::new(300.0, 240.0));

    let values = clicked.borrow().clone();
    assert_eq!(values.len(), 3, "each click is in the chart area: {values:?}");
    assert!(values[0] > values[1] && values[1] > values[2], "values fall down the chart: {values:?}");
    assert!(values.iter().all(|value| (100.0..=10_000_000.0).contains(value)), "values are within the scale's range, 100 to 10 000 000: {values:?}");

    let (upper, lower) = (values[0] / values[1], values[1] / values[2]);
    assert!((upper - lower).abs() < 1e-6 * upper, "equal steps multiply by the same factor: {upper} and {lower}");
}

/// A line of readings on a `LogScale`, two of them at or below zero. Pins down that the scale has
/// no place for them, so they aren't drawn and the line has a gap at each, and that the range
/// starts at a thousandth of its end, as it does for data reaching zero.
#[test]
fn readings_at_or_below_zero_on_a_log_scale() {
    let readings = vec![(0.0, 40.0), (1.0, 300.0), (2.0, 0.0), (3.0, 2_500.0), (4.0, 900.0), (5.0, -15.0), (6.0, 6_000.0), (7.0, 7_500.0)];

    let chart = Chart::new(LineController::new(readings))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("readings_at_or_below_zero_on_a_log_scale");
}

/// Points on a `LogScale` zoomed into a stretch between two powers of ten, from about 12 600 to
/// 79 400. Pins down that the scale maps values through the window, and that, with no power of ten
/// inside it, its ticks are the multiples of 10 000 there, from 20 000 to 70 000; the points outside
/// the window are clipped.
#[test]
fn a_zoomed_log_scale() {
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(ScatterController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical).zoom(ZoomWindow::new(0.42, 0.58)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("a_zoomed_log_scale");
}

/// Lollipops on a `LogScale` zoomed into its middle half, dragged down. Pins down that the chart's
/// gestures reach a scale of this crate's own: the drag pans it, the data following the pointer, so
/// larger values come into view, and the window keeps its width.
#[test]
fn dragging_pans_a_log_scale() {
    let zoom = LocalState::new(ZoomWindow::new(0.25, 0.75));
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(LollipopController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical).zoom(zoom.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));
    harness.render();

    harness.drag(Position::new(300.0, 150.0), Position::new(300.0, 250.0), 4);
    harness.snapshot("dragging_pans_a_log_scale");

    let window = *zoom.value();
    assert!(window.start > 0.25 && (window.span() - 0.5).abs() < 1e-9, "moved towards larger values, keeping its width: {window:?}");
}

/// Two datasets drawn by the `LollipopController`, a controller of this crate's own, with options
/// set through `DatasetExt`: stems grow from zero, up or down; the first dataset takes the
/// palette's first colour, as a built-in controller's first dataset does, the second its own green,
/// and the third dataset's point its own red; the missing point has no lollipop; and both named
/// datasets are in the legend, in their colours.
#[test]
fn lollipops() {
    let first = vec![(0.0, 4.0), (1.0, -2.5), (2.0, 6.0), (3.0, f64::NAN), (4.0, 3.0)];
    let second = vec![(0.5, 2.0), (1.5, 5.0), (2.5, -4.0), (3.5, 1.5)];
    let marked = vec![(4.5, 7.0)].color(DataColor::Color(RED));

    let chart = Chart::new(LollipopController::new((
        first.label("Palette colour"),
        second.color(DataColor::Color(GREEN)).label("Own colour"),
        marked,
    )))
        .legend(LegendPosition::Bottom)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("lollipops");
}

/// Lollipops beside a line, in a series style of the test's own with three colours, set on a stack
/// around the chart. Pins down that a controller of this crate's own colours its datasets from the
/// chart's `SeriesStyle` as the built-in ones do (`SeriesColors`): the line takes the style's first
/// colour, purple; the lollipops, starting one colour in (`palette_offset`), its second, green;
/// a dataset of them coloured by point a colour per lollipop, from green on, wrapping to purple;
/// and the legend shows each dataset in its colour, orange for the one coloured by point.
#[test]
fn lollipops_follow_the_series_style() {
    /// Purple, green and orange.
    #[derive(Debug, Clone)]
    struct ThreeColours;

    impl SeriesStyle for ThreeColours {
        fn palette(&self, _env: &Environment) -> Vec<Color> {
            vec![PURPLE, GREEN, ORANGE]
        }
    }

    let line = vec![(0.0, 3.0), (1.0, 5.0), (2.0, 4.0), (3.0, 6.0), (4.0, 5.5), (5.0, 7.0)];
    let forecast = vec![(0.0, 2.0), (1.0, 4.0), (2.0, 3.0)];
    let by_point = vec![(2.5, 1.5), (3.0, 3.0), (3.5, 2.0), (4.0, 4.0), (4.5, 2.5)];

    let chart = Chart::new((
        LineController::new(line.label("Line")),
        LollipopController::new((
            forecast.label("Lollipops"),
            by_point.color(DataColor::ByPoint).label("By point"),
        )).palette_offset(1),
    ))
        .legend(LegendPosition::Bottom);

    let mut harness = Harness::new(VStack::new(chart).series_style(ThreeColours).framed(), Dimension::new(600.0, 400.0));

    harness.snapshot("lollipops_follow_the_series_style");
}

/// Lollipops on a `LogScale`, where zero isn't on the scale: the stems start at its bottom end.
#[test]
fn lollipops_on_a_log_scale() {
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(LollipopController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("lollipops_on_a_log_scale");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide::color::BLUE;
    use carbide::i18n::{locale, LocaleExt};
    use carbide::state::ValueState;
    use carbide::widget::Widget;

    /// How every chart in the snapshots is framed.
    pub trait Framed: Widget + Sized {
        /// Draws a blue border around the chart and formats its tick labels in Danish, so the
        /// snapshots don't depend on the machine's locale.
        fn framed(self) -> impl Widget {
            self.border().color(BLUE)
                .locale(ValueState::new(locale!("da")))
        }
    }

    impl<W: Widget> Framed for W {}
}
