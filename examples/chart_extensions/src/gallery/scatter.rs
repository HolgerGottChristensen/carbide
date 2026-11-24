//! Scatter: points placed freely by two values, as clouds of points, a path through them in data
//! order, points coloured one by one, at times on a time scale, named by text, and bubbles sized
//! by a third value, with or without a line through them, zooming along both scales.

use carbide::chart::{Axis, CategoryScale, Chart, Sequence, DataColor, DatasetExt, LinearScale, ScaleId, ScatterController, ScatterDataSet, SizedDataSetExt, TimeScale, TimeScaleZone, ZoomWindow, DatasetController};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map3, State};
use carbide::widget::*;
use carbide::ui;
use chrono::{DateTime, Duration, FixedOffset, TimeZone};
use super::{first_section, pixels, section, shown_when, slider, text_input, with_side_bar, zoom_controls, Shared};

/// What the scatter chart shows.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    /// Two clouds of plain points: the heights and weights of two groups.
    Points,
    /// One path whose x doubles back, as a line through the points joins them in data order.
    Path,
    /// One dataset, each point in a colour of its own.
    ColouredPoints,
    /// Earthquakes in two regions over a week, at times on a time scale.
    Times,
    /// Reaction times on each weekday, named by the data on a category scale.
    Weekdays,
    /// Stores as bubbles, sized by their sales.
    Stores,
    /// Three clusters of bubbles.
    Clusters,
}

impl Data {
    /// Whether the data is bubbles, sized by a third value.
    fn is_bubbles(self) -> bool {
        matches!(self, Data::Stores | Data::Clusters)
    }
}

