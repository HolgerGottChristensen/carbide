//! How charts look: the axis and series styles set on a chart or any of its ancestors, and
//! datasets coloured by point.

use carbide_chart::{Axis, AxisStyle, BarController, Chart, ChartStyleExt, DataColor, DatasetExt, Fill, LineController, SeriesStyle};
use carbide_core::color::{BLUE, Color, GREEN, ORANGE, PURPLE, WHITE};
use carbide_core::draw::{DebugStyle, Dimension, Scalar};
use carbide_core::environment::Environment;
use carbide_core::widget::VStack;
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::revenue_chart;

/// Revenue and costs as bars with a filled profit line, in `DebugStyle` for both axes and
/// series. Pins down that `DebugStyle` keeps the colours charts were first drawn with: a yellow
/// grid, tick marks and labels, thick red and green borders, and a white, green and red palette.
#[test]
fn debug() {
    let chart = revenue_chart()
        .axis_style(DebugStyle)
        .series_style(DebugStyle);

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("debug");
}

/// The chart of `debug_style` in styles of the test's own: an axis style with a blue grid, white
/// labels and thick orange borders, and a series style with a palette of two colours and more
/// opaque areas. Pins down that custom styles reach every part of the chart, set on an ancestor
/// rather than the chart itself, and that the palette wraps around after its two colours.
#[test]
fn custom() {
    /// A blue grid, white labels and tick marks, and thick orange borders.
    #[derive(Debug, Clone)]
    struct BlueprintStyle;

    impl AxisStyle for BlueprintStyle {
        fn grid_color(&self, _env: &Environment) -> Color {
            BLUE
        }

        fn tick_color(&self, _env: &Environment) -> Color {
            WHITE
        }

        fn label_color(&self, _env: &Environment) -> Color {
            WHITE
        }

        fn border_color(&self, _axis: Axis, _env: &Environment) -> Color {
            ORANGE
        }

        fn border_width(&self) -> Scalar {
            4.0
        }
    }

    /// Two colours, green and purple, and areas at 0.6 opacity.
    #[derive(Debug, Clone)]
    struct TwoTonedStyle;

    impl SeriesStyle for TwoTonedStyle {
        fn palette(&self, _env: &Environment) -> Vec<Color> {
            vec![GREEN, PURPLE]
        }

        fn fill_opacity(&self) -> f32 {
            0.6
        }
    }

    // The styles are set on a stack around the chart, an ancestor, not on the chart itself.
    let chart = VStack::new(revenue_chart())
        .axis_style(BlueprintStyle)
        .series_style(TwoTonedStyle);

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("custom");
}

/// One dataset coloured `ByPoint`: each bar takes the next palette colour by its index, instead
/// of the whole dataset taking one. Pins down that the colours follow the bars' order and wrap
/// around at the end of the palette: the default palette has nine colours, so the tenth bar is
/// blue again, like the first.
#[test]
fn bars_coloured_by_point() {
    let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0, 9.0, 14.0, 7.0, 11.0].color(DataColor::ByPoint);
    let chart = Chart::new(BarController::new(votes)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_coloured_by_point");
}

/// A filled line coloured `ByPoint`. Pins down that each point takes the palette colour for its
/// index, while the line and its area, drawn once for the whole dataset, keep the dataset's own
/// palette colour (blue, as the first dataset).
#[test]
fn points_coloured_by_point() {
    let values = vec![12.0, 19.0, 8.0, 15.0, 10.0, 17.0]
        .color(DataColor::ByPoint)
        .fill(Fill::Origin);

    let chart = Chart::new(LineController::new(values))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("points_coloured_by_point");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{BarController, Chart, DatasetExt, Fill, LineController};
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    /// Revenue, costs and an extra month's forecast as bars, and profit as a line filled to
    /// zero, on the default scales.
    pub fn revenue_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0];
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0];
        let forecast = vec![125.0, 140.0, 145.0, 120.0, 170.0];
        let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0].fill(Fill::Origin);

        Chart::new((
            BarController::new((revenue, costs, forecast)),
            LineController::new(profit),
        ))
            .framed()
    }
}
