//! Scales: time, linear, logarithmic and category scales, a second y axis for one dataset, axes on
//! either side with titles and tick marks, a fixed start and step, grid lines between categories,
//! and zooming along each scale, the categories snapping to whole ones or scrolling freely.

use carbide::chart::{Axis, BarController, CategoryScale, Chart, Sequence, DatasetExt, LinearScale, LineController, LogScale, ScaleId, SnapMode, TimeScale, TimeScaleZone, ZoomWindow, DatasetController};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map3, Map4, State};
use carbide::widget::*;
use carbide::ui;
use chrono::{DateTime, Duration, FixedOffset, TimeZone};
use super::{first_section, section, shown_when, slider, snap_picker, text_input, with_labelled_side_bar, Labelled, zoom_controls, Shared};

/// The second y axis, for the second dataset.
const SECOND: ScaleId = ScaleId::new("second");

/// The data, each on an x scale of its own kind.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    /// Two months of daily weather, at dates on a time scale.
    Weather,
    /// How a city thins out away from its centre, at distances on a linear scale.
    City,
    /// The most common words, named by the data on a category scale.
    Words,
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    log: LocalState<bool>,
    second_axis: LocalState<bool>,
    x_title: LocalState<String>,
    y_title: LocalState<String>,
    tick_marks: LocalState<bool>,
    from_zero: LocalState<bool>,
    step: LocalState<Scalar>,
    grid_offset: LocalState<bool>,
    x_zoom: LocalState<ZoomWindow>,
    snap_mode: LocalState<SnapMode>,
    y_zoom: LocalState<ZoomWindow>,
    second_zoom: LocalState<ZoomWindow>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Scales".to_string()),
            data: LocalState::new(Data::Weather),
            log: LocalState::new(false),
            second_axis: LocalState::new(true),
            x_title: LocalState::new("x".to_string()),
            y_title: LocalState::new("y".to_string()),
            tick_marks: LocalState::new(true),
            from_zero: LocalState::new(false),
            step: LocalState::new(0.0),
            grid_offset: LocalState::new(true),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            snap_mode: LocalState::new(SnapMode::Categories),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            second_zoom: LocalState::new(ZoomWindow::WHOLE),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    // A scale's type, and which scale a dataset is on, are fixed when the chart is built, so each
    // choice of data, y scale and second axis is a chart of its own.
    let shown = Map3::read_map(options.data.clone(), options.log.clone(), options.second_axis.clone(), |data: &Data, log: &bool, second: &bool| (*data, *log, *second));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (Data::Weather, false, false) => chart(o.clone(), s.clone(), Data::Weather, false, false),
            (Data::Weather, false, true) => chart(o.clone(), s.clone(), Data::Weather, false, true),
            (Data::Weather, true, false) => chart(o.clone(), s.clone(), Data::Weather, true, false),
            (Data::Weather, true, true) => chart(o.clone(), s.clone(), Data::Weather, true, true),
            (Data::City, false, false) => chart(o.clone(), s.clone(), Data::City, false, false),
            (Data::City, false, true) => chart(o.clone(), s.clone(), Data::City, false, true),
            (Data::City, true, false) => chart(o.clone(), s.clone(), Data::City, true, false),
            (Data::City, true, true) => chart(o.clone(), s.clone(), Data::City, true, true),
            (Data::Words, false, false) => chart(o.clone(), s.clone(), Data::Words, false, false),
            (Data::Words, false, true) => chart(o.clone(), s.clone(), Data::Words, false, true),
            (Data::Words, true, false) => chart(o.clone(), s.clone(), Data::Words, true, false),
            (Data::Words, true, true) => chart(o.clone(), s.clone(), Data::Words, true, true),
        }
    );

    let side = VStack::new((
        first_section("Scales"),
        text_input("Title", &options.title),
        Picker::new("Data", data_resetting_zoom(&options), (
            Text::new("Weather, on a time scale").tag(Data::Weather),
            Text::new("A city by distance, on a linear scale").tag(Data::City),
            Text::new("Words, named by the data").tag(Data::Words),
        )),
        Toggle::new("y on a logarithmic scale", options.log.clone()),
        Toggle::new("The second dataset on its own axis", options.second_axis.clone()),
        VStack::new((
            section("Axes"),
            text_input("x title", &options.x_title),
            text_input("y title", &options.y_title),
            Toggle::new("Tick marks", options.tick_marks.clone()),
            // A fixed start and step are for a linear y, and grid lines between categories for the
            // words, the only data on a category scale.
            shown_when(Map1::read_map(options.log.clone(), |log: &bool| !*log), VStack::new((
                Toggle::new("y starts at zero", options.from_zero.clone()),
                slider("y step", &options.step, 0.0, 50.0, 5.0, |step| if *step == 0.0 { "nice".to_string() } else { format!("{step:.0}") }),
            )).cross_axis_alignment(CrossAxisAlignment::Start)),
            shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::Words),
                Toggle::new("Grid lines between words", options.grid_offset.clone())),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
        VStack::new((
            section("Zoom"),
            zoom_controls("x", &options.x_zoom),
            // Only the words are on a category scale, whose zoom can snap to them.
            shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::Words), snap_picker(&options.snap_mode)),
            zoom_controls("y", &options.y_zoom),
            shown_when(options.second_axis.clone(), zoom_controls("Second y", &options.second_zoom)),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_labelled_side_bar(chart, side, &shared)
}

/// The data option, which shows each new choice of data whole: picking other data resets every
/// scale's zoom.
fn data_resetting_zoom(options: &Options) -> impl State<T=Data> + use<> {
    Map4::map(
        options.data.clone(), options.x_zoom.clone(), options.y_zoom.clone(), options.second_zoom.clone(),
        |data: &Data, _: &ZoomWindow, _: &ZoomWindow, _: &ZoomWindow| *data,
        |new, mut data, mut x_zoom, mut y_zoom, mut second_zoom| {
            if *data != new {
                *data = new;
                *x_zoom = ZoomWindow::WHOLE;
                *y_zoom = ZoomWindow::WHOLE;
                *second_zoom = ZoomWindow::WHOLE;
            }
        },
    )
}

/// The chart of `data`, its y scale logarithmic when `log` is set, and its second dataset on a
/// second y axis of its own when `second` is.
pub(super) fn chart(options: Options, shared: Shared, data: Data, log: bool, second: bool) -> Box<dyn AnyWidget> {
    let second_id = if second { SECOND } else { ScaleId::Y };

    match data {
        Data::Weather => {
            let utc = FixedOffset::east_opt(0).unwrap();
            let day = |i: i64| -> DateTime<FixedOffset> { utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap() + Duration::days(i) };
            let temperature: Vec<(DateTime<FixedOffset>, Scalar)> = (0..60).map(|i| (day(i), 8.0 + i as Scalar * 0.15 + 4.0 * (i as Scalar / 4.0).sin())).collect();
            let rainfall: Vec<(DateTime<FixedOffset>, Scalar)> = (0..60).map(|i| (day(i), (12.0 + 10.0 * (i as Scalar / 6.0).sin() + 4.0 * (i as Scalar / 2.3).cos()).max(0.0))).collect();

            let chart = Chart::new(LineController::new((
                temperature.label("Temperature (°C)"),
                rainfall.label("Rainfall (mm)").y_scale(second_id),
            )))
                .scale(ScaleId::X, TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc))
                    .title(options.x_title.clone()).tick_marks(options.tick_marks.clone()).zoom(options.x_zoom.clone()).placed(&shared));
            finish(chart, options, shared, log, second)
        }
        Data::City => {
            // People per km² fall a thousandfold from the centre to the edge, which a logarithmic
            // scale shows as a straight line; rents fall more gently.
            let density: Vec<(Scalar, Scalar)> = (0..=30).map(|km| (km as Scalar, 20_000.0 * (-(km as Scalar) / 4.5).exp())).collect();
            let rent: Vec<(Scalar, Scalar)> = (0..=30).map(|km| (km as Scalar, 35.0 - 0.8 * km as Scalar)).collect();

            let chart = Chart::new(LineController::new((
                density.label("People per km²"),
                rent.label("Rent per m²").y_scale(second_id),
            )))
                .scale(ScaleId::X, LinearScale::new(Axis::Horizontal)
                    .title(options.x_title.clone()).tick_marks(options.tick_marks.clone()).zoom(options.x_zoom.clone()).placed(&shared));
            finish(chart, options, shared, log, second)
        }
        Data::Words => {
            let uses = vec![("the", 4800.0), ("of", 2300.0), ("and", 2100.0), ("to", 2000.0), ("a", 1800.0), ("in", 1500.0), ("is", 900.0), ("you", 850.0), ("that", 800.0), ("it", 780.0)];
            let letters: Vec<(&'static str, Scalar)> = uses.iter().map(|(word, _)| (*word, word.len() as Scalar)).collect();

            let chart = Chart::new(BarController::new((
                uses.label("Uses per 100 000 words"),
                letters.label("Letters").y_scale(second_id),
            )))
                .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).grid_offset(options.grid_offset.clone()).snap_mode(options.snap_mode.clone())
                    .title(options.x_title.clone()).tick_marks(options.tick_marks.clone()).zoom(options.x_zoom.clone()).placed(&shared));
            finish(chart, options, shared, log, second)
        }
    }
}

