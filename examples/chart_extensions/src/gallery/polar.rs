//! Around a circle: radars, radial bars and scattered points, with the circle's start, sweep,
//! direction and hole, and its rings as polygons or circles.

use carbide::chart::{AngleDirection, Axis, BarController, CategoryScale, Chart, DataColor, DatasetExt, Fill, GridShape, Interpolation, LinearScale, LineController, LineDataSet, ScaleId, ScatterController, Stepped};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::{Angle, Scalar};
use carbide::state::{LocalState, Map1, Map2};
use carbide::widget::*;
use carbide::ui;
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
            Text::new("Wind as scattered points").tag(Data::Wind),
        )),
        // Rings are polygons only around categories: not around the wind's degrees, nor when the
        // categories run out from the centre instead.
        shown_when(Map2::read_map(options.data.clone(), options.horizontal.clone(), |data: &Data, lying: &bool| *data != Data::Wind && !*lying),
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
    // Lying, the categories run out along the radius and the values around the circle, so the
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

            shared.apply(Chart::new(BarController::new(rainfall).horizontal(options.horizontal.clone()))
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
                    let jitter = |k: Scalar| ((i * k).sin() * 43758.5453).fract();
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

