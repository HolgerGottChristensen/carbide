//! Bubbles: sized points, with or without a line through them, zooming along both scales.

use carbide::chart::{Axis, Chart, DatasetExt, LinearScale, ScaleId, ScatterController, ScatterDataSet, SizedDataSetExt, ZoomWindow};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::ui;
use super::{pixels, section, slider, text_input, with_side_bar, zoom_controls, Shared};

/// The data the bubble chart can show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    Stores,
    Clusters,
}

impl Data {
    /// Each dataset's name and bubbles: x, y and size.
    fn data(self) -> Vec<(&'static str, Vec<(Scalar, Scalar, Scalar)>)> {
        match self {
            Data::Stores => vec![
                ("North", (0..25).map(|i| {
                    let i = i as Scalar;
                    ((i * 7.3) % 20.0, (i * 3.7) % 15.0, 5.0 + (i * 11.0) % 40.0)
                }).collect()),
                ("South", (0..15).map(|i| {
                    let i = i as Scalar;
                    ((i * 5.1 + 2.0) % 20.0, (i * 6.3 + 4.0) % 15.0, 10.0 + (i * 13.0) % 30.0)
                }).collect()),
            ],
            Data::Clusters => {
                // Three groups scattered about their centres, with spread-out sizes.
                let cluster = |cx: Scalar, cy: Scalar, seed: Scalar| -> Vec<(Scalar, Scalar, Scalar)> {
                    (0..20).map(|i| {
                        let i = i as Scalar + seed;
                        let jitter = |k: Scalar| ((i * k).sin() * 43758.5453).fract() - 0.5;
                        (cx + jitter(12.9898) * 6.0, cy + jitter(78.233) * 6.0, 2.0 + (jitter(39.425) + 0.5) * 50.0)
                    }).collect()
                };
                vec![("Alpha", cluster(10.0, 10.0, 0.0)), ("Beta", cluster(25.0, 20.0, 100.0)), ("Gamma", cluster(20.0, 5.0, 200.0))]
            }
        }
    }
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
            title: LocalState::new("Bubbles".to_string()),
            data: LocalState::new(Data::Stores),
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
            Data::Stores => chart(o.clone(), s.clone(), Data::Stores),
            Data::Clusters => chart(o.clone(), s.clone(), Data::Clusters),
        }
    );

    let side = VStack::new((
        section("Bubbles"),
        text_input("Title", &options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Stores").tag(Data::Stores),
            Text::new("Three clusters").tag(Data::Clusters),
        )),
        Toggle::new("Line through them", options.show_line.clone()),
        slider("Smallest", &options.min_radius, 1.0, 20.0, 1.0, pixels),
        slider("Largest", &options.max_radius, 10.0, 60.0, 1.0, pixels),
        Toggle::new("Grid lines", options.grid_lines.clone()),
        section("Zoom"),
        zoom_controls("x", &options.x_zoom),
        zoom_controls("y", &options.y_zoom),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The bubbles of `data`.
pub(super) fn chart(options: Options, shared: Shared, data: Data) -> Box<dyn AnyWidget> {
    let datasets: Vec<Box<dyn ScatterDataSet>> = data.data().into_iter()
        .map(|(name, bubbles)| bubbles.bubbles().label(name).boxed::<dyn ScatterDataSet>())
        .collect();

    let bubbles = ScatterController::new(datasets)
        .show_line(options.show_line.clone())
        .radius_range(options.min_radius.clone(), options.max_radius.clone());

    shared.apply(Chart::new(bubbles)
        .title(options.title.clone())
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(options.x_zoom.clone()).grid_lines(options.grid_lines.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(options.y_zoom.clone()).grid_lines(options.grid_lines.clone())))
        .boxed()
}
