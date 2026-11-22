//! A gallery of charts to play with, one at a time, the built-in ones and one built from this crate's
//! own chart elements: the bar along the top picks the chart, the side
//! bar holds its options, and the bar along the bottom reports what the pointer hovers and clicks.
//!
//! On the charts that zoom: drag or scroll to pan, pinch (or hold Ctrl or Cmd and scroll) to zoom
//! about the pointer, Shift-drag to zoom to a box, and double-click to show everything again. Over
//! an axis, the gestures move only that axis.
//!
//! Every option is a state the chart reads, so it changes the chart as it is set. Each chart also
//! offers a choice of data; as the data's types differ, each choice is a chart of its own.
//!
//! Run it with `cargo run -p chart_extensions`.

use carbide::chart::{AngleDirection, Axis, BarController, BarDataSet, DataColor, LineDataSet, PieDataSet, ScatterDataSet, CategoryScale, Chart, DatasetExt, Fill, GridShape, Interpolation, LegendPosition, LinearScale, LineController, Measure, Pick, PickMode, PieController, ScaleId, ScatterController, SizedDataSetExt, StackId, Stepped, ZoomWindow};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::picker::{MenuStyle, Picker};
use carbide::controls::slider::Slider;
use carbide::controls::toggle::{CheckboxStyle, Toggle};
use carbide::controls::{ControlsExt, TextInput};
use carbide::color::{BLUE, GREEN, PURPLE};
use carbide::draw::{Angle, Dimension, Scalar};
use chart_extensions::{Kelvin, LogScale, LollipopController};
use carbide::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide::state::{LocalState, Map1, Map2, State};
use carbide::widget::*;
use carbide::{ui, Application, Window};

/// The charts the gallery shows, one at a time.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Example {
    Bars,
    Lines,
    Bubbles,
    Pie,
    Radar,
    Custom,
}

/// How the bars of the datasets share a month.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Stacking {
    Grouped,
    Stacked,
    Percent,
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const SKILLS: [&str; 6] = ["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"];

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Charts - Carbide",
        Dimension::new(1200.0, 800.0),
        gallery(LocalState::new(Example::Bars)),
    ));

    application.launch();
}

/// The gallery: the bar of charts on top, the chart `selected` shows with its side bar, and the
/// bar reporting what the pointer hovers and clicks at the bottom.
fn gallery(selected: LocalState<Example>) -> impl Widget {
    let shared = Shared::new();

    let (bars, lines, bubbles, pie, radar, custom) = (BarOptions::new(), LineOptions::new(), BubbleOptions::new(), PieOptions::new(), RadarOptions::new(), CustomOptions::new());
    let (bars_shared, lines_shared, bubbles_shared, pie_shared, radar_shared, custom_shared) = (shared.clone(), shared.clone(), shared.clone(), shared.clone(), shared.clone(), shared.clone());

    let gallery = ui!(
        match selected {
            Example::Bars => bars_view(bars.clone(), bars_shared.clone()),
            Example::Lines => lines_view(lines.clone(), lines_shared.clone()),
            Example::Bubbles => bubbles_view(bubbles.clone(), bubbles_shared.clone()),
            Example::Pie => pie_view(pie.clone(), pie_shared.clone()),
            Example::Radar => radar_view(radar.clone(), radar_shared.clone()),
            Example::Custom => custom_view(custom.clone(), custom_shared.clone()),
        }
    );

    let tool_bar = HStack::new((
        example_button("Bars", Example::Bars, &selected, &shared),
        example_button("Lines", Example::Lines, &selected, &shared),
        example_button("Bubbles", Example::Bubbles, &selected, &shared),
        example_button("Pie", Example::Pie, &selected, &shared),
        example_button("Radar", Example::Radar, &selected, &shared),
        example_button("Custom", Example::Custom, &selected, &shared),
        Spacer::new(),
    ))
        .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
        .frame_fixed_height(35.0)
        .background(Rectangle::new().fill(EnvironmentColor::SystemFill))
        .button_style(BorderedProminentStyle);

    let status_bar = HStack::new((
        Text::new(Map1::read_map(shared.hovered.clone(), |pick: &Option<Pick>| match pick {
            Some(pick) => format!("Hovering: {}", describe(pick)),
            None => "Hover the chart".to_string(),
        })),
        Spacer::new(),
        Text::new(shared.clicked.clone()),
    ))
        .padding(EdgeInsets::single(0.0, 0.0, 10.0, 10.0))
        .frame_fixed_height(24.0)
        .background(Rectangle::new().fill(EnvironmentColor::SystemFill));

    VStack::new((tool_bar, gallery, status_bar)).spacing(0.0)
}

