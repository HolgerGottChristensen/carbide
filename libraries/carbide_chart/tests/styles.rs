//! How charts look: the axis and series styles set on a chart or any of its ancestors.

use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use support::{custom_styled_chart, debug_styled_chart};

/// Revenue and costs as bars with a filled profit line, in `DebugStyle` for both axes and
/// series. Pins down that `DebugStyle` keeps the colours charts were first drawn with: a yellow
/// grid, tick marks and labels, thick red and green borders, and a white, green and red palette.
#[test]
fn debug_style() {
    let mut harness = Harness::new(debug_styled_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("debug_style");
}

/// The chart of `debug_style` in styles of the test's own: an axis style with a blue grid, white
/// labels and thick orange borders, and a series style with a palette of two colours and more
/// opaque areas. Pins down that custom styles reach every part of the chart, set on an ancestor
/// rather than the chart itself, and that the palette wraps around after its two colours.
#[test]
fn custom_styles() {
    let mut harness = Harness::new(custom_styled_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("custom_styles");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, AxisStyle, BarController, Chart, ChartStyleExt, DatasetExt, Fill, LineController, SeriesStyle};
    use carbide_core::color::{Color, BLUE, GREEN, ORANGE, PURPLE, WHITE};
    use carbide_core::draw::{DebugStyle, Scalar};
    use carbide_core::environment::Environment;
    use carbide_core::widget::{VStack, Widget};
    use crate::common::Framed;

    /// Revenue, costs and an extra month's forecast as bars, and profit as a line filled to
    /// zero, on the default scales.
    fn revenue_chart() -> impl Widget {
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

    pub fn debug_styled_chart() -> impl Widget {
        revenue_chart()
            .axis_style(DebugStyle)
            .series_style(DebugStyle)
    }

    /// The styles are set on a stack around the chart, an ancestor, not on the chart itself.
    pub fn custom_styled_chart() -> impl Widget {
        VStack::new(revenue_chart())
            .axis_style(BlueprintStyle)
            .series_style(TwoTonedStyle)
    }

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
}
