//! Around a circle: radars, radial bars of one series or of several side by side, stacked or
//! stacked to 100%, with the widths of their slots and bars, and scattered points, with the circle's start, sweep, direction and hole, and its rings as polygons
//! or circles.

use carbide::chart::{AngleDirection, Axis, BarController, BarDataSet, CategoryScale, Chart, DataColor, DatasetExt, Fill, GridShape, Interpolation, LinearScale, LineController, LineDataSet, ScaleId, ScatterController, StackId, Stepped};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::{Angle, Scalar};
use carbide::state::{LocalState, Map1, Map2};
use carbide::widget::*;
use carbide::ui;
use super::bars::Stacking;
use super::{first_section, degrees, percent, section, shown_when, slider, text_input, with_side_bar, Shared, MONTHS};

const SKILLS: [&str; 6] = ["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"];

/// What is drawn around the circle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    /// Two players' skills as a radar.
    Players,
    /// Four players' skills as a radar.
    Squad,
    /// Rainfall by month as wedges, one colour per month.
    RadialBars,
    /// Rainfall by month in three cities as wedges, side by side, stacked or stacked to 100%.
    CitiesRadialBars,
    /// Wind readings: the direction it came from in degrees and its speed.
    Wind,
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    grid_shape: LocalState<GridShape>,
    fill: LocalState<Fill>,
    interpolation: LocalState<Interpolation>,
    start_angle: LocalState<Scalar>,
    sweep: LocalState<Scalar>,
    direction: LocalState<AngleDirection>,
    inner_radius: LocalState<Scalar>,
    horizontal: LocalState<bool>,
    stacking: LocalState<Stacking>,
    category_percentage: LocalState<Scalar>,
    bar_percentage: LocalState<Scalar>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Around a circle".to_string()),
            data: LocalState::new(Data::Players),
            grid_shape: LocalState::new(GridShape::Polygon),
            fill: LocalState::new(Fill::Origin),
            interpolation: LocalState::new(Interpolation::Linear),
            start_angle: LocalState::new(0.0),
            sweep: LocalState::new(360.0),
            direction: LocalState::new(AngleDirection::Clockwise),
            inner_radius: LocalState::new(0.0),
            horizontal: LocalState::new(false),
            stacking: LocalState::new(Stacking::Grouped),
            category_percentage: LocalState::new(0.8),
            bar_percentage: LocalState::new(0.9),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            Data::Players => chart(o.clone(), s.clone(), Data::Players),
            Data::Squad => chart(o.clone(), s.clone(), Data::Squad),
            Data::RadialBars => chart(o.clone(), s.clone(), Data::RadialBars),
            Data::CitiesRadialBars => chart(o.clone(), s.clone(), Data::CitiesRadialBars),
            Data::Wind => chart(o.clone(), s.clone(), Data::Wind),
        }
    );

    let side = VStack::new((
        first_section("Polar"),
        text_input("Title", &options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Radar of two players").tag(Data::Players),
            Text::new("Radar of a squad of four").tag(Data::Squad),
            Text::new("Rainfall as radial bars").tag(Data::RadialBars),
            Text::new("Rainfall in three cities").tag(Data::CitiesRadialBars),
            Text::new("Wind as scattered points").tag(Data::Wind),
        )),
        // Rings are polygons only around categories: not around the wind's degrees, nor when the
        // categories run out from the centre instead.
        shown_when(Map2::read_map(options.data.clone(), options.horizontal.clone(), |data: &Data, horizontal: &bool| *data != Data::Wind && !*horizontal),
            Picker::new("Rings", options.grid_shape.clone(), (
                Text::new("Polygons").tag(GridShape::Polygon),
                Text::new("Circles").tag(GridShape::Circle),
            ))),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| matches!(data, Data::Players | Data::Squad)), VStack::new((
            Picker::new("Radar fill", options.fill.clone(), (
                Text::new("None").tag(Fill::None),
                Text::new("To the centre").tag(Fill::Origin),
                Text::new("To the rim").tag(Fill::Top),
            )),
            Picker::new("Radar curve", options.interpolation.clone(), (
                Text::new("Straight").tag(Interpolation::Linear),
                Text::new("Smooth").tag(Interpolation::Smooth { tension: 0.4 }),
                Text::new("Steps").tag(Interpolation::Stepped(Stepped::Middle)),
            )),
        )).cross_axis_alignment(CrossAxisAlignment::Start)),
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| matches!(data, Data::RadialBars | Data::CitiesRadialBars)), VStack::new((
            shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data == Data::CitiesRadialBars),
                Picker::new("Cities", options.stacking.clone(), (
                    Text::new("Side by side").tag(Stacking::Grouped),
                    Text::new("Stacked").tag(Stacking::Stacked),
                    Text::new("Stacked to 100%").tag(Stacking::Percent),
                ))),
            slider("Group", &options.category_percentage, 0.1, 1.0, 0.05, percent),
            slider("Bar", &options.bar_percentage, 0.1, 1.0, 0.05, percent),
        )).cross_axis_alignment(CrossAxisAlignment::Start)),
        section("Circle"),
        slider("Start", &options.start_angle, 0.0, 360.0, 5.0, degrees),
        slider("Sweep", &options.sweep, 30.0, 360.0, 5.0, degrees),
        Picker::new("Direction", options.direction.clone(), (
            Text::new("Clockwise").tag(AngleDirection::Clockwise),
            Text::new("Counter-clockwise").tag(AngleDirection::CounterClockwise),
        )),
        slider("Hole", &options.inner_radius, 0.0, 0.9, 0.05, percent),
        // Scattered points have no categories to lay out along the radius.
        shown_when(Map1::read_map(options.data.clone(), |data: &Data| *data != Data::Wind),
            Toggle::new("Categories out from the centre", options.horizontal.clone())),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// What `data` draws around the circle: on an angle scale whose start, sweep and direction, and a