/// A button in the top bar that shows `example`, letting go of what the chart before it hovered.
fn example_button(label: &str, example: Example, selected: &LocalState<Example>, shared: &Shared) -> impl Widget + use<> {
    let (selected, hovered) = (selected.clone(), shared.hovered.clone());
    Button::new(label.to_string(), move |_| {
        let (mut selected, mut hovered) = (selected.clone(), hovered.clone());
        *selected.value_mut() = example;
        *hovered.value_mut() = None;
    }).frame(80.0, 26.0)
}

// ===============================
//  Options shared by every chart
// ===============================

/// What every chart reports and is set by: the hovered and clicked data, which the bottom bar
/// shows, and the options for the pointer and the legend.
#[derive(Clone)]
struct Shared {
    hovered: LocalState<Option<Pick>>,
    clicked: LocalState<String>,
    interactive: LocalState<bool>,
    hover_mode: LocalState<PickMode>,
    legend: LocalState<LegendPosition>,
}

impl Shared {
    fn new() -> Shared {
        Shared {
            hovered: LocalState::new(None),
            clicked: LocalState::new("Click the chart".to_string()),
            interactive: LocalState::new(true),
            hover_mode: LocalState::new(PickMode::Index),
            legend: LocalState::new(LegendPosition::Bottom),
        }
    }

    /// `chart` with the shared options: the legend, the pointer's options, and its hover and click
    /// reported to the bottom bar.
    fn apply<C: carbide::chart::ControllerSequence>(&self, chart: Chart<C>) -> Chart<C> {
        let clicked = self.clicked.clone();
        chart
            .legend(self.legend.clone())
            .interactive(self.interactive.clone())
            .hover(self.hovered.clone())
            .hover_mode(self.hover_mode.clone())
            .on_click(move |click| {
                let mut clicked = clicked.clone();
                *clicked.value_mut() = format!("Clicked: {}", describe(click.pick));
            })
    }

    /// The side bar's section for the shared options.
    fn options(&self) -> impl Widget + use<> {
        VStack::new((
            section("Pointer"),
            Toggle::new("Interactive", self.interactive.clone()),
            Picker::new("Hover finds", self.hover_mode.clone(), (
                Text::new("The category").tag(PickMode::Index),
                Text::new("What is under it").tag(PickMode::Point),
                Text::new("The nearest").tag(PickMode::Nearest(Measure::Both)),
                Text::new("The nearest along x").tag(PickMode::Nearest(Measure::X)),
                Text::new("The whole dataset").tag(PickMode::Dataset),
            )),
            section("Legend"),
            Picker::new("Legend", self.legend.clone(), (
                Text::new("Hidden").tag(LegendPosition::Hidden),
                Text::new("Top").tag(LegendPosition::Top),
                Text::new("Bottom").tag(LegendPosition::Bottom),
                Text::new("Left").tag(LegendPosition::Left),
                Text::new("Right").tag(LegendPosition::Right),
            )),
        )).cross_axis_alignment(CrossAxisAlignment::Start)
    }
}

/// A chart beside its side bar, the shared options at the bottom of it.
fn with_side_bar(chart: impl Widget, options: impl Widget, shared: &Shared) -> Box<dyn AnyWidget> {
    HSplit::new(
        chart.padding(10.0),
        ZStack::new((
            Rectangle::new().fill(EnvironmentColor::TertiarySystemFill),
            VStack::new((
                options,
                shared.options(),
                Spacer::new(),
            ))
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .padding(10.0)
                .toggle_style(CheckboxStyle)
                .picker_style(MenuStyle),
        )),
    ).relative_to_end(300.0).boxed()
}

/// A heading in the side bar.
fn section(title: &str) -> impl Widget + use<> {
    Text::new(title.to_string()).font_size(EnvironmentFontSize::Title3)
}

/// A title for the chart, edited in the side bar.
fn title_input(title: &LocalState<String>) -> impl Widget + use<> {
    HStack::new((Text::new("Title"), TextInput::new(title.clone())))
}

/// A zoom window's controls: what it shows, and a button that shows everything again.
fn zoom_controls(label: &str, zoom: &LocalState<ZoomWindow>) -> impl Widget + use<> {
    let reset = zoom.clone();
    let label = label.to_string();
    VStack::new((
        Text::new(Map1::read_map(zoom.clone(), move |window: &ZoomWindow| format!("{} shows {:.0}% to {:.0}%", label, window.start * 100.0, window.end * 100.0))),
        Button::new("Show everything", move |_| {
            let mut reset = reset.clone();
            *reset.value_mut() = ZoomWindow::WHOLE;
        }).frame(140.0, 26.0),
    )).cross_axis_alignment(CrossAxisAlignment::Start)
}

