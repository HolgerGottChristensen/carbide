//! Candlesticks and OHLC bars: each period's open, high, low and close, in the rising or the
//! falling colour.

use carbide_chart::{BarController, Chart, DataColor, DatasetExt, LegendPosition, LineController, Ohlc, OhlcDataSetExt};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Eight trading days as candlesticks, rising and falling, with one flat day. Pins down that each
/// body runs from the open to the close and each wick from the low to the high, that rising days
/// are drawn in the style's rising colour (green) and falling ones in its falling colour (red), a
/// flat day included as falling and drawn as a line, and that the y scale fits the wicks.
#[test]
fn rising_and_falling() {
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

    harness.snapshot("rising_and_falling");
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
fn in_the_legend() {
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

    harness.snapshot("in_the_legend");
}
