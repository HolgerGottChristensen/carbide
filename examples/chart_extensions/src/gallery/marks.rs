//! Marks drawn for each point instead of, or on top of, a controller's own: error bars on bars
//! and on scattered points, floating range bars and range bands, box plots, candlesticks and OHLC
//! bars.

use carbide::chart::{Axis, BarController, BoxDataSetExt, BoxStats, CategoryScale, Chart, Sequence, DataColor, DatasetExt, Estimate, LinearScale, LineController, Ohlc, OhlcDataSetExt, Range, RangeDataSetExt, ScaleId, ScatterController, Spread, DatasetController};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map2};
use carbide::widget::*;
use carbide::ui;
use super::{first_section, note, shown_when, text_input, with_labelled_side_bar, Labelled, Shared};

/// The mark shown, each on data of the kind it draws.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    ErrorBars,
    ScatterErrors,
    RangeBars,
    RangeBand,
    BoxPlots,
    Candlesticks,
    Ohlc,
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    horizontal: LocalState<bool>,
    inverted: LocalState<bool>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Marks".to_string()),
            data: LocalState::new(Data::ErrorBars),
            horizontal: LocalState::new(false),
            inverted: LocalState::new(true),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    // A scale's axis is fixed when the chart is built, so making the marks horizontal, which swaps
    // the index and value scales, shows a chart built for it. Points' error bars run both ways, and
    // aren't turned.
    let shown = Map2::read_map(options.data.clone(), options.horizontal.clone(), |data: &Data, horizontal: &bool| (*data, *horizontal && *data != Data::ScatterErrors));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (Data::ErrorBars, false) => chart(o.clone(), s.clone(), Data::ErrorBars, false),
            (Data::ErrorBars, true) => chart(o.clone(), s.clone(), Data::ErrorBars, true),
            (Data::ScatterErrors, _) => chart(o.clone(), s.clone(), Data::ScatterErrors, false),
            (Data::RangeBars, false) => chart(o.clone(), s.clone(), Data::RangeBars, false),
            (Data::RangeBars, true) => chart(o.clone(), s.clone(), Data::RangeBars, true),
            (Data::RangeBand, false) => chart(o.clone(), s.clone(), Data::RangeBand, false),
            (Data::RangeBand, true) => chart(o.clone(), s.clone(), Data::RangeBand, true),
            (Data::BoxPlots, false) => chart(o.clone(), s.clone(), Data::BoxPlots, false),
            (Data::BoxPlots, true) => chart(o.clone(), s.clone(), Data::BoxPlots, true),
            (Data::Candlesticks, false) => chart(o.clone(), s.clone(), Data::Candlesticks, false),
            (Data::Candlesticks, true) => chart(o.clone(), s.clone(), Data::Candlesticks, true),
            (Data::Ohlc, false) => chart(o.clone(), s.clone(), Data::Ohlc, false),
            (Data::Ohlc, true) => chart(o.clone(), s.clone(), Data::Ohlc, true),
        }
    );

    let side = VStack::new((
        first_section("Marks"),
        text_input("Title", &options.title),
        Picker::new("Mark", options.data.clone(), (
            Text::new("Error bars on bars").tag(Data::ErrorBars),
            Text::new("Error bars on points, both ways").tag(Data::ScatterErrors),
            Text::new("Floating range bars").tag(Data::RangeBars),
            Text::new("A range band around a line").tag(Data::RangeBand),
            Text::new("Box plots").tag(Data::BoxPlots),
            Text::new("Candlesticks").tag(Data::Candlesticks),
            Text::new("OHLC bars").tag(Data::Ohlc),
        )),
        // Points' error bars run both ways already, and only range bars can fall.
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data != Data::ScatterErrors),
            Toggle::new("Horizontal", options.horizontal.clone())),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::RangeBars),
            Toggle::new("Falling ranges in their own colour", options.inverted.clone())),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| matches!(data, Data::Candlesticks | Data::Ohlc)),
            note("Rising and falling prices take the series style's rising and falling colours.")),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_labelled_side_bar(chart, side, &shared)
}

