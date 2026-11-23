//! The chart elements of this crate, each drawn by a chart through the public API alone.

use std::cell::RefCell;
use std::rc::Rc;

use carbide::chart::{Axis, BarController, Chart, ChartStyleExt, DataColor, DatasetController, DatasetExt, DatumKey, LegendPosition, LinearScale, LineController, Pick, PickMode, ScaleBounds, ScaleId, ScaleKind, ScatterController, SeriesStyle, TimeScale, TimeScaleZone, ZoomWindow};
use carbide::color::{Color, GREEN, ORANGE, PURPLE, RED};
use carbide::draw::{Dimension, Position};
use carbide::environment::Environment;
use carbide::widget::VStack;
use carbide::state::{LocalState, ReadState};
use carbide::testing::Harness;
use chrono::TimeZone;
use chart_extensions::{Blocks, BlocksExt, Kelvin, LogScale, LollipopController, ValueTicks};

use support::{utc, Framed};

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

/// Three datasets in a bar controller: one drawn as `Blocks`, a bar mark of this crate's own,
/// attached with `draw_as` and its unit set to 2 after attaching, with `ValueTicks`, an overlay of
/// its own, on top; plain bars beside it; and blocks of 1 on a dataset with a picked colour, set
/// before attaching. Pins down that a mark from outside the crate is drawn in its dataset's slot,
/// replacing the bar, in the dataset's colour, down from zero for a negative value, with the last
/// block cut short at the value; that the overlay marks each value on top; and that the legend shows
/// each named dataset.
#[test]
fn bars_drawn_by_marks_of_its_own() {
    let sales = vec![7.0, 4.5, -3.0, 9.0];
    let plain = vec![6.0, 5.0, 2.0, 8.0];
    let small = vec![3.0, 2.0, 1.5, 4.0];

    let sales = sales.draw_as(Blocks::new()).unit(2.0);
    assert_eq!(sales.mark().block_unit(), 2.0, "the unit is set after attaching");
    let sales = sales.overlay(ValueTicks).label("Blocks of 2");

    let chart = Chart::new(BarController::new((
        sales,
        plain.label("Plain bars"),
        small.color(DataColor::Palette(3)).label("Blocks of 1").draw_as(Blocks::new().unit(1.0)),
    )))
        .legend(LegendPosition::Bottom)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("bars_drawn_by_marks_of_its_own");
}

/// Bars drawn as `Blocks`, hovered by point, over a gap between two blocks of the second bar, then
/// over the empty space above it. Pins down that a mark from outside the crate makes the bars it
/// draws found under the pointer (`BarCanvas::add_rect`), across the whole bar, gaps included, and
/// that nothing is found beside it.
#[test]
fn hovering_finds_bars_drawn_by_a_mark_of_its_own() {
    let hovered = LocalState::new(None::<Pick>);
    let chart = Chart::new(BarController::new(vec![8.0, 10.0, 9.0, 7.0].draw_as(Blocks::new().unit(1.0))))
        .hover(hovered.clone())
        .hover_mode(PickMode::Point)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));
    harness.render();

    harness.mouse_move(Position::new(232.0, 300.0));
    let pick = hovered.value().clone().expect("the pointer is over the chart area");
    assert_eq!(pick.data, vec![DatumKey { controller: 0, dataset: 0, index: 1 }], "the second bar is found");

    harness.mouse_move(Position::new(232.0, 15.0));
    let pick = hovered.value().clone();
    assert!(pick.as_ref().is_none_or(|pick| pick.data.is_empty()), "nothing above the bar: {pick:?}");
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

/// Lollipops of names and of dates, collected into fresh bounds. Pins down that a controller of
/// this crate's own registers its datasets' values through the public API
/// (`ScaleBounds::register_values`), so names become categories in the order the points give them
/// and a dataset's first value picks the kind of a default scale: a category scale for names, a
/// time scale for dates, and none for numbers.
#[test]
fn lollipops_pick_their_default_scale_from_their_data() {
    let names = LollipopController::new(vec![("Tue", 5.0), ("Mon", 3.0), ("Tue", 4.0)]);
    let mut bounds = ScaleBounds::default();
    names.collect_bounds(&mut bounds, &mut Environment::new());
    assert_eq!(bounds.requests().get(&ScaleId::X).data_kind(), Some(ScaleKind::Category));
    assert_eq!(bounds.placement().categories(&ScaleId::X), vec!["Tue".to_string(), "Mon".to_string()]);
    assert_eq!(bounds.requests().get(&ScaleId::X).range(), Some((0.0, 1.0)), "placed on their categories");

    let dates = LollipopController::new(vec![(utc().with_ymd_and_hms(2026, 3, 2, 0, 0, 0).unwrap(), 4.0)]);
    let mut bounds = ScaleBounds::default();
    dates.collect_bounds(&mut bounds, &mut Environment::new());
    assert_eq!(bounds.requests().get(&ScaleId::X).data_kind(), Some(ScaleKind::Time));

    let numbers = LollipopController::new(vec![(0.0, 4.0)]);
    let mut bounds = ScaleBounds::default();
    numbers.collect_bounds(&mut bounds, &mut Environment::new());
    assert_eq!(bounds.requests().get(&ScaleId::X).data_kind(), None);
}

