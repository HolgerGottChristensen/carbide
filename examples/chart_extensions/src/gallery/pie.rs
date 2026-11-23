//! Pies and doughnuts: rings, gaps, outlines, labels, a sweep of less than a full turn, and slices
//! sized by value.

use carbide::chart::{AngleDirection, Axis, CategoryScale, Chart, DatasetExt, LinearScale, PieController, PieDataSet, ScaleId, SizedDataSetExt};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::{Angle, Scalar};
use carbide::state::{LocalState, Map1, Map2};
use carbide::widget::*;
use carbide::ui;
use super::{first_section, degrees, percent, shown_when, slider, text_input, with_side_bar, Shared};

/// The data the pie can show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    Spending,
    Budget,
    Many,
    /// Each slice reaching out as far as its size says (`radii`).
    Radii,
}

impl Data {
    /// Each ring's name and parts: a name, a value, and a size, which only `Radii` draws.
    fn data(self) -> Vec<(&'static str, Vec<(&'static str, Scalar, Scalar)>)> {
        let parts = |parts: &[(&'static str, Scalar)]| parts.iter().map(|(name, value)| (*name, *value, 1.0)).collect();
        match self {
            Data::Spending => vec![
                ("2025", parts(&[("Rent", 45.0), ("Food", 25.0), ("Travel", 15.0), ("Other", 15.0)])),
                ("2026", parts(&[("Rent", 40.0), ("Food", 30.0), ("Travel", 10.0), ("Other", 20.0)])),
            ],
            Data::Budget => vec![
                ("Budget", parts(&[("Housing", 32.0), ("Transport", 15.0), ("Food", 13.0), ("Health", 9.0), ("Savings", 20.0), ("Fun", 11.0)])),
            ],
            Data::Many => vec![
                ("Languages", parts(&[
                    ("Rust", 18.0), ("Go", 12.0), ("Python", 22.0), ("Java", 9.0), ("C", 6.0), ("C++", 7.0), ("Swift", 4.0), ("Kotlin", 3.0),
                    ("Ruby", 2.0), ("Zig", 1.5), ("Haskell", 1.0), ("OCaml", 0.8), ("Elixir", 1.2), ("Lua", 0.9), ("Odin", 0.6),
                ])),
            ],
            // Sales shares, each region's slice reaching out by its growth.
            Data::Radii => vec![
                ("Regions", vec![("North", 30.0, 9.0), ("East", 20.0, 4.5), ("South", 35.0, 6.0), ("West", 15.0, 7.5)]),
            ],
        }
    }
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    inner_radius: LocalState<Scalar>,
    start_angle: LocalState<Scalar>,
    sweep: LocalState<Scalar>,
    direction: LocalState<AngleDirection>,
    spacing: LocalState<Scalar>,
    ring_spacing: LocalState<Scalar>,
    ring_size: LocalState<Scalar>,
    outline_width: LocalState<Scalar>,
    labels: LocalState<bool>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Shares".to_string()),
            data: LocalState::new(Data::Spending),
            inner_radius: LocalState::new(0.4),
            start_angle: LocalState::new(0.0),
            sweep: LocalState::new(360.0),
            direction: LocalState::new(AngleDirection::Clockwise),
            spacing: LocalState::new(0.0),
            ring_spacing: LocalState::new(4.0),
            ring_size: LocalState::new(1.0),
            outline_width: LocalState::new(2.0),
            labels: LocalState::new(true),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let o = options.clone();
    let s = shared.clone();

    let chart = ui!(
        match data {
            Data::Spending => chart(o.clone(), s.clone(), Data::Spending),
            Data::Budget => chart(o.clone(), s.clone(), Data::Budget),
            Data::Many => chart(o.clone(), s.clone(), Data::Many),
            Data::Radii => chart(o.clone(), s.clone(), Data::Radii),
        }
    );

    let side = VStack::new((
        first_section("Pie"),
        text_input("Title", &options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Spending, two years").tag(Data::Spending),
            Text::new("A budget").tag(Data::Budget),
            Text::new("Many small parts").tag(Data::Many),
            Text::new("Slices sized by growth").tag(Data::Radii),
        )),
        slider("Hole", &options.inner_radius, 0.0, 0.9, 0.05, percent),
        slider("Start", &options.start_angle, 0.0, 360.0, 5.0, degrees),
        slider("Sweep", &options.sweep, 30.0, 360.0, 5.0, degrees),
        Picker::new("Direction", options.direction.clone(), (
            Text::new("Clockwise").tag(AngleDirection::Clockwise),
            Text::new("Counter-clockwise").tag(AngleDirection::CounterClockwise),
        )),
        slider("Gaps", &options.spacing, 0.0, 20.0, 1.0, super::pixels),
        // Only the spending has more than one ring.
        shown_when(
            Map1::read_map(options.data.clone(), |data: &Data| *data == Data::Spending),
            VStack::new((
                slider("Ring gaps", &options.ring_spacing, 0.0, 30.0, 1.0, super::pixels),
                slider("Ring size", &options.ring_size, 0.2, 1.0, 0.05, percent),
            )).cross_axis_alignment(CrossAxisAlignment::Start)
        ),
        slider("Outline", &options.outline_width, 0.0, 8.0, 0.5, |pixels| format!("{pixels:.1} px")),
        Toggle::new("Labels", options.labels.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The pie of `data`.
pub(super) fn chart(options: Options, shared: Shared, data: Data) -> Box<dyn AnyWidget> {
    let sized = data == Data::Radii;

    let rings: Vec<Box<dyn PieDataSet>> = data.data()
        .into_iter()
        .map(|(name, parts)| {
            if sized {
                parts.radii()
                    .label(name)
                    .boxed::<dyn PieDataSet>()
            } else {
                let parts: Vec<(&'static str, Scalar)> = parts.into_iter()
                    .map(|(part, value, _)| (part, value))
                    .collect();

                parts.label(name)
                    .boxed::<dyn PieDataSet>()
            }
        })
        .collect();

    let pie = PieController::new(rings)
        .spacing(options.spacing.clone())
        .ring_spacing(options.ring_spacing.clone())
        .category_percentage(options.ring_size.clone())
        .outline_width(options.outline_width.clone())
        .labels(options.labels.clone());

    // The circle ends the sweep on from where it starts.
    shared.apply(Chart::new(pie)
        .title(options.title.clone())
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).inner_radius(options.inner_radius.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial)
            .start_angle(Map1::read_map(options.start_angle.clone(), |degrees: &Scalar| Angle::Degrees(*degrees)))
            .end_angle(Map2::read_map(options.start_angle.clone(), options.sweep.clone(), |start: &Scalar, sweep: &Scalar| Angle::Degrees(start + sweep)))
            .direction(options.direction.clone())))
        .boxed()
}
