//! Error bars: whiskers from each point's low to its high value, on bars, lines and scatter
//! points, moving with stacks.

use carbide_chart::{Value, BarController, Chart, DataColor, DataPoint, DatasetExt, Estimate, Interpolation, LineController, Range, RangeDataSetExt, RangePoint, ScatterController, Spread, StackId};
use carbide_core::color::ORANGE;
use carbide_core::draw::{Dimension, Scalar};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Two grouped datasets of measurements, each bar with error bars. Pins down that each whisker
/// runs through the middle of its own bar, in its own share of the slot, with caps at both ends,
/// drawn over the bars, and that the y scale reaches the highest whisker.
#[test]
fn on_grouped_bars() {
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

    harness.snapshot("on_grouped_bars");
}

/// Two stacked datasets with error bars, in months with positive and with negative values. Pins
/// down that each whisker moves with its stacked bar: it keeps its distance from the dataset's
/// value, around where its bar ends, upwards above zero and downwards below it, and that the y
/// scale reaches the highest and lowest whisker.
#[test]
fn on_stacked_bars() {
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

    harness.snapshot("on_stacked_bars");
}

/// Two lines with error bars, one of them monotone and missing a value. Pins down that each
/// whisker runs through its point from the low to the high value, with caps about as wide as a
/// point, drawn over the lines and points, and that the missing value has no whisker.
#[test]
fn on_lines() {
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

    harness.snapshot("on_lines");
}

/// Two stacked lines with error bars. Pins down that the upper line's whiskers move with it,
/// around where it is drawn at the running total, keeping their distance from its own value.
#[test]
fn on_stacked_lines() {
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

    harness.snapshot("on_stacked_lines");
}

