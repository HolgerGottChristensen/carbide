//! Marks drawn for each point instead of, or on top of, a controller's own: error bars on bars
//! and on scattered points, floating range bars and range bands, box plots, candlesticks and OHLC
//! bars.

use carbide::chart::{BarController, BoxDataSetExt, BoxStats, Chart, ControllerSequence, DataColor, DatasetExt, Estimate, LineController, Ohlc, OhlcDataSetExt, Range, RangeDataSetExt, ScatterController, Spread};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1};
use carbide::widget::*;
use carbide::ui;
use super::{first_section, note, shown_when, text_input, with_side_bar, Shared};

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
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            Data::ErrorBars => chart(o.clone(), s.clone(), Data::ErrorBars),
            Data::ScatterErrors => chart(o.clone(), s.clone(), Data::ScatterErrors),
            Data::RangeBars => chart(o.clone(), s.clone(), Data::RangeBars),
            Data::RangeBand => chart(o.clone(), s.clone(), Data::RangeBand),
            Data::BoxPlots => chart(o.clone(), s.clone(), Data::BoxPlots),
            Data::Candlesticks => chart(o.clone(), s.clone(), Data::Candlesticks),
            Data::Ohlc => chart(o.clone(), s.clone(), Data::Ohlc),
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
            Toggle::new("Lying", options.horizontal.clone())),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::RangeBars),
            Toggle::new("Falling ranges in their own colour", options.inverted.clone())),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| matches!(data, Data::Candlesticks | Data::Ohlc)),
            note("Rising and falling prices take the series style's rising and falling colours.")),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

const WEEKS: [&str; 5] = ["Week 1", "Week 2", "Week 3", "Week 4", "Week 5"];

/// The chart of the mark `data`, on the default scales, so they follow the bars when they lie down.
pub(super) fn chart(options: Options, shared: Shared, data: Data) -> Box<dyn AnyWidget> {
    let horizontal = options.horizontal.clone();

    match data {
        Data::ErrorBars => {
            let estimate = |values: [(Scalar, Scalar, Scalar); 5]| -> Vec<Estimate<&'static str, Scalar>> {
                WEEKS.iter().zip(values).map(|(week, (y, low, high))| Estimate { x: *week, y, low, high }).collect()
            };
            let control = estimate([(12.0, 10.0, 15.0), (18.0, 14.0, 21.0), (9.0, 8.0, 11.0), (14.0, 11.0, 16.0), (16.0, 13.0, 18.0)]);
            let treated = estimate([(15.0, 12.0, 17.0), (24.0, 20.0, 29.0), (13.0, 9.0, 16.0), (19.0, 16.0, 23.0), (22.0, 18.0, 25.0)]);
            titled(&options, &shared, Chart::new(BarController::new((control.error_bars().label("Control"), treated.error_bars().label("Treated"))).horizontal(horizontal)))
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
            titled(&options, &shared, Chart::new(ScatterController::new((measured.error_bars().label("Measured"), predicted.error_bars().label("Predicted")))))
        }
        Data::RangeBars => {
            // Each week's price from open to close: a falling week runs backwards, from high to low.
            let moves: Vec<Range<&'static str, Scalar>> = WEEKS.iter().zip([(10.0, 14.0), (14.0, 11.0), (11.0, 16.0), (16.0, 15.0), (15.0, 19.0)])
                .map(|(week, (low, high))| Range { x: *week, low, high }).collect();
            let inverted = Map1::read_map(options.inverted.clone(), |inverted: &bool| if *inverted { DataColor::Palette(3) } else { DataColor::ByDataSet });
            titled(&options, &shared, Chart::new(BarController::new(moves.range().inverted_color(inverted).label("Open to close")).horizontal(horizontal)))
        }
        Data::RangeBand => {
            let lows = [4.0, 5.0, 7.0, 9.0, 12.0, 15.0, 17.0, 16.0, 13.0, 9.0, 6.0, 4.0];
            let highs = [9.0, 11.0, 14.0, 17.0, 21.0, 24.0, 27.0, 26.0, 22.0, 17.0, 12.0, 9.0];
            let band: Vec<Range<&'static str, Scalar>> = super::MONTHS.iter().zip(lows.iter().zip(highs))
                .map(|(month, (low, high))| Range { x: *month, low: *low, high }).collect();
            let average: Vec<(&'static str, Scalar)> = super::MONTHS.iter().zip(lows.iter().zip(highs)).map(|(month, (low, high))| (*month, (low + high) / 2.0)).collect();
            titled(&options, &shared, Chart::new(LineController::new((band.range().label("Lowest to highest"), average.label("Average"))).horizontal(horizontal)))
        }
        Data::BoxPlots => {
            let stats = |values: [[Scalar; 5]; 3]| -> Vec<BoxStats<&'static str, Scalar>> {
                ["Small", "Medium", "Large"].iter().zip(values).map(|(size, [min, q1, median, q3, max])| BoxStats { x: *size, min, q1, median, q3, max }).collect()
            };
            let before = stats([[12.0, 18.0, 22.0, 27.0, 35.0], [20.0, 26.0, 31.0, 37.0, 48.0], [30.0, 41.0, 47.0, 55.0, 70.0]]);
            let after = stats([[10.0, 14.0, 17.0, 21.0, 28.0], [15.0, 20.0, 24.0, 29.0, 36.0], [22.0, 30.0, 35.0, 41.0, 52.0]]);
            titled(&options, &shared, Chart::new(BarController::new((before.box_plot().label("Before (ms)"), after.box_plot().label("After (ms)"))).horizontal(horizontal)))
        }
        Data::Candlesticks | Data::Ohlc => {
            let mut close: Scalar = 100.0;
            let prices: Vec<Ohlc<Scalar, Scalar>> = (0..30).map(|day| {
                let step = |k: Scalar| ((day as Scalar * k).sin() * 43758.5453).fract() - 0.5;
                let open = close;
                // A walk pulled back towards 100, so the price stays about it.
                close = open + step(12.9898) * 8.0 + (100.0 - open) * 0.15;
                let (high, low) = (open.max(close) + step(78.233).abs() * 4.0, open.min(close) - step(39.425).abs() * 4.0);
                Ohlc { x: day as Scalar, open, high, low, close }
            }).collect();

            if data == Data::Candlesticks {
                titled(&options, &shared, Chart::new(BarController::new(prices.candles().label("Price")).horizontal(horizontal)))
            } else {
                titled(&options, &shared, Chart::new(BarController::new(prices.ohlc().label("Price")).horizontal(horizontal)))
            }
        }
    }
}

/// `chart` with its title and the shared options.
fn titled<C: ControllerSequence>(options: &Options, shared: &Shared, chart: Chart<C>) -> Box<dyn AnyWidget> {
    shared.apply(chart.title(options.title.clone())).boxed()
}