/// `chart` with its y scale, logarithmic or linear, its second y axis on the other side when it
/// has one, and the shared options.
fn finish<C: Sequence<dyn DatasetController>>(chart: Chart<C>, options: Options, shared: Shared, log: bool, second: bool) -> Box<dyn AnyWidget> {
    let chart = chart.title(options.title.clone());

    let chart = if log {
        chart.scale(ScaleId::Y, LogScale::new(Axis::Vertical)
            .title(options.y_title.clone()).tick_marks(options.tick_marks.clone()).zoom(options.y_zoom.clone()).placed(&shared))
    } else {
        // A start or a step that isn't a number leaves the scale to fit its data.
        let min = Map1::read_map(options.from_zero.clone(), |from_zero: &bool| if *from_zero { 0.0 } else { Scalar::NAN });
        chart.scale(ScaleId::Y, LinearScale::new(Axis::Vertical).min(min).step(options.step.clone())
            .title(options.y_title.clone()).tick_marks(options.tick_marks.clone()).zoom(options.y_zoom.clone()).placed(&shared))
    };

    let chart = if second {
        chart.scale(SECOND, LinearScale::new(Axis::Vertical).position(shared.other_side(Axis::Vertical)).title("Second dataset").grid_lines(false).tick_marks(options.tick_marks.clone()).zoom(options.second_zoom.clone()).labelled(&shared))
    } else {
        chart
    };

    shared.apply(chart).boxed()
}