/// A slider for a number, with its value beside it.
fn slider(label: &str, value: &LocalState<Scalar>, min: Scalar, max: Scalar, step: Scalar, format: fn(&Scalar) -> String) -> impl Widget + use<> {
    HStack::new((
        Text::new(label.to_string()),
        Slider::new(value.clone(), min, max).step(step),
        Text::new(Map1::read_map(value.clone(), format)).frame_fixed_width(50.0),
    )).frame_fixed_height(26.0)
}

// ===============================
//  Bars
// ===============================

/// The data the bar chart can show.
#[derive(Copy, Clone, Debug, PartialEq)]
enum BarData {
    Regions,
    Profit,
    Quarters,
}

impl BarData {
    /// The categories, and each dataset's name and values.
    fn data(self) -> (Vec<&'static str>, Vec<(&'static str, Vec<Scalar>)>) {
        match self {
            BarData::Regions => (MONTHS.to_vec(), vec![
                ("North", vec![12.0, 19.0, 3.0, 5.0, 2.0, 8.0, 14.0, 9.0, 11.0, 6.0, 15.0, 18.0]),
                ("South", vec![8.0, 11.0, 6.0, 4.0, 3.0, 7.0, 9.0, 5.0, 10.0, 8.0, 12.0, 9.0]),
                ("West", vec![5.0, 7.0, 9.0, 12.0, 6.0, 4.0, 8.0, 11.0, 7.0, 9.0, 5.0, 10.0]),
            ]),
            BarData::Profit => (MONTHS.to_vec(), vec![
                ("Income", vec![30.0, 28.0, 35.0, 40.0, 38.0, 45.0, 50.0, 48.0, 42.0, 39.0, 44.0, 60.0]),
                ("Costs", vec![-25.0, -30.0, -28.0, -32.0, -41.0, -36.0, -38.0, -40.0, -45.0, -33.0, -35.0, -42.0]),
            ]),
            BarData::Quarters => (vec!["Q1", "Q2", "Q3", "Q4"], vec![
                ("2025", vec![120.0, 135.0, 128.0, 160.0]),
                ("2026", vec![140.0, 150.0, 149.0, 170.0]),
            ]),
        }
    }
}

#[derive(Clone)]
struct BarOptions {
    title: LocalState<String>,
    data: LocalState<BarData>,
    stacking: LocalState<Stacking>,
    zoom: LocalState<ZoomWindow>,
    fit_visible: LocalState<bool>,
    grid_lines: LocalState<bool>,
    horizontal: LocalState<bool>,
    category_percentage: LocalState<Scalar>,
    bar_percentage: LocalState<Scalar>,
}

impl BarOptions {
    fn new() -> BarOptions {
        BarOptions {
            title: LocalState::new("Sales".to_string()),
            data: LocalState::new(BarData::Regions),
            stacking: LocalState::new(Stacking::Grouped),
            zoom: LocalState::new(ZoomWindow::WHOLE),
            fit_visible: LocalState::new(false),
            grid_lines: LocalState::new(true),
            horizontal: LocalState::new(false),
            category_percentage: LocalState::new(0.8),
            bar_percentage: LocalState::new(0.9),
        }
    }
}

