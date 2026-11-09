use carbide_chart::Stepped;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

use support::{
    bar_chart, bars_and_line_chart, bars_on_linear_x_scale_chart, bars_on_secondary_y_scale_chart,
    bars_with_x_scale_on_top_chart, example_chart, labelled_bar_chart, line_then_bars_chart, scales_on_all_four_sides_chart,
    scatter_chart, scatter_with_lines_chart, two_bar_controllers_chart, vertical_category_scale_chart, vertical_category_scale_on_right_chart, stacked_horizontal_scales_chart, stacked_scales_chart,
    stacked_scales_on_left_chart, stacked_scales_on_top_chart, two_y_scales_chart, x_scale_on_top_chart,
};

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `stepped` mode driven by a state, Danish locale, bordered) and snapshots it once per `Stepped`
/// variant, the same sequence the example's picker steps through. The datasets are fixed, so
/// each snapshot pins down the linear scale's ticks and grid, the tick labels, the dataset
/// colors and the line path for one stepped mode.
#[test]
fn scatter_chart_stepped_modes() {
    let stepped = LocalState::new(Stepped::None);
    let mut harness = Harness::new(example_chart(stepped.clone()), Dimension::new(600.0, 500.0));

    for (mode, name) in [
        (Stepped::None, "stepped_none"),
        (Stepped::Before, "stepped_before"),
        (Stepped::After, "stepped_after"),
        (Stepped::Middle, "stepped_middle"),
        (Stepped::MiddleVertical, "stepped_middle_vertical"),
    ] {
        stepped.clone().set_value(mode);
        harness.snapshot(name);
    }
}