/// Lying floating bars beside lying bars with error bars, in one controller. Pins down that both
/// marks follow the controller's orientation: the ranges and whiskers run along the x scale, the
/// whiskers' caps across the bars, and each dataset keeps its share of the slot.
#[test]
fn horizontal() {
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

    let chart = Chart::new(BarController::new((ranges.range(), measured.error_bars())).horizontal(true)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal");
}

/// Marks with missing values: bars with error bars where one value, one low and one high are
/// missing, beside floating bars where one low and one high are missing. Pins down that a missing
/// value has neither a bar nor a whisker, that a missing low or high takes away only the whisker,
/// and that a floating bar missing either end isn't drawn, leaving its place in the slot empty.
#[test]
fn with_missing_values() {
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

    harness.snapshot("with_missing_values");
}

/// Two scatter datasets with error bars, one of them missing a high. Pins down that each whisker
/// runs upright through its point from the low to the high y, with caps about as wide as a point,
/// drawn over every dataset's points, that the y scale reaches the highest and lowest whisker, and
/// that a point missing its high keeps its point but has no whisker.
#[test]
fn on_scatter_points() {
    let first: Vec<Estimate<f64, Option<f64>>> = vec![
        Estimate { x: 1.0, y: Some(12.0), low: Some(9.0), high: Some(14.0) },
        Estimate { x: 2.5, y: Some(17.0), low: Some(14.0), high: Some(21.0) },
        Estimate { x: 4.0, y: Some(15.0), low: Some(13.0), high: None },
        Estimate { x: 6.0, y: Some(22.0), low: Some(18.0), high: Some(27.0) },
    ];
    let second = vec![
        Estimate { x: 1.5, y: 6.0, low: 3.0, high: 8.0 },
        Estimate { x: 3.0, y: 10.0, low: 8.0, high: 13.0 },
        Estimate { x: 5.0, y: 8.0, low: 6.0, high: 9.0 },
    ];

    let chart = Chart::new(ScatterController::new((first.error_bars(), second.error_bars()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_scatter_points");
}

/// Error bars in colours of their own: bars whose whiskers are a fixed colour, and a line over
/// them whose whiskers are the line's own colour. Pins down that `whisker_color` replaces the
/// label colour for that dataset only, and that the dataset's colour (`DataColor::ByDataSet`) is
/// the colour its line is drawn in.
#[test]
fn in_colours_of_their_own() {
    let rainfall = vec![
        Estimate { x: 0.0, y: 40.0, low: 32.0, high: 47.0 },
        Estimate { x: 1.0, y: 55.0, low: 48.0, high: 63.0 },
        Estimate { x: 2.0, y: 35.0, low: 30.0, high: 41.0 },
        Estimate { x: 3.0, y: 60.0, low: 51.0, high: 66.0 },
    ];
    let forecast = vec![
        Estimate { x: 0.0, y: 45.0, low: 41.0, high: 50.0 },
        Estimate { x: 1.0, y: 50.0, low: 45.0, high: 56.0 },
        Estimate { x: 2.0, y: 42.0, low: 37.0, high: 46.0 },
        Estimate { x: 3.0, y: 52.0, low: 47.0, high: 58.0 },
    ];

    let chart = Chart::new((
        BarController::new(rainfall.error_bars().whisker_color(DataColor::Color(ORANGE))),
        LineController::new(forecast.error_bars().whisker_color(DataColor::ByDataSet)).palette_offset(1),
    )).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("in_colours_of_their_own");
}

/// Two scatter datasets with error bars in colours of their own: one in a fixed colour, the other
/// in the dataset's own colour. Pins down that `whisker_color` colours the whiskers on scatter
/// points as it does on bars and lines, for that dataset only.
#[test]
fn on_scatter_points_in_colours_of_their_own() {
    let first = vec![
        Estimate { x: 1.0, y: 12.0, low: 9.0, high: 14.0 },
        Estimate { x: 2.5, y: 17.0, low: 14.0, high: 21.0 },
        Estimate { x: 4.0, y: 15.0, low: 13.0, high: 18.0 },
    ];
    let second = vec![
        Estimate { x: 1.5, y: 6.0, low: 3.0, high: 8.0 },
        Estimate { x: 3.0, y: 10.0, low: 8.0, high: 13.0 },
        Estimate { x: 5.0, y: 8.0, low: 6.0, high: 9.0 },
    ];

    let chart = Chart::new(ScatterController::new((
        first.error_bars().whisker_color(DataColor::Color(ORANGE)),
        second.error_bars().whisker_color(DataColor::ByDataSet),
    ))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_scatter_points_in_colours_of_their_own");
}

/// Scatter points with error bars along both axes, as `Spread`s: one point with both intervals,
/// one missing its x interval's high and one missing its y interval's low. Pins down that a level
/// whisker runs from each point's x low to its x high, with upright caps, beside the upright one,
/// that each whisker is drawn only where both its ends are there, and that both scales reach the
/// whiskers' ends.
#[test]
fn on_scatter_points_along_both_axes() {
    let first: Vec<Spread<Option<f64>, Option<f64>>> = vec![
        Spread { x: Some(2.0), y: Some(12.0), x_low: Some(1.4), x_high: Some(2.9), low: Some(9.0), high: Some(14.0) },
        Spread { x: Some(4.0), y: Some(17.0), x_low: Some(3.3), x_high: None, low: Some(14.0), high: Some(21.0) },
        Spread { x: Some(6.0), y: Some(15.0), x_low: Some(5.2), x_high: Some(7.1), low: None, high: Some(18.0) },
    ];
    let second = vec![
        Spread { x: 3.0, y: 6.0, x_low: 2.5, x_high: 3.4, low: 4.0, high: 8.0 },
        Spread { x: 5.0, y: 9.0, x_low: 4.1, x_high: 6.2, low: 8.0, high: 11.0 },
    ];

    let chart = Chart::new(ScatterController::new((first.error_bars(), second.error_bars().whisker_color(DataColor::ByDataSet)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_scatter_points_along_both_axes");
}

/// Error bars on scatter points of a type of your own, which gives an interval along x for some
/// points, along y for others, and neither for the rest. Pins down that a type of your own makes
/// each whisker optional through `RangePoint`, returning `None` for an end it doesn't have, and
/// that only the whiskers it gives are drawn.
#[test]
fn on_scatter_points_of_a_type_of_your_own() {
    #[derive(Clone, Debug)]
    enum Error {
        AlongX(f64),
        AlongY(f64),
        None,
    }

    #[derive(Clone, Debug)]
    struct Reading {
        time: f64,
        level: f64,
        error: Error,
    }

    impl DataPoint for Reading {
        fn x(&self) -> Option<Value<'_>> {
            Some(Value::Number(self.time))
        }

        fn y(&self) -> Option<Value<'_>> {
            Some(Value::Number(self.level))
        }
    }

    impl RangePoint for Reading {
        fn low(&self) -> Option<Scalar> {
            match self.error {
                Error::AlongY(error) => Some(self.level - error),
                _ => None,
            }
        }

        fn high(&self) -> Option<Scalar> {
            match self.error {
                Error::AlongY(error) => Some(self.level + error),
                _ => None,
            }
        }

        fn x_low(&self) -> Option<Scalar> {
            match self.error {
                Error::AlongX(error) => Some(self.time - error),
                _ => None,
            }
        }

        fn x_high(&self) -> Option<Scalar> {
            match self.error {
                Error::AlongX(error) => Some(self.time + error),
                _ => None,
            }
        }
    }

    let readings = vec![
        Reading { time: 1.0, level: 4.0, error: Error::AlongY(1.5) },
        Reading { time: 2.5, level: 7.0, error: Error::AlongX(0.8) },
        Reading { time: 4.0, level: 5.5, error: Error::None },
        Reading { time: 5.5, level: 9.0, error: Error::AlongY(2.0) },
        Reading { time: 7.0, level: 6.0, error: Error::AlongX(1.2) },
    ];

    let chart = Chart::new(ScatterController::new(readings.error_bars())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_scatter_points_of_a_type_of_your_own");
}
