//! What surrounds the chart area: axis titles, the chart title, the legend and the padding.

use carbide_chart::{Axis, AxisPosition, BarController, Chart, DatasetExt, Fill, LegendPosition, LineController, LinearScale, ScaleId, ScatterController};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// A title on a scale on each side: °C along the bottom and humidity on the left for one dataset,
/// °F along the top and pressure on the right for the other. Pins down that every axis's title is
/// centred along the chart area at the outer edge of its band, which grows to fit it; and that a
/// vertical axis's title is turned along the axis, reading upwards on the left and downwards on
/// the right.
#[test]
fn axis_titles() {
    const PRESSURE: ScaleId = ScaleId::new("pressure");
    const FAHRENHEIT: ScaleId = ScaleId::new("fahrenheit");
    // Humidity by °C on the default scales, and pressure by °F on scales at the top and on the
    // right, every scale with a title.
    let humidity = vec![(0.0, 82.0), (20.0, 70.0), (40.0, 64.0), (60.0, 55.0), (80.0, 48.0), (100.0, 40.0)];
    let pressure = vec![(32.0, 1002.0), (68.0, 1008.0), (104.0, 1011.0), (140.0, 1016.0), (176.0, 1021.0), (212.0, 1026.0)]
        .x_scale(FAHRENHEIT)
        .y_scale(PRESSURE);

    let chart = Chart::new((
        ScatterController::new(humidity).show_line(true),
        ScatterController::new(pressure).show_line(true).palette_offset(1),
    ))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).title("Temperature (°C)"))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Humidity (%)"))
        .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false).title("Temperature (°F)"))
        .scale(PRESSURE, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false).title("Pressure (hPa)"))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("axis_titles");
}

/// A bar chart with a title and a padding of 10 instead of 30. Pins down that the title is centred
/// along the top, inside the padding, and that the chart area gives up the title's height and a
/// gap, and reaches closer to the chart's edges.
#[test]
fn chart_title_and_padding() {
    // Monthly revenue as bars, with a title and a padding of 10.
    let chart = Chart::new(BarController::new(vec![120.0, 135.0, 150.0, 110.0, 160.0]))
        .title("Monthly revenue")
        .padding(10.0)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("chart_title_and_padding");
}

/// Revenue and costs as named bars, a named filled profit line, and an unnamed target line, with
/// the legend on each side in turn, its position a state. Pins down that the legend shows each
/// named dataset, in order, with a marker in its colour shaped after what it is (squares for the
/// bars and the filled line), leaves out the unnamed one, and takes its space from the chart area:
/// a row across the top or bottom, centred, or a column down the left or right, centred.
#[test]
fn legend_positions() {
    // Revenue and costs as named bars, profit as a named filled line, and a target line without a
    // name, with the legend on `position`.
    let mut position = LocalState::new(LegendPosition::Top);
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0].label("Revenue");
    let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0].label("Costs");
    let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0].fill(Fill::Origin).label("Profit");
    let target = vec![130.0, 130.0, 140.0, 140.0, 150.0];

    let chart = Chart::new((
        BarController::new((revenue, costs)),
        LineController::new((profit, target)).palette_offset(2),
    ))
        .legend(position.clone())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    for (side, name) in [
        (LegendPosition::Top, "top"),
        (LegendPosition::Bottom, "bottom"),
        (LegendPosition::Left, "left"),
        (LegendPosition::Right, "right"),
    ] {
        *position.value_mut() = side;
        harness.snapshot(name);
    }
}

/// A chart title, a legend on top, and titles on both axes together. Pins down the order from the
/// outside in: the title above the legend and the chart area below that, with the vertical axis's
/// title turned along it and the horizontal axis's labels under the chart area.
#[test]
fn title_legend_and_axis_titles_together() {
    // The chart of `legend_positions` without the target line, with the legend on top, a title,
    // and titles on both axes.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0].label("Revenue");
    let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0].label("Costs");
    let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0].fill(Fill::Origin).label("Profit");

    let chart = Chart::new((
        BarController::new((revenue, costs)),
        LineController::new(profit).palette_offset(2),
    ))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Amount (€)"))
        .title("Revenue, costs and profit")
        .legend(LegendPosition::Top)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("title_legend_and_axis_titles_together");
}

/// Two stacked scales with titles on every side: four datasets, each on its own x and y scale, an
/// inner and an outer scale at the bottom, the top, the left and the right. Pins down that each
/// title sits at the outer edge of its own band, centred along the chart area, so the inner
/// scale's title lies between the two scales' labels; that the vertical titles are turned, reading
/// upwards on the left and downwards on the right; and that every band grows by its title, so no
/// title overlaps the labels of the scale stacked outside it.
#[test]
fn stacked_axis_titles() {
    // Four datasets, each on its own pair of scales: the default ones (inner bottom and left), a
    // second pair stacked outside them, an inner pair on the top and right, and a second pair
    // stacked outside those. Every scale has a title; only the default scales draw grid lines.
    const BOTTOM_OUTER: ScaleId = ScaleId::new("bottom outer");
    const LEFT_OUTER: ScaleId = ScaleId::new("left outer");
    const TOP_INNER: ScaleId = ScaleId::new("top inner");
    const TOP_OUTER: ScaleId = ScaleId::new("top outer");
    const RIGHT_INNER: ScaleId = ScaleId::new("right inner");
    const RIGHT_OUTER: ScaleId = ScaleId::new("right outer");

    let inner = vec![(0.0, 2.0), (25.0, 6.0), (50.0, 4.0), (75.0, 8.0), (100.0, 7.0)];
    let outer = vec![(0.0, 300.0), (1.0, 500.0), (2.0, 450.0), (3.0, 700.0), (4.0, 650.0)]
        .x_scale(BOTTOM_OUTER)
        .y_scale(LEFT_OUTER);
    let top = vec![(-10.0, 0.2), (-5.0, 0.6), (0.0, 0.3), (5.0, 0.9), (10.0, 0.5)]
        .x_scale(TOP_INNER)
        .y_scale(RIGHT_INNER);
    let top_outer = vec![(1000.0, 40.0), (2000.0, 25.0), (3000.0, 55.0), (4000.0, 35.0), (5000.0, 60.0)]
        .x_scale(TOP_OUTER)
        .y_scale(RIGHT_OUTER);

    let secondary = |axis: Axis, position: AxisPosition, title: &'static str| {
        LinearScale::new(axis).position(position).grid_lines(false).title(title)
    };

    let chart = Chart::new((
        ScatterController::new(inner).show_line(true),
        ScatterController::new(outer).show_line(true).palette_offset(1),
        ScatterController::new(top).show_line(true).palette_offset(2),
        ScatterController::new(top_outer).show_line(true).palette_offset(3),
    ))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).title("Bottom inner (%)"))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Left inner (units)"))
        .scale(BOTTOM_OUTER, secondary(Axis::Horizontal, AxisPosition::Bottom, "Bottom outer (quarter)"))
        .scale(LEFT_OUTER, secondary(Axis::Vertical, AxisPosition::Left, "Left outer (€)"))
        .scale(TOP_INNER, secondary(Axis::Horizontal, AxisPosition::Top, "Top inner (°C)"))
        .scale(TOP_OUTER, secondary(Axis::Horizontal, AxisPosition::Top, "Top outer (m)"))
        .scale(RIGHT_INNER, secondary(Axis::Vertical, AxisPosition::Right, "Right inner (ratio)"))
        .scale(RIGHT_OUTER, secondary(Axis::Vertical, AxisPosition::Right, "Right outer (kg)"))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(700.0, 600.0));

    harness.snapshot("stacked_axis_titles");
}

