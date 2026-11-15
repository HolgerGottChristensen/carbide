//! What surrounds the chart area: axis titles, the chart title, the legend and the padding.

use carbide_chart::LegendPosition;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use support::{axis_titles_chart, everything_chart, legend_chart, title_and_padding_chart};

/// A title on a scale on each side: °C along the bottom and humidity on the left for one dataset,
/// °F along the top and pressure on the right for the other. Pins down that a horizontal axis's
/// title is centred under (or over) its labels, in its band, which grows to fit it; and that a
/// vertical axis's title, as text can't be rotated, is drawn horizontally above the chart area,
/// starting at its band's outer edge, with the chart area moved down to make room.
#[test]
fn axis_titles() {
    let mut harness = Harness::new(axis_titles_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("axis_titles");
}

/// A bar chart with a title and a padding of 10 instead of 30. Pins down that the title is centred
/// along the top, inside the padding, and that the chart area gives up the title's height and a
/// gap, and reaches closer to the chart's edges.
#[test]
fn chart_title_and_padding() {
    let mut harness = Harness::new(title_and_padding_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("chart_title_and_padding");
}

/// Revenue and costs as named bars, a named filled profit line, and an unnamed target line, with
/// the legend on each side in turn, its position a state. Pins down that the legend shows each
/// named dataset, in order, with a marker in its colour shaped after what it is (squares for the
/// bars and the filled line), leaves out the unnamed one, and takes its space from the chart area:
/// a row across the top or bottom, centred, or a column down the left or right, centred.
#[test]
fn legend_positions() {
    let mut position = LocalState::new(LegendPosition::Top);
    let mut harness = Harness::new(legend_chart(position.clone()), Dimension::new(600.0, 500.0));

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
/// outside in: the title above the legend, the legend above the row of vertical axis titles, and
/// the chart area below that, with the horizontal axis's title under its labels.
#[test]
fn title_legend_and_axis_titles_together() {
    let mut harness = Harness::new(everything_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("title_legend_and_axis_titles_together");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, AxisPosition, BarController, Chart, DatasetExt, Fill, LegendPosition, LineController, LinearScale, ScaleId, ScatterController};
    use carbide_core::state::LocalState;
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    const FAHRENHEIT: ScaleId = ScaleId::new("fahrenheit");
    const PRESSURE: ScaleId = ScaleId::new("pressure");

    /// Humidity by °C on the default scales, and pressure by °F on scales at the top and on the
    /// right, every scale with a title.
    pub fn axis_titles_chart() -> impl Widget {
        let humidity = vec![(0.0, 82.0), (20.0, 70.0), (40.0, 64.0), (60.0, 55.0), (80.0, 48.0), (100.0, 40.0)];
        let pressure = vec![(32.0, 1002.0), (68.0, 1008.0), (104.0, 1011.0), (140.0, 1016.0), (176.0, 1021.0), (212.0, 1026.0)]
            .x_scale(FAHRENHEIT)
            .y_scale(PRESSURE);

        Chart::new((
            ScatterController::new(humidity).show_line(true),
            ScatterController::new(pressure).show_line(true).palette_offset(1),
        ))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).title("Temperature (°C)"))
            .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Humidity (%)"))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false).title("Temperature (°F)"))
            .scale(PRESSURE, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false).title("Pressure (hPa)"))
            .framed()
    }

    /// Monthly revenue as bars, with a title and a padding of 10.
    pub fn title_and_padding_chart() -> impl Widget {
        Chart::new(BarController::new(vec![120.0, 135.0, 150.0, 110.0, 160.0]))
            .title("Monthly revenue")
            .padding(10.0)
            .framed()
    }

    /// Revenue and costs as named bars, profit as a named filled line, and a target line without a
    /// name, with the legend on `position`.
    pub fn legend_chart(position: LocalState<LegendPosition>) -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0].label("Revenue");
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0].label("Costs");
        let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0].fill(Fill::Origin).label("Profit");
        let target = vec![130.0, 130.0, 140.0, 140.0, 150.0];

        Chart::new((
            BarController::new((revenue, costs)),
            LineController::new((profit, target)).palette_offset(2),
        ))
            .legend(position)
            .framed()
    }

    /// The chart of `legend_chart` with the legend on top, a title, and titles on both axes.
    pub fn everything_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0].label("Revenue");
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0].label("Costs");
        let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0].fill(Fill::Origin).label("Profit");

        Chart::new((
            BarController::new((revenue, costs)),
            LineController::new(profit).palette_offset(2),
        ))
            .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Amount (€)"))
            .title("Revenue, costs and profit")
            .legend(LegendPosition::Top)
            .framed()
    }
}
