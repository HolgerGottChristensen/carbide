//! A chart built from this crate's own chart elements, through the public API alone: the
//! `LollipopController`, the `LogScale` and `Kelvin` values, zooming along both scales.

use carbide::chart::{Axis, BarController, CategoryScale, Chart, DataColor, DatasetExt, LinearScale, LineController, ScaleId, SnapMode, ZoomWindow};
use carbide::draw::Scalar;
use carbide::color::{BLUE, GREEN, PURPLE};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::state::{LocalState, Map1, Map3, State};
use carbide::widget::*;
use carbide::ui;
use chart_extensions::{Blocks, BlocksExt, Kelvin, LogScale, LollipopController, ValueTicks};
use super::{first_section, percent, section, shown_when, slider, snap_picker, text_input, with_labelled_side_bar, Labelled, zoom_controls, Shared};

/// The weekdays the visits are counted on.
const WEEKDAYS: [&str; 5] = ["Mon", "Tue", "Wed", "Thu", "Fri"];

/// The data the custom chart can show, drawn with this crate's own chart elements.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum CustomData {
    /// Readings in kelvin (`Kelvin`) shown in degrees Celsius, with the forecast as lollipops.
    Temperature,
    /// The populations of towns and cities in three countries, from villages of under a hundred
    /// people to capitals, five orders of magnitude, as lollipops: one dataset per country, each
    /// adding its own places as categories.
    Towns,
    /// Visits per weekday, drawn as `Blocks` with `ValueTicks` on top, beside plain bars.
    Visits,
}

/// The lollipops' colour.
#[derive(Copy, Clone, Debug, PartialEq)]
enum LollipopColor {
    /// The palette colour for the dataset, as the built-in controllers colour theirs.
    Palette,
    /// The next palette colour for each lollipop.
    ByPoint,
    Blue,
    Green,
    Purple,
}

impl LollipopColor {
    fn data_color(&self) -> DataColor {
        match self {
            LollipopColor::Palette => DataColor::ByDataSet,
            LollipopColor::ByPoint => DataColor::ByPoint,
            LollipopColor::Blue => DataColor::Color(BLUE),
            LollipopColor::Green => DataColor::Color(GREEN),
            LollipopColor::Purple => DataColor::Color(PURPLE),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<CustomData>,
    log_scale: LocalState<bool>,
    color: LocalState<LollipopColor>,
    /// What the x scale shows, which drags and the wheel pan and pinches zoom.
    x_zoom: LocalState<ZoomWindow>,
    /// What the y scale shows, the `LogScale` or the linear one.
    y_zoom: LocalState<ZoomWindow>,
    /// How the zoom of a category x scale lines up with its categories, on the charts that have
    /// one.
    snap_mode: LocalState<SnapMode>,
    /// Whether the value scale fits to the data inside the index scale's zoom window.
    fit_visible: LocalState<bool>,
    /// Whether the charts lie along x, their categories or index values down y.
    horizontal: LocalState<bool>,
    /// Whether the value scale draws grid lines.
    grid_lines: LocalState<bool>,
    /// What each of the visits' blocks is worth (`Blocks::unit`).
    block_unit: LocalState<Scalar>,
    /// How much of each category the visits' bar groups take (`BarController::category_percentage`).
    category_percentage: LocalState<Scalar>,
    /// How much of its share of a group each visits bar takes (`BarController::bar_percentage`).
    bar_percentage: LocalState<Scalar>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Extended from outside".to_string()),
            data: LocalState::new(CustomData::Temperature),
            log_scale: LocalState::new(true),
            color: LocalState::new(LollipopColor::Palette),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            snap_mode: LocalState::new(SnapMode::Categories),
            fit_visible: LocalState::new(false),
            horizontal: LocalState::new(false),
            grid_lines: LocalState::new(true),
            block_unit: LocalState::new(5.0),
            category_percentage: LocalState::new(0.8),
            bar_percentage: LocalState::new(0.9),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    // A scale's type and side are fixed when the chart is built, so the logarithmic scale and the
    // linear one, and the upright and horizontal charts, whose scales swap sides, are charts of
    // their own.
    let shown = Map3::read_map(options.data.clone(), options.log_scale.clone(), options.horizontal.clone(), |data: &CustomData, log: &bool, horizontal: &bool| (*data, *log, *horizontal));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (CustomData::Temperature, _, false) => chart(o.clone(), s.clone(), CustomData::Temperature, false, false),
            (CustomData::Temperature, _, true) => chart(o.clone(), s.clone(), CustomData::Temperature, false, true),
            (CustomData::Towns, false, false) => chart(o.clone(), s.clone(), CustomData::Towns, false, false),
            (CustomData::Towns, false, true) => chart(o.clone(), s.clone(), CustomData::Towns, false, true),
            (CustomData::Towns, true, false) => chart(o.clone(), s.clone(), CustomData::Towns, true, false),
            (CustomData::Towns, true, true) => chart(o.clone(), s.clone(), CustomData::Towns, true, true),
            (CustomData::Visits, _, false) => chart(o.clone(), s.clone(), CustomData::Visits, false, false),
            (CustomData::Visits, _, true) => chart(o.clone(), s.clone(), CustomData::Visits, false, true),
        }
    );
    let visits = Map1::read_map(options.data.clone(), |data: &CustomData| *data == CustomData::Visits);