/// radius scale whose hole and rings, follow the options.
pub(super) fn chart(options: Options, shared: Shared, data: Data) -> Box<dyn AnyWidget> {
    let start = || Map1::read_map(options.start_angle.clone(), |degrees: &Scalar| Angle::Degrees(*degrees));
    let end = || Map2::read_map(options.start_angle.clone(), options.sweep.clone(), |start: &Scalar, sweep: &Scalar| Angle::Degrees(start + sweep));
    // When horizontal, the categories run out along the radius and the values around the circle, so the
    // scales swap roles. Each scale is given the options of both roles, and reads those of the
    // role it has: the start, sweep and direction as the angle, the hole and rings as the radius.
    let values = LinearScale::new(Axis::Radial).min(0.0)
        .grid_shape(options.grid_shape.clone()).inner_radius(options.inner_radius.clone())
        .start_angle(start()).end_angle(end()).direction(options.direction.clone());

    match data {
        Data::Players | Data::Squad => {
            let mut skills = vec![
                ("Ada", vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]),
                ("Bo", vec![5.0, 8.0, 6.0, 4.0, 7.0, 9.0]),
            ];
            if data == Data::Squad {
                skills.push(("Cy", vec![9.0, 4.0, 8.0, 6.0, 8.0, 3.0]));
                skills.push(("Dee", vec![4.0, 9.0, 5.0, 7.0, 3.0, 8.0]));
            }
            let players: Vec<Box<dyn LineDataSet>> = skills.into_iter()
                .map(|(name, values)| values.label(name).fill(options.fill.clone()).interpolation(options.interpolation.clone()).boxed::<dyn LineDataSet>())
                .collect();

            shared.apply(Chart::new(LineController::new(players).horizontal(options.horizontal.clone()))
                .title(options.title.clone())
                .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS).start_angle(start()).end_angle(end()).direction(options.direction.clone())
                    .grid_shape(options.grid_shape.clone()).inner_radius(options.inner_radius.clone()))
                .scale(ScaleId::Y, values))
                .boxed()
        }
        Data::RadialBars => {
            let rainfall = vec![42.0, 35.0, 48.0, 60.0, 55.0, 70.0, 66.0, 50.0, 47.0, 58.0, 61.0, 52.0]
                .label("Rainfall (mm)")
                .color(DataColor::ByPoint);

            let bars = BarController::new(rainfall)
                .horizontal(options.horizontal.clone())
                .category_percentage(options.category_percentage.clone())
                .bar_percentage(options.bar_percentage.clone());

            shared.apply(Chart::new(bars)
                .title(options.title.clone())
                .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(MONTHS).start_angle(start()).end_angle(end()).direction(options.direction.clone())
                    .grid_shape(options.grid_shape.clone()).inner_radius(options.inner_radius.clone()))
                .scale(ScaleId::Y, values))
                .boxed()
        }
        Data::CitiesRadialBars => {
            let cities = [
                ("Copenhagen", vec![46.0, 30.0, 39.0, 32.0, 43.0, 52.0, 66.0, 66.0, 60.0, 57.0, 58.0, 55.0]),
                ("London", vec![55.0, 41.0, 41.0, 44.0, 49.0, 45.0, 45.0, 50.0, 49.0, 69.0, 59.0, 55.0]),
                ("Paris", vec![51.0, 41.0, 48.0, 52.0, 63.0, 50.0, 63.0, 55.0, 47.0, 62.0, 51.0, 58.0]),
            ];
            // Side by side, each city is a stack of its own, sharing its month's wedge; stacked,
            // they share one stack, each city's wedge starting where the one before it ends, and
            // to 100%, each month's stack is scaled to reach the rim.
            let datasets: Vec<Box<dyn BarDataSet>> = cities.into_iter().map(|(name, values)| {
                let stack = Map1::read_map(options.stacking.clone(), move |stacking: &Stacking| match stacking {
                    Stacking::Grouped => StackId::new(name),
                    Stacking::Stacked => StackId::new("all"),
                    Stacking::Percent => StackId::new("all").normalized(),
                });
                values.label(name).stack(stack).boxed::<dyn BarDataSet>()
            }).collect();

            let bars = BarController::new(datasets)
                .horizontal(options.horizontal.clone())
                .category_percentage(options.category_percentage.clone())
                .bar_percentage(options.bar_percentage.clone());

            shared.apply(Chart::new(bars)
                .title(options.title.clone())
                .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(MONTHS).start_angle(start()).end_angle(end()).direction(options.direction.clone())
                    .grid_shape(options.grid_shape.clone()).inner_radius(options.inner_radius.clone()))
                .scale(ScaleId::Y, values))
                .boxed()
        }
        Data::Wind => {
            // The direction the wind came from, and its speed, at two stations.
            let station = |seed: Scalar, spread: Scalar| -> Vec<(Scalar, Scalar)> {
                (0..30).map(|i| {
                    let i = i as Scalar + seed;
                    // From 0 to 1: `fract` keeps the sign, so a negative sine would give a
                    // negative jitter, and a speed below zero.
                    let jitter = |k: Scalar| ((i * k).sin() * 43758.5453).rem_euclid(1.0);
                    ((220.0 + (jitter(12.9898) - 0.5) * spread).rem_euclid(360.0), 2.0 + jitter(78.233) * 12.0)
                }).collect()
            };

            shared.apply(Chart::new(ScatterController::new((station(0.0, 120.0).label("Coast"), station(50.0, 300.0).label("Hills"))))
                .title(options.title.clone())
                .scale(ScaleId::X, LinearScale::new(Axis::Radial).min(0.0).max(360.0).step(45.0).start_angle(start()).end_angle(end()).direction(options.direction.clone())
                    .grid_shape(options.grid_shape.clone()).inner_radius(options.inner_radius.clone()))
                .scale(ScaleId::Y, values))
                .boxed()
        }
    }
}