/// Two charts of lollipops: weekdays named by the data, on the category scale they pick for the
/// default x scale, the second dataset adding a day the first doesn't have, the first and last
/// half a step in from the ends (the controller asks for an offset, `ScaleRequests::request_offset`);
/// and a week of dates on a time scale in UTC. Pins down that a controller of this crate's own places
/// names and dates as the built-in controllers do, and maps them to the screen with `Scales::pixel`.
#[test]
fn lollipops_on_names_and_dates() {
    let visits = vec![("Mon", 12.0), ("Tue", 18.0), ("Wed", 9.0), ("Thu", 15.0)];
    let late = vec![("Thu", 6.0), ("Fri", 11.0)];
    let names = Chart::new(LollipopController::new((visits.label("Visits"), late.label("Late visits"))))
        .legend(LegendPosition::Bottom);

    let days: Vec<_> = (2..=8).map(|day| (utc().with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap(), (day * 3 % 7) as f64 + 2.0)).collect();
    let dates = Chart::new(LollipopController::new(days))
        .scale(ScaleId::X, TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc())));

    let mut harness = Harness::new(VStack::new((names, dates)).framed(), Dimension::new(600.0, 600.0));

    harness.snapshot("lollipops_on_names_and_dates");
}

/// Lollipops laid along x (`horizontal`), their names down the y scale. Pins down that a
/// controller of this crate's own can be drawn horizontally through the public `Orientation`: its
/// index scale is the y scale, which takes the categories, offset half a step at both ends, and its
/// stems grow from zero along x, left for a negative value.
#[test]
fn horizontal_lollipops() {
    let margins = vec![("North", 8.0), ("South", -3.5), ("East", 5.0), ("West", 11.0)];
    let chart = Chart::new(LollipopController::new(margins.label("Margin")).horizontal(true))
        .legend(LegendPosition::Bottom)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("horizontal_lollipops");
}

/// Lollipops on an x scale zoomed into its first half, with a y scale that fits to the visible data.
/// Pins down that a controller of this crate's own fits to the lollipops shown, through
/// `ScaleRequests::include_point`: the large values in the second half are left out of y, which runs
/// to the largest value shown, 9, not 40.
#[test]
fn lollipops_fit_the_visible_data() {
    let readings = vec![(0.0, 4.0), (1.0, 9.0), (2.0, 6.0), (3.0, 3.0), (6.0, 35.0), (7.0, 40.0), (8.0, 28.0)];
    let chart = Chart::new(LollipopController::new(readings))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(ZoomWindow::new(0.0, 0.4)))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).fit_visible(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("lollipops_fit_the_visible_data");
}

/// The lollipops of `lollipops`, each dataset's visibility a state, with a click on the "Own colour"
/// legend entry, then, after a click elsewhere, another. Pins down that a controller of this
/// crate's own hides and shows a dataset from its legend entry (`legend_visibility`): its
/// lollipops are gone, the scales refit to the ones shown, and its entry is dimmed; the second
/// click shows it again.
#[test]
fn clicking_a_legend_entry_hides_and_shows_lollipops() {
    let shown = [LocalState::new(true), LocalState::new(true)];
    let first = vec![(0.0, 4.0), (1.0, -2.5), (2.0, 6.0), (3.0, f64::NAN), (4.0, 3.0)];
    let second = vec![(0.5, 2.0), (1.5, 5.0), (2.5, -4.0), (3.5, 1.5)];

    let chart = Chart::new(LollipopController::new((
        first.label("Palette colour").visible(shown[0].clone()),
        second.color(DataColor::Color(GREEN)).label("Own colour").visible(shown[1].clone()),
    )))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));
    harness.render();

    harness.click(Position::new(370.0, 362.0));
    harness.snapshot("own_colour_hidden");
    assert_eq!((*shown[0].value(), *shown[1].value()), (true, false), "the second dataset is hidden");

    // A click elsewhere first, so the next click on the entry isn't the second of a double-click.
    harness.click(Position::new(5.0, 5.0));
    harness.click(Position::new(370.0, 362.0));
    assert_eq!((*shown[0].value(), *shown[1].value()), (true, true), "shown again");
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
    use chrono::FixedOffset;
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

    /// UTC, so dates and time scales are the same on every machine.
    pub fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).expect("zero is a valid offset")
    }
}