    // Only the temperature and town charts draw lollipops.
    let lollipops = Map1::read_map(options.data.clone(), |data: &CustomData| *data != CustomData::Visits);

    // What the chart shows and how it is drawn, then the hover notes, then the zoom, each a stack of
    // its own, as a stack takes at most twelve items.
    let what = VStack::new((
        Text::new("Drawn with this crate's own\nLollipopController, LogScale, Kelvin\nvalues, and Blocks and ValueTicks\nmarks, through the public API."),
        text_input("Title", &options.title),
        Picker::new("Data", data_resetting_zoom(&options), (
            Text::new("Temperature").tag(CustomData::Temperature),
            Text::new("Towns and cities").tag(CustomData::Towns),
            Text::new("Visits, in blocks").tag(CustomData::Visits),
        )),
        shown_when(lollipops.clone(), Picker::new("Lollipops", options.color.clone(), (
            Text::new("Palette").tag(LollipopColor::Palette),
            Text::new("One per lollipop").tag(LollipopColor::ByPoint),
            Text::new("Blue").tag(LollipopColor::Blue),
            Text::new("Green").tag(LollipopColor::Green),
            Text::new("Purple").tag(LollipopColor::Purple),
        ))),
        shown_when(Map1::read_map(options.data.clone(), |data: &CustomData| *data == CustomData::Towns),
            Toggle::new("On the LogScale", options.log_scale.clone())),
        shown_when(visits.clone(), VStack::new((
            slider("Block size", &options.block_unit, 1.0, 10.0, 1.0, |unit| format!("{unit:.0}")),
            slider("Group", &options.category_percentage, 0.1, 1.0, 0.05, percent),
            slider("Bar", &options.bar_percentage, 0.1, 1.0, 0.05, percent),
        )).cross_axis_alignment(CrossAxisAlignment::Start)),
        Toggle::new("Horizontal", options.horizontal.clone()),
        Toggle::new("Grid lines", options.grid_lines.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let notes = VStack::new((
        shown_when(lollipops, Text::new("Hover finds no lollipops: a\ncontroller outside the crate can't\nadd its shapes yet.")),
        shown_when(visits, Text::new("Hover finds the blocks: a mark\nof your own adds their shapes.")),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let zoom = VStack::new((
        zoom_controls("x", &options.x_zoom),
        // Only the towns and the visits are on a category scale, whose zoom can snap to them.
        shown_when(Map1::read_map(options.data.clone(), |data: &CustomData| *data != CustomData::Temperature), snap_picker(&options.snap_mode)),
        zoom_controls("y", &options.y_zoom),
        Toggle::new("Values fit the part shown", options.fit_visible.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    let side = VStack::new((
        first_section("Custom"),
        what,
        notes,
        section("Zoom"),
        zoom,
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_labelled_side_bar(chart, side, &shared)
}

/// The data option, which shows each new choice of data whole: picking other data resets both
/// scales' zoom.
fn data_resetting_zoom(options: &Options) -> impl State<T=CustomData> + use<> {
    Map3::map(
        options.data.clone(), options.x_zoom.clone(), options.y_zoom.clone(),
        |data: &CustomData, _: &ZoomWindow, _: &ZoomWindow| *data,
        |new, mut data, mut x_zoom, mut y_zoom| {
            if *data != new {
                *data = new;
                *x_zoom = ZoomWindow::WHOLE;
                *y_zoom = ZoomWindow::WHOLE;
            }
        },
    )
}

/// The custom chart of `data`, its values on the crate's `LogScale` when `log` is set, and lying
/// along x when `horizontal` is, its index scale then the y scale and its value scale the x scale.
pub(super) fn chart(options: Options, shared: Shared, data: CustomData, log: bool, horizontal: bool) -> Box<dyn AnyWidget> {
    let color = Map1::read_map(options.color.clone(), |color: &LollipopColor| color.data_color());

    // The ids and sides of the index and value scales, and the zoom of each id, whichever side it
    // is on.
    let (index_id, value_id) = if horizontal { (ScaleId::Y, ScaleId::X) } else { (ScaleId::X, ScaleId::Y) };
    let (index_axis, value_axis) = if horizontal { (Axis::Vertical, Axis::Horizontal) } else { (Axis::Horizontal, Axis::Vertical) };
    let zoom = |id: &ScaleId| if *id == ScaleId::X { options.x_zoom.clone() } else { options.y_zoom.clone() };

    // The value scale, linear, with the options every chart shares.
    let linear_values = |title: Option<&str>| {
        let scale = LinearScale::new(value_axis)
            .zoom(zoom(&value_id))
            .fit_visible(options.fit_visible.clone())
            .grid_lines(options.grid_lines.clone())
            .placed(&shared);
        match title {
            Some(title) => scale.title(title.to_string()),
            None => scale,
        }
    };

    match data {
        CustomData::Temperature => {
            // A sensor that broke down on Thursday: `Kelvin` takes its reading of 0 K as missing,
            // so the line has a gap there.
            let readings = vec![(0.0, Kelvin(285.2)), (1.0, Kelvin(287.9)), (2.0, Kelvin(290.1)), (3.0, Kelvin(0.0)), (4.0, Kelvin(288.4)), (5.0, Kelvin(284.6)), (6.0, Kelvin(283.0))];
            let forecast = vec![(0.0, 11.0), (1.0, 14.5), (2.0, 16.0), (3.0, 17.5), (4.0, 15.0), (5.0, 12.0), (6.0, 10.5)];

            shared.apply(Chart::new((
                LineController::new(readings.label("Measured (°C)")).horizontal(options.horizontal.clone()),
                // One colour into the palette, so the forecast doesn't take the line's colour.
                LollipopController::new(forecast.label("Forecast (°C)").color(color)).palette_offset(1).horizontal(options.horizontal.clone()),
            ))
                .title(options.title.clone())
                .scale(index_id.clone(), LinearScale::new(index_axis).zoom(zoom(&index_id)).placed(&shared))
                .scale(value_id.clone(), linear_values(None)))
                .boxed()
        }
        CustomData::Towns => {
            // Rounded populations. Each country names its own places, so they follow each other
            // along the index scale, one country after the other, each in its colour.
            let denmark = vec![("Aarhus", 290_000.0), ("Skagen", 8_000.0), ("Tunø", 100.0)];
            let sweden = vec![("Malmö", 350_000.0), ("Kiruna", 17_000.0), ("Abisko", 85.0)];
            let norway = vec![("Oslo", 700_000.0), ("Bergen", 290_000.0), ("Røros", 5_600.0)];

            let chart = Chart::new(LollipopController::new((
                denmark.label("Denmark").color(color.clone()),
                sweden.label("Sweden").color(color.clone()),
                norway.label("Norway").color(color),
            )).horizontal(options.horizontal.clone()))
                .title(options.title.clone())
                .scale(index_id.clone(), CategoryScale::new(index_axis).zoom(zoom(&index_id)).snap_mode(options.snap_mode.clone()).placed(&shared));

            if log {
                let values = LogScale::new(value_axis)
                    .title("Inhabitants (log)")
                    .zoom(zoom(&value_id))
                    .fit_visible(options.fit_visible.clone())
                    .grid_lines(options.grid_lines.clone())
                    .placed(&shared);
                shared.apply(chart.scale(value_id.clone(), values)).boxed()
            } else {
                shared.apply(chart.scale(value_id.clone(), linear_values(Some("Inhabitants")))).boxed()
            }
        }
        CustomData::Visits => {
            // This week's visits as blocks, their size a state set after attaching, each value
            // marked by a tick, beside last week's as plain bars.
            let this_week = vec![23.0, 31.0, 18.5, 27.0, 35.0];
            let last_week = vec![20.0, 26.0, 22.0, 24.0, 30.0];

            shared.apply(Chart::new(BarController::new((
                this_week.label("This week").draw_as(Blocks::new()).unit(options.block_unit.clone()).overlay(ValueTicks),
                last_week.label("Last week"),
            ))
                .horizontal(options.horizontal.clone())
                .category_percentage(options.category_percentage.clone())
                .bar_percentage(options.bar_percentage.clone()))
                .title(options.title.clone())
                .scale(index_id.clone(), CategoryScale::new(index_axis).labels(WEEKDAYS).zoom(zoom(&index_id)).snap_mode(options.snap_mode.clone()).placed(&shared))
                .scale(value_id.clone(), linear_values(Some("Visits"))))
                .boxed()
        }
    }
}
