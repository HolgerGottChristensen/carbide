//! Bars: grouped, stacked or stacked to 100%, standing or lying, with categories from a scale's
//! labels or named by the data, coloured by dataset or by bar.

use carbide::chart::{Axis, BarController, BarDataSet, CategoryScale, Chart, DataColor, DatasetExt, LinearScale, ScaleId, StackId, ZoomWindow};
use carbide::controls::picker::Picker;
use carbide::controls::toggle::Toggle;
use carbide::controls::ControlsExt;
use carbide::draw::Scalar;
use carbide::state::{LocalState, Map1, Map2};
use carbide::widget::*;
use carbide::ui;
use super::{first_section, percent, section, slider, text_input, with_side_bar, zoom_controls, Shared, MONTHS};

/// The data the bar chart can show.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Data {
    Regions,
    Profit,
    Quarters,
    /// Categories named by the data itself, in the order it first names them.
    Fruit,
}

/// How the bars of the datasets share a category.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Stacking {
    Grouped,
    Stacked,
    Percent,
}

/// What the bars' colours follow.
#[derive(Copy, Clone, Debug, PartialEq)]
enum ColorBy {
    Dataset,
    Bar,
}

/// A dataset's values: at the scale's categories in order, or each naming its own category.
enum Series {
    Values(Vec<Scalar>),
    Named(Vec<(&'static str, Scalar)>),
}

impl Data {
    /// The scale's categories, and each dataset's name and values.
    fn data(self) -> (Vec<&'static str>, Vec<(&'static str, Series)>) {
        match self {
            Data::Regions => (MONTHS.to_vec(), vec![
                ("North", Series::Values(vec![12.0, 19.0, 3.0, 5.0, 2.0, 8.0, 14.0, 9.0, 11.0, 6.0, 15.0, 18.0])),
                ("South", Series::Values(vec![8.0, 11.0, 6.0, 4.0, 3.0, 7.0, 9.0, 5.0, 10.0, 8.0, 12.0, 9.0])),
                ("West", Series::Values(vec![5.0, 7.0, 9.0, 12.0, 6.0, 4.0, 8.0, 11.0, 7.0, 9.0, 5.0, 10.0])),
            ]),
            Data::Profit => (MONTHS.to_vec(), vec![
                ("Income", Series::Values(vec![30.0, 28.0, 35.0, 40.0, 38.0, 45.0, 50.0, 48.0, 42.0, 39.0, 44.0, 60.0])),
                ("Costs", Series::Values(vec![-25.0, -30.0, -28.0, -32.0, -41.0, -36.0, -38.0, -40.0, -45.0, -33.0, -35.0, -42.0])),
            ]),
            Data::Quarters => (vec!["Q1", "Q2", "Q3", "Q4"], vec![
                ("2025", Series::Values(vec![120.0, 135.0, 128.0, 160.0])),
                ("2026", Series::Values(vec![140.0, 150.0, 149.0, 170.0])),
            ]),
            // No labels on the scale: the market names apples, pears and figs, then the shop adds
            // plums and cherries after them.
            Data::Fruit => (vec![], vec![
                ("Market", Series::Named(vec![("Apples", 12.0), ("Pears", 7.0), ("Figs", 4.0)])),
                ("Shop", Series::Named(vec![("Pears", 5.0), ("Plums", 9.0), ("Apples", 8.0), ("Cherries", 6.0)])),
            ]),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Options {
    title: LocalState<String>,
    data: LocalState<Data>,
    stacking: LocalState<Stacking>,
    color_by: LocalState<ColorBy>,
    zoom: LocalState<ZoomWindow>,
    fit_visible: LocalState<bool>,
    grid_lines: LocalState<bool>,
    horizontal: LocalState<bool>,
    category_percentage: LocalState<Scalar>,
    bar_percentage: LocalState<Scalar>,
}

impl Options {
    pub(crate) fn new() -> Options {
        Options {
            title: LocalState::new("Sales".to_string()),
            data: LocalState::new(Data::Regions),
            stacking: LocalState::new(Stacking::Grouped),
            color_by: LocalState::new(ColorBy::Dataset),
            zoom: LocalState::new(ZoomWindow::WHOLE),
            fit_visible: LocalState::new(false),
            grid_lines: LocalState::new(true),
            horizontal: LocalState::new(false),
            category_percentage: LocalState::new(0.8),
            bar_percentage: LocalState::new(0.9),
        }
    }
}

pub(crate) fn view(options: Options, shared: Shared) -> Box<dyn AnyWidget> {
    // The data and a scale's axis are fixed when the chart is built, so choosing other data, or
    // lying the bars down, which swaps the category and value scales, shows a chart built for it.
    let shown = Map2::read_map(options.data.clone(), options.horizontal.clone(), |data: &Data, lying: &bool| (*data, *lying));
    let (o, s) = (options.clone(), shared.clone());
    let chart = ui!(
        match shown {
            (Data::Regions, false) => chart(o.clone(), s.clone(), Data::Regions, false),
            (Data::Regions, true) => chart(o.clone(), s.clone(), Data::Regions, true),
            (Data::Profit, false) => chart(o.clone(), s.clone(), Data::Profit, false),
            (Data::Profit, true) => chart(o.clone(), s.clone(), Data::Profit, true),
            (Data::Quarters, false) => chart(o.clone(), s.clone(), Data::Quarters, false),
            (Data::Quarters, true) => chart(o.clone(), s.clone(), Data::Quarters, true),
            (Data::Fruit, false) => chart(o.clone(), s.clone(), Data::Fruit, false),
            (Data::Fruit, true) => chart(o.clone(), s.clone(), Data::Fruit, true),
        }
    );

    let side = VStack::new((
        first_section("Bars"),
        text_input("Title", &options.title),
        Picker::new("Data", options.data.clone(), (
            Text::new("Sales by region").tag(Data::Regions),
            Text::new("Income and costs").tag(Data::Profit),
            Text::new("Quarters").tag(Data::Quarters),
            Text::new("Fruit, named by the data").tag(Data::Fruit),
        )),
        Picker::new("Datasets", options.stacking.clone(), (
            Text::new("Side by side").tag(Stacking::Grouped),
            Text::new("Stacked").tag(Stacking::Stacked),
            Text::new("Stacked to 100%").tag(Stacking::Percent),
        )),
        Picker::new("Colours", options.color_by.clone(), (
            Text::new("By dataset").tag(ColorBy::Dataset),
            Text::new("By bar").tag(ColorBy::Bar),
        )),
        Toggle::new("Lying", options.horizontal.clone()),
        slider("Group", &options.category_percentage, 0.1, 1.0, 0.05, percent),
        slider("Bar", &options.bar_percentage, 0.1, 1.0, 0.05, percent),
        Toggle::new("Grid lines", options.grid_lines.clone()),
        section("Zoom"),
        zoom_controls("Categories", &options.zoom),
        Toggle::new("Values fit the categories shown", options.fit_visible.clone()),
    )).cross_axis_alignment(CrossAxisAlignment::Start);

    with_side_bar(chart, side, &shared)
}

/// The bars of `data`, standing or lying: the categories along x or down y.
pub(super) fn chart(options: Options, shared: Shared, data: Data, lying: bool) -> Box<dyn AnyWidget> {
    let (categories, series) = data.data();

    // Grouped, each dataset is a stack of its own; stacked, they share one, normalised for 100%.
    let datasets: Vec<Box<dyn BarDataSet>> = series.into_iter().map(|(name, values)| {
        let stack = Map1::read_map(options.stacking.clone(), move |stacking: &Stacking| match stacking {
            Stacking::Grouped => StackId::new(name),
            Stacking::Stacked => StackId::new("all"),
            Stacking::Percent => StackId::new("all").normalized(),
        });
        let color = Map1::read_map(options.color_by.clone(), |color_by: &ColorBy| match color_by {
            ColorBy::Dataset => DataColor::ByDataSet,
            ColorBy::Bar => DataColor::ByPoint,
        });

        match values {
            Series::Values(values) => values.label(name).stack(stack).color(color).boxed::<dyn BarDataSet>(),
            Series::Named(values) => values.label(name).stack(stack).color(color).boxed::<dyn BarDataSet>(),
        }
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
