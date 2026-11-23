//! A chart built from this crate's own chart elements, through the public API alone: the
//! `LollipopController`, the `LogScale` and `Kelvin` values, zooming along both scales.

use carbide::chart::{Axis, BarController, CategoryScale, Chart, DataColor, DatasetExt, LinearScale, LineController, ScaleId, ZoomWindow};
use carbide::color::{BLUE, GREEN, PURPLE};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::state::{LocalState, Map1, Map2, Map3, State};
use carbide::widget::*;
use carbide::ui;
use chart_extensions::{Blocks, BlocksExt, Kelvin, LogScale, LollipopController, ValueTicks};

/// The weekdays the visits are counted on.
const WEEKDAYS: [&str; 5] = ["Mon", "Tue", "Wed", "Thu", "Fri"];
use super::{first_section, section, shown_when, text_input, with_side_bar, zoom_controls, Shared};


/// The data the custom chart can show, drawn with this crate's own chart elements.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum CustomData {
    /// Readings in kelvin (`Kelvin`) shown in degrees Celsius, with the forecast as lollipops.
    Temperature,
    /// Populations spanning five orders of magnitude, as lollipops.
    Settlements,
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
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    // A scale's type is fixed when the chart is built, so the logarithmic scale and the linear one
    // are charts of their own.
    let shown = Map2::read_map(options.data.clone(), options.log_scale.clone(), |data: &CustomData, log: &bool| (*data, *log));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (CustomData::Temperature, _) => chart(o.clone(), s.clone(), CustomData::Temperature, false),
            (CustomData::Settlements, false) => chart(o.clone(), s.clone(), CustomData::Settlements, false),
            (CustomData::Settlements, true) => chart(o.clone(), s.clone(), CustomData::Settlements, true),
            (CustomData::Visits, _) => chart(o.clone(), s.clone(), CustomData::Visits, false),
        }
    );

    // Only the temperature and settlement charts draw lollipops.
    let lollipops = Map1::read_map(options.data.clone(), |data: &CustomData| *data != CustomData::Visits);

    let side = VStack::new((
        first_section("Custom"),
        Text::new("Drawn with this crate's own\nLollipopController, LogScale, Kelvin\nvalues, and Blocks and ValueTicks\nmarks, through the public API."),
        text_input("Title", &options.title),
        Picker::new("Data", data_resetting_zoom(&options), (
            Text::new("Temperature").tag(CustomData::Temperature),
            Text::new("Settlements").tag(CustomData::Settlements),
            Text::new("Visits, in blocks").tag(CustomData::Visits),
        )),
        shown_when(lollipops.clone(), Picker::new("Lollipops", options.color.clone(), (
            Text::new("Palette").tag(LollipopColor::Palette),
            Text::new("One per lollipop").tag(LollipopColor::ByPoint),
            Text::new("Blue").tag(LollipopColor::Blue),
            Text::new("Green").tag(LollipopColor::Green),
            Text::new("Purple").tag(LollipopColor::Purple),
        ))),
        shown_when(Map1::read_map(options.data.clone(), |data: &CustomData| *data == CustomData::Settlements),
            Toggle::new("On the LogScale", options.log_scale.clone())),
        shown_when(lollipops, Text::new("Hover finds no lollipops: a\ncontroller outside the crate can't\nadd its shapes yet.")),
        shown_when(Map1::read_map(options.data.clone(), |data: &CustomData| *data == CustomData::Visits),
            Text::new("Hover finds the blocks: a mark\nof your own adds their shapes.")),
        section("Zoom"),
        zoom_controls("x", &options.x_zoom),
        zoom_controls("y", &options.y_zoom),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
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

/// The custom chart of `data`, on the crate's `LogScale` when `log` is set.
pub(super) fn chart(options: Options, shared: Shared, data: CustomData, log: bool) -> Box<dyn AnyWidget> {
    let color = Map1::read_map(options.color.clone(), |color: &LollipopColor| color.data_color());

    match data {
        CustomData::Temperature => {
            // A sensor that broke down on Thursday: `Kelvin` takes its reading of 0 K as missing,
            // so the line has a gap there.
            let readings = vec![(0.0, Kelvin(285.2)), (1.0, Kelvin(287.9)), (2.0, Kelvin(290.1)), (3.0, Kelvin(0.0)), (4.0, Kelvin(288.4)), (5.0, Kelvin(284.6)), (6.0, Kelvin(283.0))];
            let forecast = vec![(0.0, 11.0), (1.0, 14.5), (2.0, 16.0), (3.0, 17.5), (4.0, 15.0), (5.0, 12.0), (6.0, 10.5)];

            shared.apply(Chart::new((
                LineController::new(readings.label("Measured (°C)")),
                // One colour into the palette, so the forecast doesn't take the line's colour.
                LollipopController::new(forecast.label("Forecast (°C)").color(color)).palette_offset(1),
            ))
                .title(options.title.clone())
                .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(options.x_zoom.clone()))
                .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(options.y_zoom.clone())))
                .boxed()
        }
        CustomData::Settlements => {
            let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];
            let chart = Chart::new(LollipopController::new(populations.label("Population").color(color)))
                .title(options.title.clone())
                .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(options.x_zoom.clone()));

            if log {
                shared.apply(chart.scale(ScaleId::Y, LogScale::new(Axis::Vertical).title("Inhabitants (log)").zoom(options.y_zoom.clone()))).boxed()
            } else {
                shared.apply(chart.scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Inhabitants").zoom(options.y_zoom.clone()))).boxed()
            }
        }
        CustomData::Visits => {
            // This week's visits as blocks of 5, set after attaching, each value marked by a tick,
            // beside last week's as plain bars.
            let this_week = vec![23.0, 31.0, 18.5, 27.0, 35.0];
            let last_week = vec![20.0, 26.0, 22.0, 24.0, 30.0];

            shared.apply(Chart::new(BarController::new((
                this_week.label("This week").draw_as(Blocks::new()).unit(5.0).overlay(ValueTicks),
                last_week.label("Last week"),
            )))
                .title(options.title.clone())
                .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(WEEKDAYS).zoom(options.x_zoom.clone()))
                .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Visits").zoom(options.y_zoom.clone())))
                .boxed()
        }
    }
}
