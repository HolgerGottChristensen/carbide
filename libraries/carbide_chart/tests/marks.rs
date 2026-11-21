//! Marks: floating bars in place of bars and error bars on top of them, in a bar controller's
//! slots, and bubbles in place of a scatter controller's points.

use carbide_chart::{BarController, Bubble, Chart, DataColor, DataPoint, Estimate, Range, RangeDataSetExt, ScatterController, SizedDataSetExt, SizedPoint};
use carbide_core::draw::{Dimension, Scalar};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// A month's lowest and highest temperature as a floating bar. Pins down that each bar spans from
/// its low to its high value instead of growing from zero, and that the y scale fits the ranges
/// without reaching down to zero.
#[test]
fn floating_bars() {
    let temperatures = vec![
        Range { x: 0.0, low: 4.0, high: 12.0 },
        Range { x: 1.0, low: 6.0, high: 15.0 },
        Range { x: 2.0, low: 9.0, high: 19.0 },
        Range { x: 3.0, low: 11.0, high: 23.0 },
        Range { x: 4.0, low: 10.0, high: 21.0 },
    ];

    let chart = Chart::new(BarController::new(temperatures.range())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("floating_bars");
}

/// Two grouped datasets of measurements, each bar with error bars. Pins down that each whisker
/// runs through the middle of its own bar, in its own share of the slot, with caps at both ends,
/// drawn over the bars, and that the y scale reaches the highest whisker.
#[test]
fn grouped_bars_with_error_bars() {
    let control = vec![
        Estimate { x: 0.0, y: 12.0, low: 10.0, high: 15.0 },
        Estimate { x: 1.0, y: 18.0, low: 14.0, high: 21.0 },
        Estimate { x: 2.0, y: 9.0, low: 8.0, high: 11.0 },
    ];
    let treated = vec![
        Estimate { x: 0.0, y: 15.0, low: 12.0, high: 17.0 },
        Estimate { x: 1.0, y: 24.0, low: 20.0, high: 29.0 },
        Estimate { x: 2.0, y: 13.0, low: 9.0, high: 16.0 },
    ];

    let chart = Chart::new(BarController::new((control.error_bars(), treated.error_bars()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("grouped_bars_with_error_bars");
}

/// Lying floating bars beside lying bars with error bars, in one controller. Pins down that both
/// marks follow the controller's orientation: the ranges and whiskers run along the x scale, the
/// whiskers' caps across the bars, and each dataset keeps its share of the slot.
#[test]
fn horizontal_marks() {
    let ranges = vec![
        Range { x: 0.0, low: 3.0, high: 9.0 },
        Range { x: 1.0, low: 5.0, high: 14.0 },
        Range { x: 2.0, low: 2.0, high: 7.0 },
    ];
    let measured = vec![
        Estimate { x: 0.0, y: 8.0, low: 6.0, high: 11.0 },
        Estimate { x: 1.0, y: 12.0, low: 10.0, high: 16.0 },
        Estimate { x: 2.0, y: 5.0, low: 4.0, high: 7.0 },
    ];

    let chart = Chart::new(BarController::new((ranges.range(), measured.error_bars())).horizontal()).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_marks");
}

/// Countries as bubbles: income along x, life expectancy up y, and population as the bubble's
/// size, in two datasets for two continents. Pins down that a bubble's area is proportional to its
/// size, with the largest size of either dataset at the largest radius, so bubbles compare across
/// datasets; that the bubbles are translucent with an outline, so overlapping ones show through;
/// and that the scales fit the bubbles' centres.
#[test]
fn bubbles() {
    let first = vec![
        Bubble { x: 12.0, y: 71.0, size: 140.0 },
        Bubble { x: 25.0, y: 78.0, size: 35.0 },
        Bubble { x: 40.0, y: 81.0, size: 8.0 },
        Bubble { x: 18.0, y: 74.0, size: 60.0 },
    ];
    let second = vec![
        Bubble { x: 45.0, y: 82.0, size: 10.0 },
        Bubble { x: 30.0, y: 80.0, size: 20.0 },
        Bubble { x: 22.0, y: 76.0, size: 2.0 },
    ];

    let chart = Chart::new(ScatterController::new((first.bubbles(), second.bubbles()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles");
}

/// Bubbles with a line through them in order. Pins down that the line is drawn under the bubbles,
/// through their centres, and that the bubbles replace the points the line would otherwise have.
#[test]
fn bubbles_with_a_line() {
    let visitors = vec![
        Bubble { x: 1.0, y: 120.0, size: 30.0 },
        Bubble { x: 2.0, y: 180.0, size: 55.0 },
        Bubble { x: 3.0, y: 150.0, size: 20.0 },
        Bubble { x: 4.0, y: 260.0, size: 90.0 },
        Bubble { x: 5.0, y: 230.0, size: 45.0 },
    ];

    let chart = Chart::new(ScatterController::new(visitors.bubbles()).show_line(true)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles_with_a_line");
}

/// A dataset of bubbles beside a dataset of plain points in one scatter controller. Pins down
/// that only the dataset with the mark draws bubbles, and the plain one keeps its points.
#[test]
fn bubbles_beside_points() {
    let sized = vec![(2.0, 4.0, 10.0), (4.0, 7.0, 40.0), (6.0, 5.0, 25.0)];
    let plain = vec![(1.0, 2.0), (3.0, 6.0), (5.0, 3.0), (7.0, 8.0)];

    let chart = Chart::new(ScatterController::new((sized.bubbles(), plain))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles_beside_points");
}

/// Sizes at the edges of what a bubble can show, on a radius range of 6 to 24 pixels: the
/// largest size at 24, a quarter of it at 12, a tiny size and a zero size at the smallest radius,
/// and a negative and a missing size without a bubble. Pins down the size scale's limits, and that
/// the points without a bubble are left out of the x scale, which ends at the last bubble.
#[test]
fn bubble_sizes_at_the_limits() {
    let sizes: Vec<Bubble<f64, f64, Option<f64>>> = vec![
        Bubble { x: 1.0, y: 1.0, size: Some(100.0) },
        Bubble { x: 2.0, y: 1.0, size: Some(25.0) },
        Bubble { x: 3.0, y: 1.0, size: Some(0.5) },
        Bubble { x: 4.0, y: 1.0, size: Some(0.0) },
        Bubble { x: 5.0, y: 1.0, size: Some(-20.0) },
        Bubble { x: 6.0, y: 1.0, size: None },
    ];

    let chart = Chart::new(ScatterController::new(sizes.bubbles()).radius_range(6.0, 24.0)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubble_sizes_at_the_limits");
}

/// Large bubbles at the ends of both scales. The scales fit the bubbles' centres, not their
/// radii, so the bubbles spill past the chart area onto the axes. Pins down that overflow, which
/// this snapshot should show changing once edge bubbles are kept inside the chart area.
#[test]
fn bubbles_at_the_edge() {
    let corners = vec![
        Bubble { x: 0.0, y: 0.0, size: 100.0 },
        Bubble { x: 10.0, y: 0.0, size: 60.0 },
        Bubble { x: 0.0, y: 10.0, size: 60.0 },
        Bubble { x: 10.0, y: 10.0, size: 100.0 },
        Bubble { x: 5.0, y: 5.0, size: 20.0 },
    ];

    let chart = Chart::new(ScatterController::new(corners.bubbles())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles_at_the_edge");
}

/// A type of your own drawn as bubbles: a country's income along x, life expectancy up y, its
/// population as the bubble's size, and its continent picking the palette colour. Implementing
/// `DataPoint` and `SizedPoint` is all it takes for a `Vec` of them to have `.bubbles()`. Pins
/// down that the type's own values and colours reach the chart.
#[test]
fn bubbles_from_a_type_of_your_own() {
    #[derive(Clone, Debug)]
    struct Country {
        income: Scalar,
        life_expectancy: Scalar,
        population: Scalar,
        continent: usize,
    }

    impl DataPoint for Country {
        fn x(&self) -> Option<Scalar> {
            Some(self.income)
        }

        fn y(&self) -> Option<Scalar> {
            Some(self.life_expectancy)
        }

        fn color(&self) -> DataColor {
            DataColor::Palette(self.continent)
        }
    }

    impl SizedPoint for Country {
        fn size(&self) -> Option<Scalar> {
            Some(self.population)
        }
    }

    let countries = vec![
        Country { income: 12.0, life_expectancy: 71.0, population: 140.0, continent: 0 },
        Country { income: 25.0, life_expectancy: 78.0, population: 35.0, continent: 0 },
        Country { income: 45.0, life_expectancy: 82.0, population: 10.0, continent: 1 },
        Country { income: 30.0, life_expectancy: 80.0, population: 60.0, continent: 1 },
        Country { income: 6.0, life_expectancy: 63.0, population: 90.0, continent: 2 },
    ];

    let chart = Chart::new(ScatterController::new(countries.bubbles())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles_from_a_type_of_your_own");
}
