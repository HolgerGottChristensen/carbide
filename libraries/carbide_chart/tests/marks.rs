//! Marks: floating bars in place of bars and error bars on top of them, in a bar controller's
//! slots; bands in place of a line controller's lines; and bubbles in place of a scatter
//! controller's points.

use carbide_chart::{BarController, Bubble, Chart, DataColor, DataPoint, DatasetExt, Estimate, Interpolation, LineController, Range, RangeDataSetExt, ScatterController, SizedDataSetExt, SizedPoint, StackId, Stepped};
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

/// Two stacked datasets with error bars, in months with positive and with negative values. Pins
/// down that each whisker moves with its stacked bar: it keeps its distance from the dataset's
/// value, around where its bar ends, upwards above zero and downwards below it, and that the y
/// scale reaches the highest and lowest whisker.
#[test]
fn error_bars_on_stacked_bars() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![
        Estimate { x: 0.0, y: 10.0, low: 8.0, high: 12.0 },
        Estimate { x: 1.0, y: 14.0, low: 12.0, high: 15.0 },
        Estimate { x: 2.0, y: -8.0, low: -10.0, high: -7.0 },
    ];
    let labour = vec![
        Estimate { x: 0.0, y: 6.0, low: 4.0, high: 9.0 },
        Estimate { x: 1.0, y: 4.0, low: 3.0, high: 6.0 },
        Estimate { x: 2.0, y: -5.0, low: -7.0, high: -4.0 },
    ];

    let chart = Chart::new(BarController::new((materials.error_bars().stack(COSTS), labour.error_bars().stack(COSTS)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("error_bars_on_stacked_bars");
}

/// Marks with missing values: bars with error bars where one value, one low and one high are
/// missing, beside floating bars where one low and one high are missing. Pins down that a missing
/// value has neither a bar nor a whisker, that a missing low or high takes away only the whisker,
/// and that a floating bar missing either end isn't drawn, leaving its place in the slot empty.
#[test]
fn marks_with_missing_values() {
    let measured: Vec<Estimate<f64, Option<f64>>> = vec![
        Estimate { x: 0.0, y: Some(10.0), low: Some(8.0), high: Some(13.0) },
        Estimate { x: 1.0, y: None, low: Some(9.0), high: Some(14.0) },
        Estimate { x: 2.0, y: Some(12.0), low: None, high: Some(15.0) },
        Estimate { x: 3.0, y: Some(7.0), low: Some(5.0), high: None },
    ];
    let ranges: Vec<Range<f64, Option<f64>>> = vec![
        Range { x: 0.0, low: Some(3.0), high: Some(9.0) },
        Range { x: 1.0, low: Some(4.0), high: Some(11.0) },
        Range { x: 2.0, low: None, high: Some(10.0) },
        Range { x: 3.0, low: Some(2.0), high: None },
    ];

    let chart = Chart::new(BarController::new((measured.error_bars(), ranges.range()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("marks_with_missing_values");
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

/// A month's temperature range as a band, with the month's average as a line through it, in one
/// line controller. Pins down that the band fills between the line through the lows and the line
/// through the highs, translucent, with both edges stroked; that it is drawn under the other
/// dataset's line; and that the y scale fits the band without reaching down to zero.
#[test]
fn range_band() {
    let temperatures = vec![
        Range { x: 0.0, low: 2.0, high: 9.0 },
        Range { x: 1.0, low: 3.0, high: 11.0 },
        Range { x: 2.0, low: 6.0, high: 15.0 },
        Range { x: 3.0, low: 9.0, high: 19.0 },
        Range { x: 4.0, low: 12.0, high: 22.0 },
        Range { x: 5.0, low: 11.0, high: 20.0 },
    ];
    let average = vec![5.5, 7.0, 10.5, 14.0, 17.0, 15.5];

    let chart = Chart::new(LineController::new((temperatures.range(), average))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("range_band");
}

/// A monotone band with a missing high in the middle, beside a stepped band. Pins down that a
/// band's edges follow its dataset's interpolation, both edges alike, and that a missing low or
/// high breaks the band into two, as a missing value breaks a line.
#[test]
fn range_bands_with_curves_and_a_gap() {
    let monotone: Vec<Range<f64, Option<f64>>> = vec![
        Range { x: 0.0, low: Some(10.0), high: Some(14.0) },
        Range { x: 1.0, low: Some(12.0), high: Some(18.0) },
        Range { x: 2.0, low: Some(11.0), high: Some(16.0) },
        Range { x: 3.0, low: Some(13.0), high: None },
        Range { x: 4.0, low: Some(15.0), high: Some(21.0) },
        Range { x: 5.0, low: Some(14.0), high: Some(19.0) },
        Range { x: 6.0, low: Some(17.0), high: Some(23.0) },
    ];
    let stepped = vec![
        Range { x: 0.0, low: 2.0, high: 5.0 },
        Range { x: 1.0, low: 3.0, high: 7.0 },
        Range { x: 2.0, low: 5.0, high: 6.0 },
        Range { x: 3.0, low: 4.0, high: 9.0 },
        Range { x: 4.0, low: 6.0, high: 8.0 },
        Range { x: 5.0, low: 3.0, high: 6.0 },
        Range { x: 6.0, low: 5.0, high: 10.0 },
    ];

    let chart = Chart::new(LineController::new((
        monotone.range().interpolation(Interpolation::Monotone),
        stepped.range().interpolation(Interpolation::Stepped(Stepped::Middle)),
    ))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("range_bands_with_curves_and_a_gap");
}

/// A lying band with a lying line through it. Pins down that the band follows the controller's
/// orientation, the categories running down the y scale and the band spreading along the x scale.
#[test]
fn horizontal_range_band() {
    let durations = vec![
        Range { x: 0.0, low: 3.0, high: 8.0 },
        Range { x: 1.0, low: 5.0, high: 12.0 },
        Range { x: 2.0, low: 4.0, high: 9.0 },
        Range { x: 3.0, low: 7.0, high: 15.0 },
    ];
    let median = vec![5.0, 8.0, 6.5, 11.0];

    let chart = Chart::new(LineController::new((durations.range(), median)).horizontal()).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_range_band");
}