/// Two controllers on one chart: revenue on the default Y scale (left), and a margin line bound
/// to a custom `percent` scale on the right. Pins down that each dataset is drawn on its own
/// scale, that the right-hand scale gets its ticks, labels and border on the right, and that the
/// chart area makes room for it.
#[test]
fn two_y_scales() {
    let mut harness = Harness::new(two_y_scales_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("two_y_scales");
}

/// Two scales on the right, as in Chart.js: the first added sits against the chart area, the
/// second in its own band outside it. Pins down that neither scale's ticks, labels or border
/// overlap the other's, and that the chart area gives up both bands.
#[test]
fn stacked_scales_on_one_side() {
    let mut harness = Harness::new(stacked_scales_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_one_side");
}

/// The horizontal counterpart of `stacked_scales_on_one_side`: a second x scale stacked below the
/// default one, with spacing between their bands. Pins down that horizontal bands stack
/// downwards, each with its own tick marks, labels and border, and that the chart area gives up
/// the height of both bands.
#[test]
fn stacked_horizontal_scales() {
    let mut harness = Harness::new(stacked_horizontal_scales_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_horizontal_scales");
}

/// The example chart with its default x scale moved to the top. Pins down a horizontal scale's
/// top side: tick marks reaching up from the chart area, labels above them, its border along the
/// top edge, its band taken off the top of the chart area, and no band left at the bottom.
#[test]
fn x_scale_on_top() {
    let mut harness = Harness::new(x_scale_on_top_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("x_scale_on_top");
}

/// The temperature chart from `stacked_horizontal_scales`, with both x scales at the top: the
/// default x scale against the chart area, the °F scale stacked above it with spacing between
/// their bands.
#[test]
fn stacked_scales_on_top() {
    let mut harness = Harness::new(stacked_scales_on_top_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_top");
}

/// Two vertical scales on the left: revenue on the default y scale against the chart area, units
/// sold on a second scale stacked outside it with spacing between their bands.
#[test]
fn stacked_scales_on_left() {
    let mut harness = Harness::new(stacked_scales_on_left_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_left");
}

/// The point clouds of `scatter_points_only` with `show_line(true)`, the second one also
/// `stepped(After)`. Pins down that a scatter chart's lines connect the points in data order, not
/// sorted by x, so they double back like Chart.js's `showLine`; and that stepped lines work on a
/// scatter chart too.
#[test]
fn scatter_with_lines() {
    let mut harness = Harness::new(scatter_with_lines_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_with_lines");
}

/// Bars on a secondary y scale: margin as bars on a `percent` scale on the right, revenue as a
/// line on the default y scale on the left. Pins down that bars grow from zero on the scale they
/// are bound to, whose range is stretched to include zero, while the line's scale is not.
#[test]
fn bars_on_secondary_y_scale() {
    let mut harness = Harness::new(bars_on_secondary_y_scale_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_on_secondary_y_scale");
}

/// Bars with their category x scale at the top, labelled. Pins down a category scale's labels,
/// tick marks and grid offset on the top side, with bars still growing up from zero at the
/// bottom of the y scale.
#[test]
fn bars_with_x_scale_on_top() {
    let mut harness = Harness::new(bars_with_x_scale_on_top_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_with_x_scale_on_top");
}

/// Bars at numeric x values (hours of the day) on an explicit linear x scale. Pins down the
/// linear scale's offset of half a tick step at both ends, and bar widths taken from the smallest
/// gap between bars or ticks.
#[test]
fn bars_on_linear_x_scale() {
    let mut harness = Harness::new(bars_on_linear_x_scale_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_on_linear_x_scale");
}

/// A status over time on a labelled vertical category scale with grid offset. Pins down that
/// vertical category scales run from top to bottom (the first label at the top), with labels on
/// the left and grid lines between the categories.
#[test]
fn vertical_category_scale() {
    let mut harness = Harness::new(vertical_category_scale_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("vertical_category_scale");
}

/// A temperature line on the default linear y scale, and a status line on a vertical category
/// scale on the right, without grid lines. Pins down a category scale's labels on the right side
/// next to a linear scale on the left.
#[test]
fn vertical_category_scale_on_right() {
    let mut harness = Harness::new(vertical_category_scale_on_right_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("vertical_category_scale_on_right");
}

/// The data of `bars_and_line_on_shared_scales` with the controllers the other way round: the
/// line controller first. Its default x scale wins, a category scale without grid offset, so grid
/// lines run through the bar groups instead of between them; the bars still offset the scale.
/// The line is also drawn first, under the bars.
#[test]
fn line_then_bars() {
    let mut harness = Harness::new(line_then_bars_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("line_then_bars");
}

/// Two bar controllers on the same x scale. Bars are grouped only within one controller, so
/// separate controllers overlap: the second controller's bars are drawn over the first's in the
/// same slot, like the layers of a `ZStack`. This is intended (unlike Chart.js, which groups
/// across them); use one controller with several datasets for side-by-side bars.
#[test]
fn two_bar_controllers_on_one_x_scale() {
    let mut harness = Harness::new(two_bar_controllers_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("two_bar_controllers_on_one_x_scale");
}

/// A scale on every side of the chart area at once: °C along the bottom and humidity on the left
/// for one dataset, °F along the top and pressure on the right for the other. Pins down that each
/// side's band is laid out independently and the chart area gives up all four.
#[test]
fn scales_on_all_four_sides() {
    let mut harness = Harness::new(scales_on_all_four_sides_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scales_on_all_four_sides");
}

/// A plain scatter chart: two datasets of (x, y) points with numeric x values, points only, as
/// Chart.js's scatter chart draws them by default (`showLine: false`). Pins down that the
/// `ScatterController` draws no lines unless asked to, and keeps linear x and y scales.
#[test]
fn scatter_points_only() {
    let mut harness = Harness::new(scatter_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_points_only");
}

/// Bars and a line in one chart, on the same scales: revenue and costs as grouped bars from a
/// `BarController`, profit as a line from a `LineController` drawn over them. Pins down that
/// the two bar datasets sit side by side around each x value, that the first and last groups
/// aren't cut off at the chart area's edges (the x scale's offset), that bars grow from zero,
/// and that the line's points sit at the centre of each group, including the negative month.
#[test]
fn bars_and_line_on_shared_scales() {
    let mut harness = Harness::new(bars_and_line_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_and_line_on_shared_scales");
}

/// A plain bar chart: one dataset of positive values, nothing else on the chart (the data from
/// Chart.js's sample bar chart). Pins down the single-dataset bar width, the x scale's offset at
/// both ends, and bars growing up from zero at the bottom of the y scale.
#[test]
fn bar_chart_single_dataset() {
    let mut harness = Harness::new(bar_chart(vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_single_dataset");
}

/// Chart.js's sample bar chart, with its category labels: votes for six colours. The x scale is
/// set explicitly to a `CategoryScale` with labels, so the ticks show "Red", "Blue", ... instead
/// of indices. Pins down label text on a category scale and grid lines between the bars.
#[test]
fn bar_chart_with_labels() {
    let mut harness = Harness::new(labelled_bar_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_with_labels");
}

/// One dataset with both positive and negative values. Pins down that every bar starts at zero,
/// in the middle of the y scale: positive bars grow up from it, negative ones hang down from it.
#[test]
fn bar_chart_positive_and_negative() {
    let mut harness = Harness::new(bar_chart(vec![-12.0, 19.0, -3.0, 5.0, -2.0, 8.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_positive_and_negative");
}

/// Three grouped datasets with both positive and negative values. Pins down that each group
/// splits its slot three ways in dataset order (white, green, red), and that bars on either side
/// of zero in the same group share the zero line.
#[test]
fn grouped_bars_positive_and_negative() {
    let datasets = (
        vec![65.0, -59.0, 80.0, -81.0, 56.0],
        vec![-28.0, 48.0, -40.0, 19.0, 86.0],
        vec![35.0, -10.0, 22.0, 60.0, -45.0],
    );
    let mut harness = Harness::new(bar_chart(datasets), Dimension::new(600.0, 500.0));

    harness.snapshot("grouped_bars_positive_and_negative");
}

/// Grouped bars whose first dataset is held in a state, changed between the two snapshots: its
/// values grow past the old y range, and a sixth month is appended. Pins down that the chart
/// follows data changed after it is built: the y scale refits to the new values, the category x
/// scale gains a slot for the new month, and the second dataset, which didn't change, keeps its
/// bars in every month it has data for.
#[test]
fn data_changed_between_renders() {
    let revenue = LocalState::new(vec![12.0, 19.0, 3.0, 5.0, 2.0]);
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0];

    let mut harness = Harness::new(bar_chart((revenue.clone(), costs)), Dimension::new(600.0, 500.0));
    harness.snapshot("before_change");

    revenue.clone().set_value(vec![12.0, 19.0, 30.0, 5.0, 24.0, 17.0]);
    harness.snapshot("after_change");
}

/// Grouped bars whose datasets are a state of a `Vec`, with a third dataset added between the two
/// snapshots. Pins down that the chart follows a dataset sequence changed after it is built: each
/// month's slot splits three ways instead of two, the new dataset takes the next palette colour
/// (red), and the y scale refits to include its larger values.
#[test]
fn dataset_added_between_renders() {
    let revenue = vec![12.0, 19.0, 3.0, 5.0, 2.0];
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0];
    let datasets = LocalState::new(vec![revenue.clone(), costs.clone()]);

    let mut harness = Harness::new(bar_chart(datasets.clone()), Dimension::new(600.0, 500.0));
    harness.snapshot("two_datasets");

    datasets.clone().set_value(vec![revenue, costs, vec![15.0, 24.0, 9.0, 7.0, 4.0]]);
    harness.snapshot("three_datasets");
}

/// One dataset of only negative values. Zero is still included in the y scale, so it is the top
/// of the scale and every bar hangs down from it.
#[test]
fn bar_chart_all_negative() {
    let mut harness = Harness::new(bar_chart(vec![-5.0, -12.0, -8.0, -20.0, -3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_all_negative");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, DataColor, DataSetSequence, DataSet, DatasetExt, IntoSequence, LineController, LinearScale, ScaleId, ScatterController, Stepped};
    // `.border()` alone picks a random colour on every run, which makes snapshots flaky.
    use carbide_core::color::{BLUE, GREEN, RED};
    use carbide_core::draw::Scalar;
    use carbide_core::state::{LocalState, ValueState};
    use carbide_core::widget::{Widget, WidgetExt};
    use carbide_fluent::{locale, LocaleExt};

    const PERCENT: ScaleId = ScaleId::new("percent");
    const UNITS: ScaleId = ScaleId::new("units");
    const FAHRENHEIT: ScaleId = ScaleId::new("fahrenheit");
    const STATUS: ScaleId = ScaleId::new("status");
    const PRESSURE: ScaleId = ScaleId::new("pressure");

    const STATUS_LABELS: [&str; 4] = ["Low", "Medium", "High", "Critical"];

    /// Margin as bars on `percent` on the right, revenue as a line on Y on the left.
    pub fn bars_on_secondary_y_scale_chart() -> impl Widget {
        let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
            .y_scale(PERCENT)
            .color(DataColor::Color(GREEN));
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];

        Chart::new((
            BarController::new(margin),
            LineController::new(revenue),
        ))
            .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Chart.js's sample votes as bars, with the labelled category x scale at the top.
    pub fn bars_with_x_scale_on_top_chart() -> impl Widget {
        let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

        Chart::new(BarController::new(votes))
            .scale(
                ScaleId::X,
                CategoryScale::new(Axis::Horizontal)
                    .position(AxisPosition::Top)
                    .labels(["Red", "Blue", "Yellow", "Green", "Purple", "Orange"])
                    .grid_offset(true),
            )
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Visitors per four-hour block of a day, as bars at numeric x values (the hour) on a
    /// linear x scale.
    pub fn bars_on_linear_x_scale_chart() -> impl Widget {
        let visitors = vec![(0.0, 3.0), (4.0, 1.0), (8.0, 12.0), (12.0, 18.0), (16.0, 15.0), (20.0, 9.0), (24.0, 4.0)];

        Chart::new(BarController::new(visitors))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// A status per day (an index into `STATUS_LABELS`) on a vertical category y scale.
    pub fn vertical_category_scale_chart() -> impl Widget {
        let status = vec![0.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0];

        Chart::new(LineController::new(status))
            .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(STATUS_LABELS).grid_offset(true))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// A temperature on the default y scale, and the status from `vertical_category_scale_chart`
    /// on a category scale on the right.
    pub fn vertical_category_scale_on_right_chart() -> impl Widget {
        let temperature = vec![18.0, 21.0, 24.0, 29.0, 34.0, 30.0, 23.0];
        let status = vec![0.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0]
            .y_scale(STATUS)
            .color(DataColor::Color(RED));

        Chart::new(LineController::new((temperature, status)))
            .scale(STATUS, CategoryScale::new(Axis::Vertical).position(AxisPosition::Right).labels(STATUS_LABELS).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// `bars_and_line_chart` with the line controller before the bar controller.
    pub fn line_then_bars_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0];
        let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0, 25.0, 35.0]
            .color(DataColor::Color(RED));

        Chart::new((
            LineController::new(profit),
            BarController::new((revenue, costs)),
        ))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue and costs as bars in two separate bar controllers on the default scales.
    pub fn two_bar_controllers_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0]
            .color(DataColor::Color(GREEN));

        Chart::new((
            BarController::new(revenue),
            BarController::new(costs),
        ))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Humidity by temperature in °C on the default scales (bottom and left), and air pressure by
    /// temperature in °F on scales at the top and on the right.
    pub fn scales_on_all_four_sides_chart() -> impl Widget {
        let humidity = vec![(0.0, 82.0), (20.0, 70.0), (40.0, 64.0), (60.0, 55.0), (80.0, 48.0), (100.0, 40.0)];
        let pressure = vec![(32.0, 1002.0), (68.0, 1008.0), (104.0, 1011.0), (140.0, 1016.0), (176.0, 1021.0), (212.0, 1026.0)]
            .x_scale(FAHRENHEIT)
            .y_scale(PRESSURE)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(humidity).show_line(true),
            ScatterController::new(pressure).show_line(true),
        ))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false))
            .scale(PRESSURE, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// The example's two datasets, with the default x scale replaced by one at the top.
    pub fn x_scale_on_top_chart() -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0];
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(LineController::new((data7, data8)))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// `stacked_horizontal_scales_chart` with both x scales at the top. X is added first (it is a
    /// default scale, replaced in place), so it stays against the chart area.
    pub fn stacked_scales_on_top_chart() -> impl Widget {
        let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
        let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
            .x_scale(FAHRENHEIT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(celsius).show_line(true),
            ScatterController::new(fahrenheit).show_line(true),
        ))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue on the default y scale and units sold on a second scale, also on the left, added
    /// after Y so it stacks outside it.
    pub fn stacked_scales_on_left_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0, 3400.0, 3900.0]
            .y_scale(UNITS)
            .color(DataColor::Color(RED));

        Chart::new((
            LineController::new(revenue),
            LineController::new(units),
        ))
            .scale(UNITS, LinearScale::new(Axis::Vertical).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Two clouds of (x, y) points with numeric x values, drawn as points only.
    pub fn scatter_chart() -> impl Widget {
        let first = vec![(-10.0, 0.0), (0.0, 10.0), (10.0, 5.0), (0.5, 5.5), (-4.0, 7.5), (6.0, -2.0), (3.0, 3.0)];
        let second = vec![(-8.0, -4.0), (-2.0, 2.0), (2.5, -6.0), (7.5, 8.0), (9.0, 1.5), (-6.5, 4.0)];

        Chart::new(ScatterController::new((first, second)))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// `scatter_chart`'s points with lines through them in data order, the second dataset stepped.
    pub fn scatter_with_lines_chart() -> impl Widget {
        let first = vec![(-10.0, 0.0), (0.0, 10.0), (10.0, 5.0), (0.5, 5.5), (-4.0, 7.5), (6.0, -2.0), (3.0, 3.0)];
        let second = vec![(-8.0, -4.0), (-2.0, 2.0), (2.5, -6.0), (7.5, 8.0), (9.0, 1.5), (-6.5, 4.0)]
            .stepped(Stepped::After);

        Chart::new(ScatterController::new((first, second)).show_line(true))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Chart.js's sample bar chart: votes for six colours, labelled on a category scale.
    pub fn labelled_bar_chart() -> impl Widget {
        let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

        Chart::new(BarController::new(votes))
            .scale(
                ScaleId::X,
                CategoryScale::new(Axis::Horizontal)
                    .labels(["Red", "Blue", "Yellow", "Green", "Purple", "Orange"])
                    .grid_offset(true),
            )
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// A chart with only a `BarController` for `datasets` on the default scales: no lines or
    /// points on top.
    pub fn bar_chart<I, M, D>(datasets: I) -> impl Widget
    where
        I: IntoSequence<dyn DataSet<Scalar, Scalar, Scalar>, M, Sequence=D>,
        D: DataSetSequence<Scalar, Scalar, Scalar>,
    {
        Chart::new(BarController::new(datasets))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue and costs as grouped bars, and profit (revenue minus costs, negative in one month)
    /// as a red line over them, all on the default scales.
    pub fn bars_and_line_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0];
        let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0, 25.0, 35.0]
            .color(DataColor::Color(RED));

        Chart::new((
            BarController::new((revenue, costs)),
            LineController::new(profit),
        ))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Two series of temperature readings, one measured in °C on the default x scale and one in
    /// °F on a second x scale, which is also at the bottom and so stacks below the default one.
    /// The °F scale draws no grid lines, so only the °C scale's lines cross the chart area.
    pub fn stacked_horizontal_scales_chart() -> impl Widget {
        let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
        let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
            .x_scale(FAHRENHEIT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(celsius).show_line(true),
            ScatterController::new(fahrenheit).show_line(true),
        ))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// `two_y_scales_chart` plus units sold on a third scale, also on the right, added after
    /// `percent` so it stacks outside it. Both right-hand scales draw no grid lines, so only Y's
    /// lines cross the chart area; `units` also draws no tick marks, only labels and its border.
    pub fn stacked_scales_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
            .y_scale(PERCENT)
            .color(DataColor::Color(GREEN));
        let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0, 3400.0, 3900.0]
            .y_scale(UNITS)
            .color(DataColor::Color(RED));

        Chart::new((
            LineController::new(revenue),
            LineController::new(margin),
            LineController::new(units),
        ))
            .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
            .scale(UNITS, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false).tick_marks(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue on the default scales and a margin (as a fraction) on a `percent` scale on the
    /// right, in two controllers.
    pub fn two_y_scales_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
            .y_scale(PERCENT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            LineController::new(revenue),
            LineController::new(margin),
        ))
            .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// The chart from `carbide/examples/chart/chart.rs`, with the example's `data7` and `data8`.
    pub fn example_chart(stepped: LocalState<Stepped>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].stepped(stepped);
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(LineController::new((data7, data8)))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }
}
