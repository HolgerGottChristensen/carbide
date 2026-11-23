//! Scales: time, linear, logarithmic and category scales, a second y axis for one dataset, axes on
//! either side with titles and tick marks, a fixed start and step, and grid lines between
//! categories.

use carbide::chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, ControllerSequence, DatasetExt, LinearScale, LineController, LogScale, ScaleId, TimeScale, TimeScaleZone};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map3};
use carbide::widget::*;
use carbide::ui;
use chrono::{DateTime, Duration, FixedOffset, TimeZone};
use super::{first_section, section, shown_when, slider, text_input, with_side_bar, Shared};

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
    x_position: LocalState<AxisPosition>,
    y_position: LocalState<AxisPosition>,
    x_title: LocalState<String>,
    y_title: LocalState<String>,
    tick_marks: LocalState<bool>,
    from_zero: LocalState<bool>,
    step: LocalState<Scalar>,
    grid_offset: LocalState<bool>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Scales".to_string()),
            data: LocalState::new(Data::Weather),
            log: LocalState::new(false),
            second_axis: LocalState::new(true),
            x_position: LocalState::new(AxisPosition::Bottom),
            y_position: LocalState::new(AxisPosition::Left),
            x_title: LocalState::new("x".to_string()),
            y_title: LocalState::new("y".to_string()),
            tick_marks: LocalState::new(true),
            from_zero: LocalState::new(false),
            step: LocalState::new(0.0),
            grid_offset: LocalState::new(true),
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
        Picker::new("Data", options.data.clone(), (
            Text::new("Weather, on a time scale").tag(Data::Weather),
            Text::new("A city by distance, on a linear scale").tag(Data::City),
            Text::new("Words, named by the data").tag(Data::Words),
        )),
        Toggle::new("y on a logarithmic scale", options.log.clone()),
        Toggle::new("The second dataset on its own axis", options.second_axis.clone()),
        VStack::new((
            section("Axes"),
            Picker::new("x side", options.x_position.clone(), (
                Text::new("Bottom").tag(AxisPosition::Bottom),
                Text::new("Top").tag(AxisPosition::Top),
            )),
            Picker::new("y side", options.y_position.clone(), (
                Text::new("Left").tag(AxisPosition::Left),
                Text::new("Right").tag(AxisPosition::Right),
            )),
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
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
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
                    .position(options.x_position.clone()).title(options.x_title.clone()).tick_marks(options.tick_marks.clone()));
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
                    .position(options.x_position.clone()).title(options.x_title.clone()).tick_marks(options.tick_marks.clone()));
            finish(chart, options, shared, log, second)
        }
        Data::Words => {
            let uses = vec![("the", 4800.0), ("of", 2300.0), ("and", 2100.0), ("to", 2000.0), ("a", 1800.0), ("in", 1500.0), ("is", 900.0), ("you", 850.0), ("that", 800.0), ("it", 780.0)];
            let letters: Vec<(&'static str, Scalar)> = uses.iter().map(|(word, _)| (*word, word.len() as Scalar)).collect();

            let chart = Chart::new(BarController::new((
                uses.label("Uses per 100 000 words"),
                letters.label("Letters").y_scale(second_id),
            )))
                .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).grid_offset(options.grid_offset.clone())
                    .position(options.x_position.clone()).title(options.x_title.clone()).tick_marks(options.tick_marks.clone()));
            finish(chart, options, shared, log, second)
        }
    }
}

/// `chart` with its y scale, logarithmic or linear, its second y axis on the other side when it
/// has one, and the shared options.
fn finish<C: ControllerSequence>(chart: Chart<C>, options: Options, shared: Shared, log: bool, second: bool) -> Box<dyn AnyWidget> {
    let chart = chart.title(options.title.clone());

    let chart = if log {
        chart.scale(ScaleId::Y, LogScale::new(Axis::Vertical)
            .position(options.y_position.clone()).title(options.y_title.clone()).tick_marks(options.tick_marks.clone()))
    } else {
        // A start or a step that isn't a number leaves the scale to fit its data.
        let min = Map1::read_map(options.from_zero.clone(), |from_zero: &bool| if *from_zero { 0.0 } else { Scalar::NAN });
        chart.scale(ScaleId::Y, LinearScale::new(Axis::Vertical).min(min).step(options.step.clone())
            .position(options.y_position.clone()).title(options.y_title.clone()).tick_marks(options.tick_marks.clone()))
    };

    let chart = if second {
        let other_side = Map1::read_map(options.y_position.clone(), |side: &AxisPosition| if *side == AxisPosition::Left { AxisPosition::Right } else { AxisPosition::Left });
        chart.scale(SECOND, LinearScale::new(Axis::Vertical).position(other_side).title("Second dataset").grid_lines(false).tick_marks(options.tick_marks.clone()))
    } else {
        chart
    };

    shared.apply(chart).boxed()
}
