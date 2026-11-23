//! Hover tooltips (`Chart::tooltip`): a box next to the hovered data, naming what it is and its
//! values, from the data's descriptions (`Pick::items`), placed where it fits and drawn above
//! everything else.

use carbide_chart::{Axis, BarController, TooltipAnchor, CategoryScale, Chart, ControllerSequence, DatasetExt, LegendPosition, LineController, Measure, Ohlc, OhlcDataSetExt, Pick, PickMode, PieController, ScaleId, ScatterController};
use carbide_core::draw::{Alignment, Dimension, Position};
use carbide_core::draw::theme::Theme;
use carbide_core::state::{LocalState, ReadState};
use carbide_testing::Harness;

mod common;

use common::Framed;

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];

/// Grouped, named bars with a legend, as the pointer tests hover them.
fn regions() -> Chart<impl ControllerSequence> {
    Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0].label("North"), vec![15.0, 25.0, 5.0, 20.0].label("South"))))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS))
        .legend(LegendPosition::Bottom)
}

/// Grouped, named bars hovered above Q3, in the default `Index` mode, with a hover state. Pins
/// down that the tooltip is titled with the category and has a row per dataset, its legend square
/// in its colour, its name and its value; that it points at the middle of the two bars; and that
/// the hover state's `Pick::items` carries the same names and values.
#[test]
fn a_shared_tooltip_over_grouped_bars() {
    let hovered = LocalState::new(None::<Pick>);
    let chart = regions().hover(hovered.clone()).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(380.0, 120.0));
    harness.snapshot("a_shared_tooltip_over_grouped_bars");

    let pick = hovered.value().clone().expect("the pointer is over the chart area");
    let rows: Vec<String> = pick.items.iter().map(|item| item.row_text()).collect();
    assert_eq!(rows, vec!["North: 30", "South: 5"]);
    assert_eq!(pick.items[0].index.as_deref(), Some("Q3"));
}

/// Two named scatter datasets, hovered by the nearest point. Pins down that the tooltip shows the
/// one point, titled with its x as the x scale labels it, its dataset's point marker and its y.
#[test]
fn the_nearest_point_on_a_scatter_chart() {
    let chart = Chart::new(ScatterController::new((
        vec![(1.0, 2.0), (3.0, 8.0), (5.0, 3.0)].label("Field"),
        vec![(2.0, 9.0), (4.0, 1.0)].label("Lab"),
    )))
        .hover_mode(PickMode::Nearest(Measure::Both))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(340.0, 170.0));
    harness.snapshot("the_nearest_point_on_a_scatter_chart");
}

/// A named line of eight points in the `Dataset` mode, its rows capped at three. Pins down that
/// the tooltip is titled with the dataset's name, shows its first three data, each by its index
/// and value, and then a row saying how many more there are.
#[test]
fn a_datasets_rows_capped() {
    let chart = Chart::new(LineController::new(vec![3.0, 5.0, 4.0, 6.0, 7.0, 5.0, 8.0, 6.0].label("Visitors")))
        .hover_mode(PickMode::Dataset)
        .tooltip_max_rows(3usize)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(250.0, 250.0));
    harness.snapshot("a_datasets_rows_capped");
}

/// A line whose last point, its highest, is at the chart's top right corner, hovered by the
/// nearest point. Pins down that a tooltip that doesn't fit above its data goes below it, sliding
/// left to stay inside the chart, its caret still pointing at the data.
#[test]
fn flipped_near_the_edges() {
    let chart = Chart::new(LineController::new(vec![3.0, 4.0, 3.5, 5.0, 4.5, 9.0].label("Signups")))
        .hover_mode(PickMode::Nearest(Measure::Both))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(560.0, 40.0));
    harness.snapshot("flipped_near_the_edges");
}

/// A pie hovered over a slice. Pins down that the tooltip is titled with the part's name and
/// shows its value and its share of the whole, pointing at the slice's middle.
#[test]
fn a_pie_slice() {
    let votes = vec![("Red", 12.0), ("Blue", 19.0), ("Yellow", 3.0), ("Green", 6.0)];
    let chart = Chart::new(PieController::new(votes.label("Votes")))
        .hover_mode(PickMode::Point)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(380.0, 300.0));
    harness.snapshot("a_pie_slice");
}

