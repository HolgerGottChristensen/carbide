//! Lines: curves, steps and their tension, fills to every kind of baseline, stacks, gaps and how
//! they are bridged, and zooming along both scales.

use carbide::chart::{Axis, Chart, DatasetExt, Fill, Interpolation, LinearScale, LineController, LineDataSet, ScaleId, SpanGaps, StackId, Stepped, ZoomWindow};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map2};
use carbide::widget::*;
use carbide::ui;
use super::{first_section, section, shown_when, slider, text_input, with_side_bar, zoom_controls, Shared};

/// The data the line chart can show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    /// The two lines of the `carbide` crate's chart example (`carbide/examples/chart/chart.rs`).
    CarbideExample,
    Visitors,
    Gaps,
    Dense,
}

/// The kind of curve through the points; smooth curves take the tension slider's tension.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Curve {
    Straight,
    Smooth,
    Monotone,
    StepsBefore,
    StepsMiddle,
    StepsMiddleVertical,
    StepsAfter,
}

/// How a line goes on past a missing value.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Gaps {
    Break,
    Bridge,
    /// Bridge gaps up to the gap slider's width along x.
    BridgeWithin,
}

impl Data {
    /// Each dataset's name and values, at their indices; `None` is a missing value.
    fn data(self) -> Vec<(&'static str, Vec<Option<Scalar>>)> {
        let wave = |count: usize, phase: Scalar, level: Scalar| -> Vec<Option<Scalar>> {
            (0..count).map(|i| {
                let x = i as Scalar;
                Some(level + 12.0 * ((x + phase) / 8.0).sin() + 6.0 * ((x + phase) / 23.0).cos())
            }).collect()
        };

        match self {
            Data::CarbideExample => vec![
                ("First", [75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].map(Some).to_vec()),
                ("Second", [-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0].map(Some).to_vec()),
            ],
            Data::Visitors => vec![("Mobile", wave(120, 0.0, 40.0)), ("Desktop", wave(120, 10.0, 30.0)), ("Tablet", wave(120, 25.0, 20.0))],
            Data::Gaps => {
                // Two sensors that drop out now and then, for a few readings or for many.
                let with_gaps = |values: Vec<Option<Scalar>>, gaps: &[std::ops::Range<usize>]| -> Vec<Option<Scalar>> {
                    values.into_iter().enumerate().map(|(i, value)| if gaps.iter().any(|gap| gap.contains(&i)) { None } else { value }).collect()
                };
                vec![
                    ("Inside", with_gaps(wave(80, 0.0, 30.0), &[12..14, 40..41, 60..70])),
                    ("Outside", with_gaps(wave(80, 30.0, 15.0), &[25..35, 55..57])),
                ]
            }
            Data::Dense => {
                // 2000 readings with noise, to zoom into.
                let noisy = (0..2000).map(|i| {
                    let x = i as Scalar;
                    let noise = ((x * 12.9898).sin() * 43758.5453).fract() * 6.0;
                    Some(40.0 + 15.0 * (x / 90.0).sin() + 8.0 * (x / 13.0).sin() + noise)
                }).collect();
                vec![("Readings", noisy)]
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    curve: LocalState<Curve>,
    tension: LocalState<Scalar>,
    gaps: LocalState<Gaps>,
    gap_width: LocalState<Scalar>,
    fill: LocalState<Fill>,
    stacked: LocalState<bool>,
    x_zoom: LocalState<ZoomWindow>,
    y_zoom: LocalState<ZoomWindow>,
    fit_visible: LocalState<bool>,
    horizontal: LocalState<bool>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Readings".to_string()),
            data: LocalState::new(Data::Visitors),
            curve: LocalState::new(Curve::Monotone),
            tension: LocalState::new(0.4),
            gaps: LocalState::new(Gaps::Break),
            gap_width: LocalState::new(3.0),
            fill: LocalState::new(Fill::None),
            stacked: LocalState::new(false),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            fit_visible: LocalState::new(true),
            horizontal: LocalState::new(false),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            Data::CarbideExample => chart(o.clone(), s.clone(), Data::CarbideExample),
            Data::Visitors => chart(o.clone(), s.clone(), Data::Visitors),
            Data::Gaps => chart(o.clone(), s.clone(), Data::Gaps),
            Data::Dense => chart(o.clone(), s.clone(), Data::Dense),
        }
    );

    let side = VStack::new((
        first_section("Lines"),
        text_input("Title", &options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("The carbide chart example").tag(Data::CarbideExample),
            Text::new("Visitors").tag(Data::Visitors),
            Text::new("Sensors with gaps").tag(Data::Gaps),
            Text::new("2000 noisy readings").tag(Data::Dense),
        )),
        Picker::new("Curve", options.curve.clone(), (
            Text::new("Straight").tag(Curve::Straight),
            Text::new("Smooth").tag(Curve::Smooth),
            Text::new("Monotone").tag(Curve::Monotone),
            Text::new("Steps before").tag(Curve::StepsBefore),
            Text::new("Steps in the middle").tag(Curve::StepsMiddle),
            Text::new("Steps in the middle, upright").tag(Curve::StepsMiddleVertical),
            Text::new("Steps after").tag(Curve::StepsAfter),
        )),
        // Only smooth curves have a tension, and only the sensors' data has gaps to bridge.
        shown_when(
            Map1::read_map(options.curve.clone(), |curve: &Curve| *curve == Curve::Smooth),
            slider("Tension", &options.tension, 0.0, 1.0, 0.05, |tension| format!("{tension:.2}"))
        ),
        shown_when(
            Map1::read_map(options.data.clone(), |data: &Data| *data == Data::Gaps),
            VStack::new((
                Picker::new("Gaps", options.gaps.clone(), (
                    Text::new("Break the line").tag(Gaps::Break),
                    Text::new("Bridge them").tag(Gaps::Bridge),
                    Text::new("Bridge the narrow ones").tag(Gaps::BridgeWithin),
                )),
                // How wide a gap may be, along x, and still be bridged.
                shown_when(
                    Map1::read_map(options.gaps.clone(), |gaps: &Gaps| *gaps == Gaps::BridgeWithin),
                    slider("Up to", &options.gap_width, 1.0, 12.0, 1.0, |width| format!("{width:.0}"))
                ),
            )).cross_axis_alignment(CrossAxisAlignment::Start)
        ),
        Picker::new("Fill", options.fill.clone(), (
            Text::new("None").tag(Fill::None),
            Text::new("To zero").tag(Fill::Origin),
            Text::new("To 30").tag(Fill::Value(30.0)),
            Text::new("To the bottom").tag(Fill::Bottom),
            Text::new("To the top").tag(Fill::Top),
            Text::new("To the line below").tag(Fill::Stack),
        )),
        Toggle::new("Stacked", options.stacked.clone()),
        Toggle::new("Lying", options.horizontal.clone()),
        VStack::new((
            section("Zoom"),
            zoom_controls("x", &options.x_zoom),
            zoom_controls("y", &options.y_zoom),
            Toggle::new("y fits the part of x shown", options.fit_visible.clone()),
        )).cross_axis_alignment(CrossAxisAlignment::Start),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The lines of `data`.
pub(super) fn chart(options: Options, shared: Shared, data: Data) -> Box<dyn AnyWidget> {
    let interpolation = Map2::read_map(options.curve.clone(), options.tension.clone(), |curve: &Curve, tension: &Scalar| match curve {
        Curve::Straight => Interpolation::Linear,
        Curve::Smooth => Interpolation::Smooth { tension: *tension },
        Curve::Monotone => Interpolation::Monotone,
        Curve::StepsBefore => Interpolation::Stepped(Stepped::Before),
        Curve::StepsMiddle => Interpolation::Stepped(Stepped::Middle),
        Curve::StepsMiddleVertical => Interpolation::Stepped(Stepped::MiddleVertical),
        Curve::StepsAfter => Interpolation::Stepped(Stepped::After),
    });

    let span_gaps = Map2::read_map(options.gaps.clone(), options.gap_width.clone(), |gaps: &Gaps, width: &Scalar| match gaps {
        Gaps::Break => SpanGaps::Never,
        Gaps::Bridge => SpanGaps::Always,
        Gaps::BridgeWithin => SpanGaps::Within(*width),
    });

    let lines: Vec<Box<dyn LineDataSet>> = data.data().into_iter().map(|(name, values)| {
        let stack = Map1::read_map(options.stacked.clone(), move |stacked: &bool| {
            if *stacked {
                StackId::new("all")
            } else {
                StackId::new(name)
            }
        });

        values.label(name)
            .interpolation(interpolation.clone())
            .span_gaps(span_gaps.clone())
            .fill(options.fill.clone())
            .stack(stack)
            .boxed::<dyn LineDataSet>()
    }).collect();

    shared.apply(Chart::new(LineController::new(lines).horizontal(options.horizontal.clone()))
        .title(options.title.clone())
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(options.x_zoom.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(options.y_zoom.clone()).fit_visible(options.fit_visible.clone())))
        .boxed()
}

