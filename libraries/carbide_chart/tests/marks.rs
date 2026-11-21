//! Marks: floating bars, box plots, candlesticks and OHLC bars in place of bars and error bars on
//! top of them, in a bar controller's slots; bands in place of a line controller's lines; and
//! bubbles in place of a scatter controller's points.

use carbide_chart::{BarController, BoxDataSetExt, BoxStats, Bubble, Chart, DataColor, DataPoint, DatasetExt, Estimate, Interpolation, LegendPosition, LineController, Ohlc, OhlcDataSetExt, Range, RangeDataSetExt, ScatterController, SizedDataSetExt, SizedPoint, StackId, Stepped};
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

/// A straight band and a monotone band whose highs and lows cross twice, with an inverted colour.
/// Pins down that the parts where the line through the highs runs below the line through the lows
/// are filled in the inverted colour, split exactly where the lines cross, on straight and on
/// curved edges alike, while the edges keep the dataset's colour.
#[test]
fn inverted_range_bands() {
    let straight = vec![
        Range { x: 0.0, low: 10.0, high: 16.0 },
        Range { x: 1.0, low: 12.0, high: 15.0 },
        Range { x: 2.0, low: 14.0, high: 11.0 },
        Range { x: 3.0, low: 13.0, high: 9.0 },
        Range { x: 4.0, low: 11.0, high: 14.0 },
        Range { x: 5.0, low: 10.0, high: 17.0 },
    ];
    let curved = vec![
        Range { x: 0.0, low: 2.0, high: 7.0 },
        Range { x: 1.0, low: 4.0, high: 6.0 },
        Range { x: 2.0, low: 6.0, high: 3.0 },
        Range { x: 3.0, low: 5.0, high: 2.0 },
        Range { x: 4.0, low: 3.0, high: 6.0 },
        Range { x: 5.0, low: 2.0, high: 8.0 },
    ];

    let chart = Chart::new(LineController::new((
        straight.range().inverted_color(DataColor::Palette(3)),
        curved.range().inverted_color(DataColor::Palette(3)).interpolation(Interpolation::Monotone),
    ))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("inverted_range_bands");
}

/// Floating bars, some of whose highs are below their lows, with an inverted colour. Pins down
/// that only the inverted bars take the inverted colour, and that they span the same values as if
/// their ends were the other way round.
#[test]
fn inverted_floating_bars() {
    let changes = vec![
        Range { x: 0.0, low: 4.0, high: 10.0 },
        Range { x: 1.0, low: 10.0, high: 7.0 },
        Range { x: 2.0, low: 7.0, high: 13.0 },
        Range { x: 3.0, low: 13.0, high: 5.0 },
        Range { x: 4.0, low: 5.0, high: 9.0 },
    ];

    let chart = Chart::new(BarController::new(changes.range().inverted_color(DataColor::Palette(3)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("inverted_floating_bars");
}

/// Two grouped datasets of box plots over three categories. Pins down that each box runs from the
/// first to the third quartile with the median across it, that the whiskers reach the minimum and
/// the maximum and end in caps half the box wide, that the two datasets sit side by side in each
/// slot, and that the y scale fits the whiskers without reaching down to zero.
#[test]
fn box_plots() {
    let before = vec![
        BoxStats { x: 0.0, min: 12.0, q1: 18.0, median: 22.0, q3: 27.0, max: 35.0 },
        BoxStats { x: 1.0, min: 15.0, q1: 20.0, median: 25.0, q3: 29.0, max: 38.0 },
        BoxStats { x: 2.0, min: 10.0, q1: 16.0, median: 19.0, q3: 24.0, max: 31.0 },
    ];
    let after = vec![
        BoxStats { x: 0.0, min: 9.0, q1: 13.0, median: 16.0, q3: 20.0, max: 26.0 },
        BoxStats { x: 1.0, min: 11.0, q1: 14.0, median: 18.0, q3: 22.0, max: 30.0 },
        BoxStats { x: 2.0, min: 8.0, q1: 11.0, median: 13.0, q3: 17.0, max: 23.0 },
    ];

    let chart = Chart::new(BarController::new((before.box_plot(), after.box_plot()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("box_plots");
}

/// Lying box plots. Pins down that the boxes, medians, whiskers and caps follow the controller's
/// orientation: the categories down the y scale, the values along the x scale, the caps across.
#[test]
fn horizontal_box_plots() {
    let response_times = vec![
        BoxStats { x: 0.0, min: 80.0, q1: 120.0, median: 150.0, q3: 190.0, max: 260.0 },
        BoxStats { x: 1.0, min: 60.0, q1: 90.0, median: 110.0, q3: 140.0, max: 200.0 },
        BoxStats { x: 2.0, min: 100.0, q1: 160.0, median: 210.0, q3: 250.0, max: 340.0 },
    ];

    let chart = Chart::new(BarController::new(response_times.box_plot()).horizontal()).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_box_plots");
}

/// Eight trading days as candlesticks, rising and falling, with one flat day. Pins down that each
/// body runs from the open to the close and each wick from the low to the high, that rising days
/// are drawn in the style's rising colour (green) and falling ones in its falling colour (red), a
/// flat day included as falling and drawn as a line, and that the y scale fits the wicks.
#[test]
fn candlesticks() {
    let prices = vec![
        Ohlc { x: 0.0, open: 100.0, high: 104.0, low: 98.0, close: 103.0 },
        Ohlc { x: 1.0, open: 103.0, high: 106.0, low: 101.0, close: 102.0 },
        Ohlc { x: 2.0, open: 102.0, high: 103.0, low: 96.0, close: 97.0 },
        Ohlc { x: 3.0, open: 97.0, high: 99.0, low: 95.0, close: 98.5 },
        Ohlc { x: 4.0, open: 98.5, high: 101.0, low: 97.0, close: 98.5 },
        Ohlc { x: 5.0, open: 98.5, high: 105.0, low: 98.0, close: 104.5 },
        Ohlc { x: 6.0, open: 104.5, high: 108.0, low: 103.0, close: 107.0 },
        Ohlc { x: 7.0, open: 107.0, high: 107.5, low: 102.0, close: 103.5 },
    ];

    let chart = Chart::new(BarController::new(prices.candles())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("candlesticks");
}

/// The same trading days as OHLC bars, in colours of their own. Pins down that each bar is a line
/// from the low to the high with the open ticked to the left and the close to the right, and that
/// a dataset's own rising and falling colours replace the style's.
#[test]
fn ohlc_bars_in_colours_of_their_own() {
    let prices = vec![
        Ohlc { x: 0.0, open: 100.0, high: 104.0, low: 98.0, close: 103.0 },
        Ohlc { x: 1.0, open: 103.0, high: 106.0, low: 101.0, close: 102.0 },
        Ohlc { x: 2.0, open: 102.0, high: 103.0, low: 96.0, close: 97.0 },
        Ohlc { x: 3.0, open: 97.0, high: 99.0, low: 95.0, close: 98.5 },
        Ohlc { x: 4.0, open: 98.5, high: 101.0, low: 97.0, close: 98.5 },
        Ohlc { x: 5.0, open: 98.5, high: 105.0, low: 98.0, close: 104.5 },
        Ohlc { x: 6.0, open: 104.5, high: 108.0, low: 103.0, close: 107.0 },
        Ohlc { x: 7.0, open: 107.0, high: 107.5, low: 102.0, close: 103.5 },
    ];

    let chart = Chart::new(BarController::new(prices.ohlc().rising_color(DataColor::Palette(0)).falling_color(DataColor::Palette(2)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("ohlc_bars_in_colours_of_their_own");
}

/// Candlesticks with a moving average as a line over them, both named, with the legend on top.
/// Pins down that the candlesticks' legend entry is a square split into the rising colour on the
/// left and the falling colour on the right, beside the line's entry in its own colour.
#[test]
fn candles_in_the_legend() {
    let prices = vec![
        Ohlc { x: 0.0, open: 100.0, high: 104.0, low: 98.0, close: 103.0 },
        Ohlc { x: 1.0, open: 103.0, high: 106.0, low: 101.0, close: 102.0 },
        Ohlc { x: 2.0, open: 102.0, high: 103.0, low: 96.0, close: 97.0 },
        Ohlc { x: 3.0, open: 97.0, high: 99.0, low: 95.0, close: 98.5 },
        Ohlc { x: 4.0, open: 98.5, high: 105.0, low: 98.0, close: 104.5 },
        Ohlc { x: 5.0, open: 104.5, high: 108.0, low: 103.0, close: 107.0 },
    ];
    let average = vec![101.0, 102.0, 100.7, 99.2, 100.0, 103.3];

    let chart = Chart::new((
        BarController::new(prices.candles().label("Price")),
        LineController::new(average.label("Average")).palette_offset(4),
    ))
        .legend(LegendPosition::Top)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("candles_in_the_legend");
}

/// Two lines with error bars, one of them monotone and missing a value. Pins down that each
/// whisker runs through its point from the low to the high value, with caps about as wide as a
/// point, drawn over the lines and points, and that the missing value has no whisker.
#[test]
fn error_bars_on_lines() {
    let first: Vec<Estimate<f64, Option<f64>>> = vec![
        Estimate { x: 0.0, y: Some(12.0), low: Some(10.0), high: Some(15.0) },
        Estimate { x: 1.0, y: Some(18.0), low: Some(15.0), high: Some(20.0) },
        Estimate { x: 2.0, y: None, low: Some(14.0), high: Some(19.0) },
        Estimate { x: 3.0, y: Some(21.0), low: Some(18.0), high: Some(25.0) },
        Estimate { x: 4.0, y: Some(17.0), low: Some(15.0), high: Some(19.0) },
    ];
    let second = vec![
        Estimate { x: 0.0, y: 6.0, low: 4.0, high: 7.0 },
        Estimate { x: 1.0, y: 9.0, low: 7.0, high: 12.0 },
        Estimate { x: 2.0, y: 8.0, low: 6.0, high: 9.0 },
        Estimate { x: 3.0, y: 11.0, low: 9.0, high: 14.0 },
        Estimate { x: 4.0, y: 13.0, low: 11.0, high: 15.0 },
    ];

    let chart = Chart::new(LineController::new((first.error_bars(), second.error_bars().interpolation(Interpolation::Monotone)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("error_bars_on_lines");
}

/// Two stacked lines with error bars. Pins down that the upper line's whiskers move with it,
/// around where it is drawn at the running total, keeping their distance from its own value.
#[test]
fn error_bars_on_stacked_lines() {
    const ALL: StackId = StackId::new("all");

    let base = vec![
        Estimate { x: 0.0, y: 10.0, low: 8.0, high: 11.0 },
        Estimate { x: 1.0, y: 12.0, low: 10.0, high: 14.0 },
        Estimate { x: 2.0, y: 11.0, low: 10.0, high: 13.0 },
        Estimate { x: 3.0, y: 14.0, low: 12.0, high: 15.0 },
    ];
    let extra = vec![
        Estimate { x: 0.0, y: 5.0, low: 4.0, high: 7.0 },
        Estimate { x: 1.0, y: 6.0, low: 4.0, high: 8.0 },
        Estimate { x: 2.0, y: 8.0, low: 6.0, high: 9.0 },
        Estimate { x: 3.0, y: 7.0, low: 5.0, high: 10.0 },
    ];

    let chart = Chart::new(LineController::new((base.error_bars().stack(ALL), extra.error_bars().stack(ALL)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("error_bars_on_stacked_lines");
}