/// Candlesticks hovered over one. Pins down that its row shows the open, high, low and close,
/// named, with the candles' split legend marker.
#[test]
fn a_candlesticks_prices() {
    let prices = vec![
        Ohlc { x: 0.0, open: 100.0, high: 104.0, low: 98.0, close: 103.0 },
        Ohlc { x: 1.0, open: 103.0, high: 106.0, low: 101.0, close: 102.0 },
        Ohlc { x: 2.0, open: 102.0, high: 103.0, low: 97.0, close: 98.0 },
        Ohlc { x: 3.0, open: 98.0, high: 101.0, low: 96.0, close: 100.0 },
    ];
    let hovered = LocalState::new(None::<Pick>);
    let chart = Chart::new(BarController::new(prices.candles().label("ACME")))
        .hover(hovered.clone())
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(240.0, 250.0));
    harness.snapshot("a_candlesticks_prices");

    let pick = hovered.value().clone().expect("over a candle");
    assert_eq!(pick.items[0].row_text(), "ACME: open 103, high 106, low 101, close 102");
}

/// The bars with a title and row text of their own. Pins down that `tooltip_title` and
/// `tooltip_text` replace the default title and rows.
#[test]
fn custom_title_and_rows() {
    let chart = regions()
        .tooltip_title(|pick| format!("{} items", pick.items.len()))
        .tooltip_text(|item| format!("{} → {}", item.index.clone().unwrap_or_default(), item.values_text()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(380.0, 120.0));
    harness.snapshot("custom_title_and_rows");
}

/// The shared tooltip over the bars in the light theme. Pins down that `AutomaticStyle` follows
/// the theme: a light background, dark text.
#[test]
fn in_the_light_theme() {
    let mut harness = Harness::new(regions().framed(), Dimension::new(600.0, 500.0));
    harness.set_theme(Theme::Light);
    harness.render();

    harness.mouse_move(Position::new(380.0, 120.0));
    harness.snapshot("in_the_light_theme");
}

/// The bars hovered with `tooltip(false)`, and with `interactive(false)`, then hovered over the
/// legend. Pins down that no tooltip is drawn either way, and that the legend has none.
#[test]
fn when_no_tooltip_shows() {
    let mut harness = Harness::new(regions().tooltip(false).framed(), Dimension::new(600.0, 500.0));
    harness.render();
    harness.mouse_move(Position::new(380.0, 120.0));
    harness.snapshot("turned_off");

    let mut harness = Harness::new(regions().interactive(false).framed(), Dimension::new(600.0, 500.0));
    harness.render();
    harness.mouse_move(Position::new(380.0, 120.0));
    harness.snapshot("not_interactive");

    let mut harness = Harness::new(regions().framed(), Dimension::new(600.0, 500.0));
    harness.render();
    harness.mouse_move(Position::new(300.0, 462.0));
    harness.snapshot("over_the_legend");
}

/// The shared tooltip over the grouped bars following the pointer. Pins down that it sits above
/// the pointer, its caret pointing at it, rather than at the middle of Q3's bars, still showing
/// both bars.
#[test]
fn following_the_pointer() {
    let chart = regions().tooltip_anchor(TooltipAnchor::Pointer).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(400.0, 120.0));
    harness.snapshot("following_the_pointer");
}

/// A tooltip preferring `TopLeading`, following the pointer near the chart's top left. Pins down
/// that, as it doesn't fit above and before the pointer there, it goes to the mirrored corner,
/// below and after it, without a caret.
#[test]
fn a_corner_falling_back_to_its_mirror() {
    let chart = regions()
        .tooltip_anchor(TooltipAnchor::Pointer)
        .tooltip_alignment(Alignment::TopLeading)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(90.0, 60.0));
    harness.snapshot("a_corner_falling_back_to_its_mirror");
}

/// The nearest scatter point's tooltip preferring `Trailing`. Pins down that it sits after the
/// point, centred on it, its caret on its leading side pointing back at it.
#[test]
fn trailing_a_point() {
    let chart = Chart::new(ScatterController::new((
        vec![(1.0, 2.0), (3.0, 8.0), (5.0, 3.0)].label("Field"),
        vec![(2.0, 9.0), (4.0, 1.0)].label("Lab"),
    )))
        .hover_mode(PickMode::Nearest(Measure::Both))
        .tooltip_alignment(Alignment::Trailing)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(340.0, 170.0));
    harness.snapshot("trailing_a_point");
}