/// A repeatable number from 0 to 1 for `i`, different for each `k`: the scattering of the data.
fn jitter(i: Scalar, k: Scalar) -> Scalar {
    // `fract` keeps the sign, so a negative sine would give a negative jitter.
    ((i * k).sin() * 43758.5453).rem_euclid(1.0)
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    x_zoom: LocalState<ZoomWindow>,
    y_zoom: LocalState<ZoomWindow>,
    grid_lines: LocalState<bool>,
    show_line: LocalState<bool>,
    min_radius: LocalState<Scalar>,
    max_radius: LocalState<Scalar>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Scatter".to_string()),
            data: LocalState::new(Data::Points),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            grid_lines: LocalState::new(true),
            show_line: LocalState::new(false),
            min_radius: LocalState::new(4.0),
            max_radius: LocalState::new(30.0),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            Data::Points => chart(o.clone(), s.clone(), Data::Points),
            Data::Path => chart(o.clone(), s.clone(), Data::Path),
            Data::ColouredPoints => chart(o.clone(), s.clone(), Data::ColouredPoints),
            Data::Times => chart(o.clone(), s.clone(), Data::Times),
            Data::Weekdays => chart(o.clone(), s.clone(), Data::Weekdays),
            Data::Stores => chart(o.clone(), s.clone(), Data::Stores),
            Data::Clusters => chart(o.clone(), s.clone(), Data::Clusters),
        }
    );

    let side = VStack::new((
        first_section("Scatter"),
        text_input("Title", &options.title),
        Picker::new("Data", data_resetting_zoom(&options), (
            Text::new("Heights and weights").tag(Data::Points),
            Text::new("A path that doubles back").tag(Data::Path),
            Text::new("Each point its own colour").tag(Data::ColouredPoints),
            Text::new("Earthquakes, at times").tag(Data::Times),
            Text::new("Reaction times by weekday").tag(Data::Weekdays),
            Text::new("Stores as bubbles").tag(Data::Stores),
            Text::new("Three clusters of bubbles").tag(Data::Clusters),
        )),
        Toggle::new("Line through them", options.show_line.clone()),
        // Only bubbles have sizes to fit between the smallest and the largest radius.
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| data.is_bubbles()), VStack::new((
            slider("Smallest", &options.min_radius, 1.0, 20.0, 1.0, pixels),
            slider("Largest", &options.max_radius, 10.0, 60.0, 1.0, pixels),
        )).cross_axis_alignment(CrossAxisAlignment::Start)),
        Toggle::new("Grid lines", options.grid_lines.clone()),
        VStack::new((
            section("Zoom"),
            zoom_controls("x", &options.x_zoom),
            zoom_controls("y", &options.y_zoom),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The data option, which shows each new choice of data whole: picking other data resets the
/// zoom along both scales.
fn data_resetting_zoom(options: &Options) -> impl State<T=Data> + use<> {
    Map3::map(
        options.data.clone(), options.x_zoom.clone(), options.y_zoom.clone(),
        |data: &Data, _: &ZoomWindow, _: &ZoomWindow| *data,
        |new, mut data, mut x_zoom, mut y_zoom| {
            if *data != new {
                *data = new;
                *x_zoom = ZoomWindow::WHOLE;
                *y_zoom = ZoomWindow::WHOLE;
            }
        },
    )
}

/// The scatter chart of `data`, its x scale linear, a time scale or categories, as its x values
/// are numbers, times or text.
pub(super) fn chart(options: Options, shared: Shared, data: Data) -> Box<dyn AnyWidget> {
    let linear_x = || LinearScale::new(Axis::Horizontal).zoom(options.x_zoom.clone()).grid_lines(options.grid_lines.clone());

    match data {
        Data::Points => {
            // Heights in cm and weights in kg, the weight growing with the height.
            let group = |seed: Scalar, height: Scalar, weight: Scalar| -> Vec<(Scalar, Scalar)> {
                (0..40).map(|i| {
                    let i = i as Scalar + seed;
                    let h = height + (jitter(i, 12.9898) - 0.5) * 24.0;
                    (h, weight + (h - height) * 0.9 + (jitter(i, 78.233) - 0.5) * 16.0)
                }).collect()
            };
            let scatter = ScatterController::new((group(0.0, 168.0, 64.0).label("Group A"), group(100.0, 180.0, 78.0).label("Group B")));
            finish(Chart::new(scatter.show_line(options.show_line.clone())).scale(ScaleId::X, linear_x()), options, shared)
        }
        Data::Path => {
            // A figure of eight, traced once: its x turns back twice, so a line through the points
            // in data order crosses itself.
            let path: Vec<(Scalar, Scalar)> = (0..=40).map(|i| {
                let t = i as Scalar / 40.0 * std::f64::consts::TAU;
                (10.0 * t.sin(), 6.0 * (2.0 * t).sin())
            }).collect();
            let scatter = ScatterController::new(path.label("Path"));
            finish(Chart::new(scatter.show_line(options.show_line.clone())).scale(ScaleId::X, linear_x()), options, shared)
        }
        Data::ColouredPoints => {
            let points: Vec<(Scalar, Scalar)> = (0..12).map(|i| {
                let i = i as Scalar;
                (i + jitter(i, 12.9898) * 2.0, 3.0 + i * 0.6 + (jitter(i, 78.233) - 0.5) * 4.0)
            }).collect();
            let scatter = ScatterController::new(points.label("Samples").color(DataColor::ByPoint));
            finish(Chart::new(scatter.show_line(options.show_line.clone())).scale(ScaleId::X, linear_x()), options, shared)
        }
        Data::Times => {
            let utc = FixedOffset::east_opt(0).unwrap();
            let start = utc.with_ymd_and_hms(2026, 3, 2, 0, 0, 0).unwrap();
            // Earthquakes at scattered times over a week, with their magnitudes.
            let region = |seed: Scalar, count: usize, strength: Scalar| -> Vec<(DateTime<FixedOffset>, Scalar)> {
                let mut quakes: Vec<(DateTime<FixedOffset>, Scalar)> = (0..count).map(|i| {
                    let i = i as Scalar + seed;
                    let minutes = (jitter(i, 12.9898) * 7.0 * 24.0 * 60.0) as i64;
                    (start + Duration::minutes(minutes), strength + jitter(i, 78.233) * 3.0)
                }).collect();
                // In time order, so a line through them runs forwards.
                quakes.sort_by_key(|(time, _)| *time);
                quakes
            };
            let scatter = ScatterController::new((region(0.0, 30, 2.0).label("Coast"), region(100.0, 15, 3.0).label("Mountains")));
            let x = TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc)).zoom(options.x_zoom.clone()).grid_lines(options.grid_lines.clone());
            finish(Chart::new(scatter.show_line(options.show_line.clone())).scale(ScaleId::X, x), options, shared)
        }
        Data::Weekdays => {
            // Several readings a day, so each weekday is a strip of points; the evening group
            // has no Monday, and names its days after the morning's.
            const DAYS: [&str; 5] = ["Mon", "Tue", "Wed", "Thu", "Fri"];
            let readings = |seed: Scalar, days: &[&'static str], base: Scalar| -> Vec<(&'static str, Scalar)> {
                days.iter().enumerate().flat_map(|(day, name)| {
                    (0..6).map(move |reading| {
                        let i = (day * 6 + reading) as Scalar + seed;
                        (*name, base + day as Scalar * 8.0 + jitter(i, 12.9898) * 60.0)
                    })
                }).collect()
            };
            let scatter = ScatterController::new((readings(0.0, &DAYS, 250.0).label("Morning (ms)"), readings(100.0, &DAYS[1..], 290.0).label("Evening (ms)")));
            let x = CategoryScale::new(Axis::Horizontal).zoom(options.x_zoom.clone()).grid_lines(options.grid_lines.clone());
            finish(Chart::new(scatter.show_line(options.show_line.clone())).scale(ScaleId::X, x), options, shared)
        }
        Data::Stores | Data::Clusters => {
            let datasets: Vec<Box<dyn ScatterDataSet>> = bubbles(data).into_iter()
                .map(|(name, bubbles)| bubbles.bubbles().label(name).boxed::<dyn ScatterDataSet>())
                .collect();
            let scatter = ScatterController::new(datasets)
                .show_line(options.show_line.clone())
                .radius_range(options.min_radius.clone(), options.max_radius.clone());
            finish(Chart::new(scatter).scale(ScaleId::X, linear_x()), options, shared)
        }
    }
}