fn bars_view(options: BarOptions, shared: Shared) -> Box<dyn AnyWidget> {
    // The data and a scale's axis are fixed when the chart is built, so choosing other data, or
    // lying the bars down, which swaps the category and value scales, shows a chart built for it.
    let shown = Map2::read_map(options.data.clone(), options.horizontal.clone(), |data: &BarData, lying: &bool| (*data, *lying));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (BarData::Regions, false) => bars_chart(o.clone(), s.clone(), BarData::Regions, false),
            (BarData::Regions, true) => bars_chart(o.clone(), s.clone(), BarData::Regions, true),
            (BarData::Profit, false) => bars_chart(o.clone(), s.clone(), BarData::Profit, false),
            (BarData::Profit, true) => bars_chart(o.clone(), s.clone(), BarData::Profit, true),
            (BarData::Quarters, false) => bars_chart(o.clone(), s.clone(), BarData::Quarters, false),
            (BarData::Quarters, true) => bars_chart(o.clone(), s.clone(), BarData::Quarters, true),
        }
    );

    let side = VStack::new((
        section("Bars"),
        title_input(&options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Sales by region").tag(BarData::Regions),
            Text::new("Income and costs").tag(BarData::Profit),
            Text::new("Quarters").tag(BarData::Quarters),
        )),
        Picker::new("Datasets", options.stacking.clone(), (
            Text::new("Side by side").tag(Stacking::Grouped),
            Text::new("Stacked").tag(Stacking::Stacked),
            Text::new("Stacked to 100%").tag(Stacking::Percent),
        )),
        Toggle::new("Lying", options.horizontal.clone()),
        slider("Group", &options.category_percentage, 0.1, 1.0, 0.05, |share| format!("{:.0}%", share * 100.0)),
        slider("Bar", &options.bar_percentage, 0.1, 1.0, 0.05, |share| format!("{:.0}%", share * 100.0)),
        Toggle::new("Grid lines", options.grid_lines.clone()),
        section("Zoom"),
        zoom_controls("Categories", &options.zoom),
        Toggle::new("Values fit the categories shown", options.fit_visible.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The bars of `data`, standing or lying: the categories along x or down y.
fn bars_chart(options: BarOptions, shared: Shared, data: BarData, lying: bool) -> Box<dyn AnyWidget> {
    let (categories, series) = data.data();

    // Grouped, each dataset is a stack of its own; stacked, they share one, normalised for 100%.
    let datasets: Vec<Box<dyn BarDataSet>> = series.into_iter().map(|(name, values)| {
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

    let (category_id, value_id, category_axis, value_axis) = if lying {
        (ScaleId::Y, ScaleId::X, Axis::Vertical, Axis::Horizontal)
    } else {
        (ScaleId::X, ScaleId::Y, Axis::Horizontal, Axis::Vertical)
    };

    shared.apply(Chart::new(bars)
        .title(options.title.clone())
        .scale(category_id, CategoryScale::new(category_axis).labels(categories).zoom(options.zoom.clone()))
        .scale(value_id, LinearScale::new(value_axis).fit_visible(options.fit_visible.clone()).grid_lines(options.grid_lines.clone())))
        .boxed()
}

// ===============================
//  Lines
// ===============================

/// The data the line chart can show.
#[derive(Copy, Clone, Debug, PartialEq)]
enum LineData {
    Visitors,
    Gaps,
    Dense,
}

impl LineData {
    /// Each dataset's name and values, at their indices; `None` is a missing value.
    fn data(self) -> Vec<(&'static str, Vec<Option<Scalar>>)> {
        let wave = |count: usize, phase: Scalar, level: Scalar| -> Vec<Option<Scalar>> {
            (0..count).map(|i| {
                let x = i as Scalar;
                Some(level + 12.0 * ((x + phase) / 8.0).sin() + 6.0 * ((x + phase) / 23.0).cos())
            }).collect()
        };

        match self {
            LineData::Visitors => vec![("Mobile", wave(120, 0.0, 40.0)), ("Desktop", wave(120, 10.0, 30.0)), ("Tablet", wave(120, 25.0, 20.0))],
            LineData::Gaps => {
                // Two sensors that drop out now and then.
                let with_gaps = |values: Vec<Option<Scalar>>, gaps: &[std::ops::Range<usize>]| -> Vec<Option<Scalar>> {
                    values.into_iter().enumerate().map(|(i, value)| if gaps.iter().any(|gap| gap.contains(&i)) { None } else { value }).collect()
                };
                vec![
                    ("Inside", with_gaps(wave(80, 0.0, 30.0), &[12..18, 40..41, 60..70])),
                    ("Outside", with_gaps(wave(80, 30.0, 15.0), &[25..35, 55..57])),
                ]
            }
            LineData::Dense => {
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
struct LineOptions {
    title: LocalState<String>,
    data: LocalState<LineData>,
    interpolation: LocalState<Interpolation>,
    fill: LocalState<Fill>,
    stacked: LocalState<bool>,
    x_zoom: LocalState<ZoomWindow>,
    y_zoom: LocalState<ZoomWindow>,
    fit_visible: LocalState<bool>,
    horizontal: LocalState<bool>,
}

impl LineOptions {
    fn new() -> LineOptions {
        LineOptions {
            title: LocalState::new("Readings".to_string()),
            data: LocalState::new(LineData::Visitors),
            interpolation: LocalState::new(Interpolation::Monotone),
            fill: LocalState::new(Fill::None),
            stacked: LocalState::new(false),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            fit_visible: LocalState::new(true),
            horizontal: LocalState::new(false),
        }
    }
}

fn lines_view(options: LineOptions, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            LineData::Visitors => lines_chart(o.clone(), s.clone(), LineData::Visitors),
            LineData::Gaps => lines_chart(o.clone(), s.clone(), LineData::Gaps),
            LineData::Dense => lines_chart(o.clone(), s.clone(), LineData::Dense),
        }
    );

    let side = VStack::new((
        section("Lines"),
        title_input(&options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Visitors").tag(LineData::Visitors),
            Text::new("Sensors with gaps").tag(LineData::Gaps),
            Text::new("2000 noisy readings").tag(LineData::Dense),
        )),
        Picker::new("Curve", options.interpolation.clone(), (
            Text::new("Straight").tag(Interpolation::Linear),
            Text::new("Smooth").tag(Interpolation::Smooth { tension: 0.4 }),
            Text::new("Monotone").tag(Interpolation::Monotone),
            Text::new("Steps before").tag(Interpolation::Stepped(Stepped::Before)),
            Text::new("Steps in the middle").tag(Interpolation::Stepped(Stepped::Middle)),
            Text::new("Steps after").tag(Interpolation::Stepped(Stepped::After)),
        )),
        Picker::new("Fill", options.fill.clone(), (
            Text::new("None").tag(Fill::None),
            Text::new("To zero").tag(Fill::Origin),
            Text::new("To the bottom").tag(Fill::Bottom),
            Text::new("To the line below").tag(Fill::Stack),
        )),
        Toggle::new("Stacked", options.stacked.clone()),
        Toggle::new("Lying", options.horizontal.clone()),
        section("Zoom"),
        zoom_controls("x", &options.x_zoom),
        zoom_controls("y", &options.y_zoom),
        Toggle::new("y fits the part of x shown", options.fit_visible.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The lines of `data`.
fn lines_chart(options: LineOptions, shared: Shared, data: LineData) -> Box<dyn AnyWidget> {
    let lines: Vec<Box<dyn LineDataSet>> = data.data().into_iter().map(|(name, values)| {
        let stack = Map1::read_map(options.stacked.clone(), move |stacked: &bool| if *stacked { StackId::new("all") } else { StackId::new(name) });
        values.label(name)
            .interpolation(options.interpolation.clone())
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

// ===============================
//  Bubbles
// ===============================

/// The data the bubble chart can show.
#[derive(Copy, Clone, Debug, PartialEq)]
enum BubbleData {
    Stores,
    Clusters,
}

impl BubbleData {
    /// Each dataset's name and bubbles: x, y and size.
    fn data(self) -> Vec<(&'static str, Vec<(Scalar, Scalar, Scalar)>)> {
        match self {
            BubbleData::Stores => vec![
                ("North", (0..25).map(|i| {
                    let i = i as Scalar;
                    ((i * 7.3) % 20.0, (i * 3.7) % 15.0, 5.0 + (i * 11.0) % 40.0)
                }).collect()),
                ("South", (0..15).map(|i| {
                    let i = i as Scalar;
                    ((i * 5.1 + 2.0) % 20.0, (i * 6.3 + 4.0) % 15.0, 10.0 + (i * 13.0) % 30.0)
                }).collect()),
            ],
            BubbleData::Clusters => {
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
struct BubbleOptions {
    title: LocalState<String>,
    data: LocalState<BubbleData>,
    x_zoom: LocalState<ZoomWindow>,
    y_zoom: LocalState<ZoomWindow>,
    grid_lines: LocalState<bool>,
    show_line: LocalState<bool>,
    min_radius: LocalState<Scalar>,
    max_radius: LocalState<Scalar>,
}

impl BubbleOptions {
    fn new() -> BubbleOptions {
        BubbleOptions {
            title: LocalState::new("Bubbles".to_string()),
            data: LocalState::new(BubbleData::Stores),
            x_zoom: LocalState::new(ZoomWindow::WHOLE),
            y_zoom: LocalState::new(ZoomWindow::WHOLE),
            grid_lines: LocalState::new(true),
            show_line: LocalState::new(false),
            min_radius: LocalState::new(4.0),
            max_radius: LocalState::new(30.0),
        }
    }
}

fn bubbles_view(options: BubbleOptions, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            BubbleData::Stores => bubbles_chart(o.clone(), s.clone(), BubbleData::Stores),
            BubbleData::Clusters => bubbles_chart(o.clone(), s.clone(), BubbleData::Clusters),
        }
    );

    let side = VStack::new((
        section("Bubbles"),
        title_input(&options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Stores").tag(BubbleData::Stores),
            Text::new("Three clusters").tag(BubbleData::Clusters),
        )),
        Toggle::new("Line through them", options.show_line.clone()),
        slider("Smallest", &options.min_radius, 1.0, 20.0, 1.0, |radius| format!("{radius:.0} px")),
        slider("Largest", &options.max_radius, 10.0, 60.0, 1.0, |radius| format!("{radius:.0} px")),
        Toggle::new("Grid lines", options.grid_lines.clone()),
        section("Zoom"),
        zoom_controls("x", &options.x_zoom),
        zoom_controls("y", &options.y_zoom),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The bubbles of `data`.
fn bubbles_chart(options: BubbleOptions, shared: Shared, data: BubbleData) -> Box<dyn AnyWidget> {
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

// ===============================
//  Pie
// ===============================

/// The data the pie can show.
#[derive(Copy, Clone, Debug, PartialEq)]
enum PieData {
    Spending,
    Budget,
    Many,
}

impl PieData {
    /// Each ring's name and parts.
    fn data(self) -> Vec<(&'static str, Vec<(&'static str, Scalar)>)> {
        match self {
            PieData::Spending => vec![
                ("2025", vec![("Rent", 45.0), ("Food", 25.0), ("Travel", 15.0), ("Other", 15.0)]),
                ("2026", vec![("Rent", 40.0), ("Food", 30.0), ("Travel", 10.0), ("Other", 20.0)]),
            ],
            PieData::Budget => vec![
                ("Budget", vec![("Housing", 32.0), ("Transport", 15.0), ("Food", 13.0), ("Health", 9.0), ("Savings", 20.0), ("Fun", 11.0)]),
            ],
            PieData::Many => vec![
                ("Languages", vec![
                    ("Rust", 18.0), ("Go", 12.0), ("Python", 22.0), ("Java", 9.0), ("C", 6.0), ("C++", 7.0), ("Swift", 4.0), ("Kotlin", 3.0),
                    ("Ruby", 2.0), ("Zig", 1.5), ("Haskell", 1.0), ("OCaml", 0.8), ("Elixir", 1.2), ("Lua", 0.9), ("Odin", 0.6),
                ]),
            ],
        }
    }
}

#[derive(Clone)]
struct PieOptions {
    title: LocalState<String>,
    data: LocalState<PieData>,
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

impl PieOptions {
    fn new() -> PieOptions {
        PieOptions {
            title: LocalState::new("Shares".to_string()),
            data: LocalState::new(PieData::Spending),
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

fn pie_view(options: PieOptions, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            PieData::Spending => pie_chart(o.clone(), s.clone(), PieData::Spending),
            PieData::Budget => pie_chart(o.clone(), s.clone(), PieData::Budget),
            PieData::Many => pie_chart(o.clone(), s.clone(), PieData::Many),
        }
    );

    let side = VStack::new((
        section("Pie"),
        title_input(&options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Spending, two years").tag(PieData::Spending),
            Text::new("A budget").tag(PieData::Budget),
            Text::new("Many small parts").tag(PieData::Many),
        )),
        slider("Hole", &options.inner_radius, 0.0, 0.9, 0.05, |ratio| format!("{:.0}%", ratio * 100.0)),
        slider("Start", &options.start_angle, 0.0, 360.0, 5.0, |degrees| format!("{degrees:.0}°")),
        slider("Sweep", &options.sweep, 30.0, 360.0, 5.0, |degrees| format!("{degrees:.0}°")),
        Picker::new("Direction", options.direction.clone(), (
            Text::new("Clockwise").tag(AngleDirection::Clockwise),
            Text::new("Counter-clockwise").tag(AngleDirection::CounterClockwise),
        )),
        slider("Gaps", &options.spacing, 0.0, 20.0, 1.0, |pixels| format!("{pixels:.0} px")),
        slider("Ring gaps", &options.ring_spacing, 0.0, 30.0, 1.0, |pixels| format!("{pixels:.0} px")),
        slider("Ring size", &options.ring_size, 0.2, 1.0, 0.05, |share| format!("{:.0}%", share * 100.0)),
        slider("Outline", &options.outline_width, 0.0, 8.0, 0.5, |pixels| format!("{pixels:.1} px")),
        Toggle::new("Labels", options.labels.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The pie of `data`.
fn pie_chart(options: PieOptions, shared: Shared, data: PieData) -> Box<dyn AnyWidget> {
    let rings: Vec<Box<dyn PieDataSet>> = data.data().into_iter()
        .map(|(name, parts)| parts.label(name).boxed::<dyn PieDataSet>())
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

// ===============================
//  Radar
// ===============================

/// The data the radar can show.
#[derive(Copy, Clone, Debug, PartialEq)]
enum RadarData {
    Players,
    Squad,
}

impl RadarData {
    /// Each player's name and skills, in the order of `SKILLS`.
    fn data(self) -> Vec<(&'static str, Vec<Scalar>)> {
        match self {
            RadarData::Players => vec![
                ("Ada", vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]),
                ("Bo", vec![5.0, 8.0, 6.0, 4.0, 7.0, 9.0]),
            ],
            RadarData::Squad => vec![
                ("Ada", vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]),
                ("Bo", vec![5.0, 8.0, 6.0, 4.0, 7.0, 9.0]),
                ("Cy", vec![9.0, 4.0, 8.0, 6.0, 8.0, 3.0]),
                ("Dee", vec![4.0, 9.0, 5.0, 7.0, 3.0, 8.0]),
            ],
        }
    }
}

#[derive(Clone)]
struct RadarOptions {
    title: LocalState<String>,
    data: LocalState<RadarData>,
    grid_shape: LocalState<GridShape>,
    fill: LocalState<Fill>,
    interpolation: LocalState<Interpolation>,
    start_angle: LocalState<Scalar>,
    horizontal: LocalState<bool>,
}

impl RadarOptions {
    fn new() -> RadarOptions {
        RadarOptions {
            title: LocalState::new("Players".to_string()),
            data: LocalState::new(RadarData::Players),
            grid_shape: LocalState::new(GridShape::Polygon),
            fill: LocalState::new(Fill::Origin),
            interpolation: LocalState::new(Interpolation::Linear),
            start_angle: LocalState::new(0.0),
            horizontal: LocalState::new(false),
        }
    }
}

fn radar_view(options: RadarOptions, shared: Shared) -> Box<dyn AnyWidget> {
    let data = options.data.clone();
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match data {
            RadarData::Players => radar_chart(o.clone(), s.clone(), RadarData::Players),
            RadarData::Squad => radar_chart(o.clone(), s.clone(), RadarData::Squad),
        }
    );

    let side = VStack::new((
        section("Radar"),
        title_input(&options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Two players").tag(RadarData::Players),
            Text::new("A squad of four").tag(RadarData::Squad),
        )),
        Picker::new("Rings", options.grid_shape.clone(), (
            Text::new("Polygons").tag(GridShape::Polygon),
            Text::new("Circles").tag(GridShape::Circle),
        )),
        Picker::new("Fill", options.fill.clone(), (
            Text::new("None").tag(Fill::None),
            Text::new("To the centre").tag(Fill::Origin),
            Text::new("To the rim").tag(Fill::Top),
        )),
        Picker::new("Curve", options.interpolation.clone(), (
            Text::new("Straight").tag(Interpolation::Linear),
            Text::new("Smooth").tag(Interpolation::Smooth { tension: 0.4 }),
            Text::new("Steps").tag(Interpolation::Stepped(Stepped::Middle)),
        )),
        slider("Start", &options.start_angle, 0.0, 360.0, 5.0, |degrees| format!("{degrees:.0}°")),
        Toggle::new("Skills out from the centre", options.horizontal.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The radar of `data`.
fn radar_chart(options: RadarOptions, shared: Shared, data: RadarData) -> Box<dyn AnyWidget> {
    let players: Vec<Box<dyn LineDataSet>> = data.data().into_iter()
        .map(|(name, skills)| skills.label(name).fill(options.fill.clone()).interpolation(options.interpolation.clone()).boxed::<dyn LineDataSet>())
        .collect();

    shared.apply(Chart::new(LineController::new(players).horizontal(options.horizontal.clone()))
        .title(options.title.clone())
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS)
            .start_angle(Map1::read_map(options.start_angle.clone(), |degrees: &Scalar| Angle::Degrees(*degrees)))
            .end_angle(Map1::read_map(options.start_angle.clone(), |degrees: &Scalar| Angle::Degrees(*degrees + 360.0))))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).grid_shape(options.grid_shape.clone())))
        .boxed()
}

// ===============================
//  Custom
// ===============================

/// The data the custom chart can show, drawn with this crate's own chart elements.
#[derive(Copy, Clone, Debug, PartialEq)]
enum CustomData {
    /// Readings in kelvin (`Kelvin`) shown in degrees Celsius, with the forecast as lollipops.
    Temperature,
    /// Populations spanning five orders of magnitude, as lollipops.
    Settlements,
}

/// The lollipops' colour.
#[derive(Copy, Clone, Debug, PartialEq)]
enum LollipopColor {
    Fallback,
    Blue,
    Green,
    Purple,
}

impl LollipopColor {
    fn data_color(&self) -> DataColor {
        match self {
            LollipopColor::Fallback => DataColor::ByDataSet,
            LollipopColor::Blue => DataColor::Color(BLUE),
            LollipopColor::Green => DataColor::Color(GREEN),
            LollipopColor::Purple => DataColor::Color(PURPLE),
        }
    }
}

#[derive(Clone)]
struct CustomOptions {
    title: LocalState<String>,
    data: LocalState<CustomData>,
    log_scale: LocalState<bool>,
    color: LocalState<LollipopColor>,
}

impl CustomOptions {
    fn new() -> CustomOptions {
        CustomOptions {
            title: LocalState::new("Extended from outside".to_string()),
            data: LocalState::new(CustomData::Temperature),
            log_scale: LocalState::new(true),
            color: LocalState::new(LollipopColor::Fallback),
        }
    }
}

fn custom_view(options: CustomOptions, shared: Shared) -> Box<dyn AnyWidget> {
    // A scale's type is fixed when the chart is built, so the logarithmic scale and the linear one
    // are charts of their own.
    let shown = Map2::read_map(options.data.clone(), options.log_scale.clone(), |data: &CustomData, log: &bool| (*data, *log));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (CustomData::Temperature, _) => custom_chart(o.clone(), s.clone(), CustomData::Temperature, false),
            (CustomData::Settlements, false) => custom_chart(o.clone(), s.clone(), CustomData::Settlements, false),
            (CustomData::Settlements, true) => custom_chart(o.clone(), s.clone(), CustomData::Settlements, true),
        }
    );

    let side = VStack::new((
        section("Custom"),
        Text::new("Drawn with this crate's own\nLollipopController, LogScale and\nKelvin values, through the public API."),
        title_input(&options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Temperature").tag(CustomData::Temperature),
            Text::new("Settlements").tag(CustomData::Settlements),
        )),
        Picker::new("Lollipops", options.color.clone(), (
            Text::new("Fallback orange").tag(LollipopColor::Fallback),
            Text::new("Blue").tag(LollipopColor::Blue),
            Text::new("Green").tag(LollipopColor::Green),
            Text::new("Purple").tag(LollipopColor::Purple),
        )),
        Toggle::new("Settlements on the LogScale", options.log_scale.clone()),
        Text::new("Hover finds no lollipops: a\ncontroller outside the crate can't\nadd its shapes yet."),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The custom chart of `data`, on the crate's `LogScale` when `log` is set.
fn custom_chart(options: CustomOptions, shared: Shared, data: CustomData, log: bool) -> Box<dyn AnyWidget> {
    let color = Map1::read_map(options.color.clone(), |color: &LollipopColor| color.data_color());

    match data {
        CustomData::Temperature => {
            // A sensor that broke down on Thursday: `Kelvin` takes its reading of 0 K as missing,
            // so the line has a gap there.
            let readings = vec![(0.0, Kelvin(285.2)), (1.0, Kelvin(287.9)), (2.0, Kelvin(290.1)), (3.0, Kelvin(0.0)), (4.0, Kelvin(288.4)), (5.0, Kelvin(284.6)), (6.0, Kelvin(283.0))];
            let forecast = vec![(0.0, 11.0), (1.0, 14.5), (2.0, 16.0), (3.0, 17.5), (4.0, 15.0), (5.0, 12.0), (6.0, 10.5)];

            shared.apply(Chart::new((
                LineController::new(readings.label("Measured (°C)")),
                LollipopController::new(forecast.label("Forecast (°C)").color(color)),
            ))
                .title(options.title.clone()))
                .boxed()
        }
        CustomData::Settlements => {
            let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];
            let chart = Chart::new(LollipopController::new(populations.label("Population").color(color)))
                .title(options.title.clone());

            if log {
                shared.apply(chart.scale(ScaleId::Y, LogScale::new(Axis::Vertical).title("Inhabitants (log)"))).boxed()
            } else {
                shared.apply(chart.scale(ScaleId::Y, LinearScale::new(Axis::Vertical).title("Inhabitants"))).boxed()
            }
        }
    }
}

/// What a pick found, for the bottom bar: each datum by its dataset and index, and the value under
/// the pointer on each scale, or its category.
fn describe(pick: &Pick) -> String {
    let data = if pick.data.is_empty() {
        "nothing".to_string()
    } else {
        pick.data.iter().map(|datum| format!("dataset {} #{}", datum.dataset, datum.index)).collect::<Vec<_>>().join(", ")
    };

    let values = pick.values.iter()
        .map(|(id, value)| match &value.category {
            Some(category) => format!("{} {}", id.name(), category.name.clone().unwrap_or_else(|| format!("{}", category.index))),
            None => format!("{} {:.1}", id.name(), value.value),
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("{data} at {values}")
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position};
    use carbide::state::{LocalState, State};
    use carbide::testing::Harness;
    use super::{gallery, Example};

    /// Every chart in the gallery renders, and can be hovered. A smoke test: it renders each chart
    /// and moves the pointer over it, without comparing pixels.
    #[test]
    fn every_chart_renders_and_hovers() {
        let mut selected = LocalState::new(Example::Bars);
        let mut harness = Harness::new(gallery(selected.clone()), Dimension::new(1200.0, 800.0));

        for (example, at) in [
            (Example::Bars, Position::new(450.0, 300.0)),
            (Example::Lines, Position::new(300.0, 350.0)),
            (Example::Bubbles, Position::new(520.0, 450.0)),
            (Example::Pie, Position::new(600.0, 300.0)),
            (Example::Radar, Position::new(560.0, 300.0)),
            (Example::Custom, Position::new(450.0, 300.0)),
        ] {
            *selected.value_mut() = example;
            harness.render();
            harness.mouse_move(at);
            harness.render();
        }
    }


}