const WEEKS: [&str; 5] = ["Week 1", "Week 2", "Week 3", "Week 4", "Week 5"];

/// The chart of the mark `data`, its index scale along x, or down y when `horizontal` is.
pub(super) fn chart(options: Options, shared: Shared, data: Data, horizontal: bool) -> Box<dyn AnyWidget> {
    let scales = Scales { horizontal, categories: !matches!(data, Data::ScatterErrors | Data::Candlesticks | Data::Ohlc) };

    match data {
        Data::ErrorBars => {
            let estimate = |values: [(Scalar, Scalar, Scalar); 5]| -> Vec<Estimate<&'static str, Scalar>> {
                WEEKS.iter().zip(values).map(|(week, (y, low, high))| Estimate { x: *week, y, low, high }).collect()
            };
            let control = estimate([(12.0, 10.0, 15.0), (18.0, 14.0, 21.0), (9.0, 8.0, 11.0), (14.0, 11.0, 16.0), (16.0, 13.0, 18.0)]);
            let treated = estimate([(15.0, 12.0, 17.0), (24.0, 20.0, 29.0), (13.0, 9.0, 16.0), (19.0, 16.0, 23.0), (22.0, 18.0, 25.0)]);
            titled(&options, &shared, Chart::new(BarController::new((control.error_bars().label("Control"), treated.error_bars().label("Treated"))).horizontal(horizontal)), scales)
        }
        Data::ScatterErrors => {
            let measured = vec![
                Spread { x: 2.0, y: 12.0, x_low: 1.4, x_high: 2.9, low: 9.0, high: 14.0 },
                Spread { x: 4.0, y: 17.0, x_low: 3.3, x_high: 4.6, low: 14.0, high: 21.0 },
                Spread { x: 6.0, y: 15.0, x_low: 5.2, x_high: 7.1, low: 12.0, high: 18.0 },
                Spread { x: 8.0, y: 22.0, x_low: 7.4, x_high: 8.9, low: 19.0, high: 26.0 },
            ];
            let predicted = vec![
                Spread { x: 3.0, y: 6.0, x_low: 2.5, x_high: 3.4, low: 4.0, high: 8.0 },
                Spread { x: 5.0, y: 9.0, x_low: 4.1, x_high: 6.2, low: 8.0, high: 11.0 },
                Spread { x: 7.0, y: 13.0, x_low: 6.5, x_high: 7.6, low: 11.0, high: 16.0 },
            ];
            titled(&options, &shared, Chart::new(ScatterController::new((measured.error_bars().label("Measured"), predicted.error_bars().label("Predicted")))), scales)
        }
        Data::RangeBars => {
            // Each week's price from open to close: a falling week runs backwards, from high to low.
            let moves: Vec<Range<&'static str, Scalar>> = WEEKS.iter().zip([(10.0, 14.0), (14.0, 11.0), (11.0, 16.0), (16.0, 15.0), (15.0, 19.0)])
                .map(|(week, (low, high))| Range { x: *week, low, high }).collect();
            let inverted = Map1::read_map(options.inverted.clone(), |inverted: &bool| if *inverted { DataColor::Palette(3) } else { DataColor::ByDataSet });
            titled(&options, &shared, Chart::new(BarController::new(moves.range().inverted_color(inverted).label("Open to close")).horizontal(horizontal)), scales)
        }
        Data::RangeBand => {
            let lows = [4.0, 5.0, 7.0, 9.0, 12.0, 15.0, 17.0, 16.0, 13.0, 9.0, 6.0, 4.0];
            let highs = [9.0, 11.0, 14.0, 17.0, 21.0, 24.0, 27.0, 26.0, 22.0, 17.0, 12.0, 9.0];
            let band: Vec<Range<&'static str, Scalar>> = super::MONTHS.iter().zip(lows.iter().zip(highs))
                .map(|(month, (low, high))| Range { x: *month, low: *low, high }).collect();
            let average: Vec<(&'static str, Scalar)> = super::MONTHS.iter().zip(lows.iter().zip(highs)).map(|(month, (low, high))| (*month, (low + high) / 2.0)).collect();
            titled(&options, &shared, Chart::new(LineController::new((band.range().label("Lowest to highest"), average.label("Average"))).horizontal(horizontal)), scales)
        }
        Data::BoxPlots => {
            let stats = |values: [[Scalar; 5]; 3]| -> Vec<BoxStats<&'static str, Scalar>> {
                ["Small", "Medium", "Large"].iter().zip(values).map(|(size, [min, q1, median, q3, max])| BoxStats { x: *size, min, q1, median, q3, max }).collect()
            };
            let before = stats([[12.0, 18.0, 22.0, 27.0, 35.0], [20.0, 26.0, 31.0, 37.0, 48.0], [30.0, 41.0, 47.0, 55.0, 70.0]]);
            let after = stats([[10.0, 14.0, 17.0, 21.0, 28.0], [15.0, 20.0, 24.0, 29.0, 36.0], [22.0, 30.0, 35.0, 41.0, 52.0]]);
            titled(&options, &shared, Chart::new(BarController::new((before.box_plot().label("Before (ms)"), after.box_plot().label("After (ms)"))).horizontal(horizontal)), scales)
        }
        Data::Candlesticks | Data::Ohlc => {
            let mut close: Scalar = 100.0;
            let prices: Vec<Ohlc<Scalar, Scalar>> = (0..30).map(|day| {
                // From -0.5 to 0.5: `fract` keeps the sign, so it alone would reach -1.
                let step = |k: Scalar| ((day as Scalar * k).sin() * 43758.5453).rem_euclid(1.0) - 0.5;
                let open = close;
                // A walk pulled back towards 100, so the price stays about it.
                close = open + step(12.9898) * 8.0 + (100.0 - open) * 0.15;
                let (high, low) = (open.max(close) + step(78.233).abs() * 4.0, open.min(close) - step(39.425).abs() * 4.0);
                Ohlc { x: day as Scalar, open, high, low, close }
            }).collect();

            if data == Data::Candlesticks {
                titled(&options, &shared, Chart::new(BarController::new(prices.candles().label("Price")).horizontal(horizontal)), scales)
            } else {
                titled(&options, &shared, Chart::new(BarController::new(prices.ohlc().label("Price")).horizontal(horizontal)), scales)
            }
        }
    }
}

/// Which scales a mark's chart is drawn on: its index scale categories or numbers, along x or,
/// horizontal, down y; its value scale linear, across from it.
#[derive(Copy, Clone)]
struct Scales {
    horizontal: bool,
    categories: bool,
}

/// `chart` with its title, its scales, their tick labels as the side bar sets them, and the
/// shared options.
fn titled<C: Sequence<dyn DatasetController>>(options: &Options, shared: &Shared, chart: Chart<C>, scales: Scales) -> Box<dyn AnyWidget> {
    let (index_id, value_id, index_axis, value_axis) = if scales.horizontal {
        (ScaleId::Y, ScaleId::X, Axis::Vertical, Axis::Horizontal)
    } else {
        (ScaleId::X, ScaleId::Y, Axis::Horizontal, Axis::Vertical)
    };

    let chart = chart
        .title(options.title.clone())
        .scale(value_id, LinearScale::new(value_axis).placed(shared));

    let chart = if scales.categories {
        chart.scale(index_id, CategoryScale::new(index_axis).placed(shared))
    } else {
        chart.scale(index_id, LinearScale::new(index_axis).placed(shared))
    };

    shared.apply(chart).boxed()
}
