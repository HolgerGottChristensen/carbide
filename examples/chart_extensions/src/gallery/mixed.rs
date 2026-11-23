//! Controllers of different kinds in one chart, a dataset on a second axis, and data that changes
//! while the chart is shown: a timer appending readings, and datasets added and removed.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use carbide::asynchronous::Timer;
use carbide::chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, DatasetExt, LinearScale, LineController, ScaleId, ZoomWindow};
use carbide::controls::button::Button;
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map2, Map4, ReadState, State};
use carbide::time::Duration;
use carbide::widget::*;
use carbide::ui;
use super::{first_section, note, section, shown_when, text_input, with_side_bar, zoom_controls, Shared, MONTHS};

/// The growth line's own y axis.
const GROWTH: ScaleId = ScaleId::new("growth");

/// What the mixed chart shows.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    /// Revenue as bars and its growth as a line, in one chart.
    RevenueAndGrowth,
    /// Readings arriving while the chart is shown.
    Live,
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    own_axis: LocalState<bool>,
    /// How many readings have arrived, which the timer counts up from another thread.
    ticks: Arc<AtomicUsize>,
    /// How many live datasets are shown.
    datasets: LocalState<usize>,
    x_zoom: LocalState<ZoomWindow>,
    y_zoom: LocalState<ZoomWindow>,
    growth_zoom: LocalState<ZoomWindow>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Mixed".to_string()),
            data: LocalState::new(Data::RevenueAndGrowth),
            own_axis: LocalState::new(true),
            ticks: Arc::new(AtomicUsize::new(60)),
            datasets: LocalState::new(1),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            growth_zoom: LocalState::new(ZoomWindow::WHOLE),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    let shown = Map2::read_map(options.data.clone(), options.own_axis.clone(), |data: &Data, own: &bool| (*data, *own));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (Data::RevenueAndGrowth, false) => revenue(o.clone(), s.clone(), false),
            (Data::RevenueAndGrowth, true) => revenue(o.clone(), s.clone(), true),
            (Data::Live, _) => live(o.clone(), s.clone()),
        }
    );

    // A reading arrives four times a second while the timer runs.
    let ticks = options.ticks.clone();
    let timer = Timer::new(move || {
        ticks.fetch_add(1, Ordering::Relaxed);
    }).interval(Duration::from_secs_f64(0.25)).repeat();
    let (start, stop) = (timer.clone(), timer);
    let datasets = options.datasets.clone();
    let (more, fewer) = (datasets.clone(), datasets.clone());

    let side = VStack::new((
        first_section("Mixed"),
        text_input("Title", &options.title),
        Picker::new("Data", data_resetting_zoom(&options), (
            Text::new("Revenue and growth").tag(Data::RevenueAndGrowth),
            Text::new("Live readings").tag(Data::Live),
        )),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::RevenueAndGrowth),
            Toggle::new("Growth on its own axis", options.own_axis.clone())),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::Live), VStack::new((
            section("Live readings"),
            HStack::new((
                Button::new("Start", move |_| { start.clone().start(); }).frame(70.0, 26.0),
                Button::new("Stop", move |_| { stop.clone().stop(); }).frame(70.0, 26.0),
            )),
            HStack::new((
                Button::new("Add a dataset", move |_| {
                    let mut more = more.clone();
                    let count = (*more.value() + 1).min(4);
                    *more.value_mut() = count;
                }).frame(120.0, 26.0),
                Button::new("Remove one", move |_| {
                    let mut fewer = fewer.clone();
                    let count = (*fewer.value()).max(2) - 1;
                    *fewer.value_mut() = count;
                }).frame(100.0, 26.0),
            )),
            Text::new(Map1::read_map(datasets, |count: &usize| format!("Live datasets: {count}"))),
            note("The line moves on as each reading arrives, keeping the last 60."),
        )).cross_axis_alignment(CrossAxisAlignment::Start)),
        VStack::new((
            section("Zoom"),
            zoom_controls("x", &options.x_zoom),
            zoom_controls("y", &options.y_zoom),
            // Growth has a scale of its own only beside the revenue, on its own axis.
            shown_when(Map2::read_map(options.data.clone(), options.own_axis.clone(), |data: &Data, own: &bool| *data == Data::RevenueAndGrowth && *own),
                zoom_controls("Growth", &options.growth_zoom)),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The data option, which shows each new choice of data whole: picking other data resets every
/// scale's zoom.
fn data_resetting_zoom(options: &Options) -> impl State<T=Data> + use<> {
    Map4::map(
        options.data.clone(), options.x_zoom.clone(), options.y_zoom.clone(), options.growth_zoom.clone(),
        |data: &Data, _: &ZoomWindow, _: &ZoomWindow, _: &ZoomWindow| *data,
        |new, mut data, mut x_zoom, mut y_zoom, mut growth_zoom| {
            if *data != new {
                *data = new;
                *x_zoom = ZoomWindow::WHOLE;
                *y_zoom = ZoomWindow::WHOLE;
                *growth_zoom = ZoomWindow::WHOLE;
            }
        },
    )
}

/// Revenue as bars and its growth as a line, the line on an axis of its own on the right, or on
/// the bars' axis.
pub(super) fn revenue(options: Options, shared: Shared, own_axis: bool) -> Box<dyn AnyWidget> {
    let revenue = vec![120.0, 135.0, 128.0, 160.0, 172.0, 168.0, 190.0, 205.0, 198.0, 220.0, 236.0, 250.0];
    let growth: Vec<Scalar> = revenue.windows(2).map(|pair| (pair[1] / pair[0] - 1.0) * 100.0).collect();
    let growth: Vec<Option<Scalar>> = std::iter::once(None).chain(growth.into_iter().map(Some)).collect();

    let growth_scale = if own_axis { GROWTH } else { ScaleId::Y };
    let chart = Chart::new((
        BarController::new(revenue.label("Revenue (k)")),
        // Its own colours start after the bars', as each controller counts its datasets from 0.
        LineController::new(growth.label("Growth (%)").y_scale(growth_scale)).palette_offset(1),
    ))
        .title(options.title.clone())
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(MONTHS).zoom(options.x_zoom.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(options.y_zoom.clone()));

    let chart = if own_axis {
        chart.scale(GROWTH, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).title("Growth (%)").grid_lines(false).zoom(options.growth_zoom.clone()))
    } else {
        chart
    };

    shared.apply(chart).boxed()
}

/// The live readings: the last 60 of each dataset, moving on as the timer counts.
pub(super) fn live(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    let ticks = options.ticks.clone();
    let readings = Map1::read_map(options.datasets.clone(), move |count: &usize| {
        let now = ticks.load(Ordering::Relaxed);
        (0..*count).map(|dataset| {
            let phase = dataset as Scalar * 17.0;
            (now.saturating_sub(60)..now).map(|i| {
                let x = i as Scalar;
                let noise = ((x * 12.9898 + phase).sin() * 43758.5453).fract() * 4.0;
                (x, 30.0 + dataset as Scalar * 15.0 + 10.0 * ((x + phase) / 9.0).sin() + noise)
            }).collect::<Vec<(Scalar, Scalar)>>()
        }).collect::<Vec<Vec<(Scalar, Scalar)>>>()
    });
    shared.apply(Chart::new(LineController::new(readings))
        .title(options.title.clone())
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).title("Reading").zoom(options.x_zoom.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(options.y_zoom.clone())))
        .boxed()
}