/// Each dataset's name and bubbles, x, y and size, for the bubble data.
fn bubbles(data: Data) -> Vec<(&'static str, Vec<(Scalar, Scalar, Scalar)>)> {
    match data {
        Data::Clusters => {
            // Three groups scattered about their centres, with spread-out sizes.
            let cluster = |cx: Scalar, cy: Scalar, seed: Scalar| -> Vec<(Scalar, Scalar, Scalar)> {
                (0..20).map(|i| {
                    let i = i as Scalar + seed;
                    (cx + (jitter(i, 12.9898) - 0.5) * 6.0, cy + (jitter(i, 78.233) - 0.5) * 6.0, 2.0 + jitter(i, 39.425) * 50.0)
                }).collect()
            };
            vec![("Alpha", cluster(10.0, 10.0, 0.0)), ("Beta", cluster(25.0, 20.0, 100.0)), ("Gamma", cluster(20.0, 5.0, 200.0))]
        }
        _ => vec![
            ("North", (0..25).map(|i| {
                let i = i as Scalar;
                ((i * 7.3) % 20.0, (i * 3.7) % 15.0, 5.0 + (i * 11.0) % 40.0)
            }).collect()),
            ("South", (0..15).map(|i| {
                let i = i as Scalar;
                ((i * 5.1 + 2.0) % 20.0, (i * 6.3 + 4.0) % 15.0, 10.0 + (i * 13.0) % 30.0)
            }).collect()),
        ],
    }
}

/// `chart` with its title, and a linear y scale zooming and showing grid lines as the options say.
fn finish<C: Sequence<dyn DatasetController>>(chart: Chart<C>, options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    shared.apply(chart
        .title(options.title.clone())
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(options.y_zoom.clone()).grid_lines(options.grid_lines.clone())))
        .boxed()
}
