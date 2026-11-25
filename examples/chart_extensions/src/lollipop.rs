use carbide::chart::{
    DataSet, DatasetController, IntoSequence, LegendMarker, Orientation, ScaleBounds, Scales, SeriesColors, Shape,
    Shapes, Sequence,
};
use carbide::draw::{Position, Rect, Scalar};
use carbide::environment::Environment;
use carbide::state::{AnyReadState, IntoReadState, ReadStateExt};
use carbide::widget::canvas::CanvasContext;

/// How wide a stem is, in pixels.
const STEM_WIDTH: Scalar = 2.0;

/// The diameter of the dot at the end of a stem, in pixels.
const DOT_DIAMETER: Scalar = 10.0;

/// Draws each point of each dataset as a lollipop: a stem from zero to the point's value, with a
/// dot at the end. Stems start at the end of the value scale nearest zero when zero isn't on it, as
/// on a `LogScale`.
///
/// Its points are read as the built-in controllers read theirs (`Placement::coordinates` and
/// `Scales::coordinates`): numbers at themselves, names on their categories and dates on a time
/// scale, and a dataset's first value picks a category or time scale for a default scale. On
/// categories, the first and last lollipops sit half a step in from the ends, as bars do. Upright
/// by default, the values up the y scale; `horizontal` lays them along x instead. A value scale
/// fitting to the visible data fits to the lollipops inside the index scale's zoom window.
///
/// Datasets and points are coloured as the built-in controllers colour them, from the chart's
/// `SeriesStyle` (see `SeriesColors`): each dataset takes the next palette colour unless it sets
/// one with `DatasetExt::color`, a dataset coloured by point gives each lollipop the next one, and
/// a point's own colour wins.
///
/// A missing point, or one with no place on its scales, has no lollipop. Each lollipop is found by
/// the pointer, by its stem or its dot, for hovering, tooltips and clicks, as two shapes of one
/// point (`Shapes::add`), placed on the index scale for the data at one value (`on_index`).
///
/// It gives its datasets to the chart (`DatasetController::visit_datasets`), so the provided legend
/// entries, legend clicks and tooltips work for it as for the built-in controllers: a named dataset
/// has a legend entry with a point marker, a click on it hides or shows the dataset, and a tooltip
/// describes a lollipop's x and value. A hidden dataset (`DatasetExt::visible`) has no lollipops
/// and adds nothing to the scales, and its legend entry is dimmed.
#[derive(Clone, Debug)]
pub struct LollipopController<D> {
    datasets: D,
    palette_offset: usize,
    /// Whether the lollipops grow along x, as last synced.
    horizontal: Box<dyn AnyReadState<T=bool>>,
}

impl<D: Sequence<dyn DataSet>> LollipopController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet, M, Sequence=D>, M>(datasets: I) -> LollipopController<D> {
        LollipopController {
            datasets: datasets.into_sequence(),
            palette_offset: 0,
            horizontal: IntoReadState::<bool>::into_read_state(false).as_dyn_read(),
        }
    }

    /// Starts the datasets' colours `offset` colours into the palette, so they don't take the same
    /// colours as another controller's datasets in the chart.
    pub fn palette_offset(mut self, offset: usize) -> LollipopController<D> {
        self.palette_offset = offset;
        self
    }

    /// Lays the lollipops along the x scale with `true`, a value or a state of one: each point's x
    /// runs down the y scale and its value along x. `false`, upright, by default, as for the
    /// built-in controllers' `horizontal`.
    pub fn horizontal(mut self, horizontal: impl IntoReadState<bool>) -> LollipopController<D> {
        self.horizontal = horizontal.into_dyn_read_state();
        self
    }
}

impl<D: Sequence<dyn DataSet>> DatasetController for LollipopController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let changed = self.datasets.sync(env);
        self.horizontal.sync(env) | changed
    }

    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        self.datasets.foreach(&mut |_, dataset| {
            let options = dataset.options();
            let (index_id, value_id) = self.orientation().index_and_value_scales(&options);

            // Every dataset registers its names and dates, shown or not, so the scales don't
            // change when it is shown again.
            bounds.register_values(dataset, index_id, value_id);
            let (placement, requests) = bounds.split();

            // On categories, the first and last lollipops sit half a step in from the ends, as
            // bars do, rather than on the chart's edges.
            if !placement.categories(index_id).is_empty() {
                requests.request_offset(index_id);
            }

            if !options.visible {
                return;
            }

            let coordinates = placement.coordinates(index_id, value_id);
            dataset.points(&mut |_, point| {
                if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                    requests.include_point(index_id, value_id, x, y);
                }
            });
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect, shapes: &mut Shapes) {
        let colors = SeriesColors::new(ctx.env(), self.palette_offset);
        let orientation = self.orientation();
        ctx.save();
        ctx.set_line_width(STEM_WIDTH);

        self.datasets.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_id, value_id) = orientation.index_and_value_scales(&options);
            let Some(value_scale) = scales.get(value_id) else {
                return;
            };
            if !options.visible {
                return;
            }

            let dataset_colors = colors.dataset(options.color, dataset_index);
            let coordinates = scales.coordinates(index_id, value_id);

            // Stems grow from zero, or from the end of the value scale nearest it.
            let range = value_scale.range();
            let base = 0.0f64.clamp(range.start.min(range.end), range.start.max(range.end));
            let Some(base) = scales.pixel(value_id, base, area) else {
                return;
            };

            dataset.points(&mut |point_index, point| {
                let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                    return;
                };
                let (Some(index), Some(value)) = (scales.pixel(index_id, x, area), scales.pixel(value_id, y, area)) else {
                    return;
                };

                // In the index frame, along the index scale and along the value scale, turned onto
                // the screen.
                let from = orientation.swap(Position::new(index, base));
                let to = orientation.swap(Position::new(index, value));
                let color = dataset_colors.point(point_index, point.color());

                // Two shapes for one point, so the pointer finds it by either: the stem, in the
                // index frame, and the dot, on the screen.
                let stem = Shape::rect(orientation, index - STEM_WIDTH / 2.0, STEM_WIDTH, base, value);
                shapes.add(dataset_index, point_index, color, stem).on_index(index_id, x);
                shapes.add(dataset_index, point_index, color, Shape::marker(to, DOT_DIAMETER / 2.0)).on_index(index_id, x);

                ctx.begin_path();
                ctx.move_to(from.x, from.y);
                ctx.line_to(to.x, to.y);
                ctx.set_stroke_style(color);
                ctx.stroke();

                ctx.begin_path();
                ctx.circle(to.x, to.y, DOT_DIAMETER);
                ctx.set_fill_style(color);
                ctx.fill();
            });
        });

        ctx.restore();
    }

    /// Every dataset, shown or hidden, for the provided legend entries, legend clicks and
    /// tooltips.
    fn visit_datasets(&self, f: &mut dyn FnMut(usize, &dyn DataSet)) {
        self.datasets.foreach(f);
    }

    /// A point, as each lollipop ends in a dot.
    fn legend_marker(&self, _dataset_index: usize) -> LegendMarker {
        LegendMarker::Point
    }

    /// Horizontal when `horizontal` was last synced `true`.
    fn orientation(&self) -> Orientation {
        if *self.horizontal.value_dyn() { Orientation::Horizontal } else { Orientation::Vertical }
    }

    /// The offset set with `palette_offset`.
    fn palette_offset(&self) -> usize {
        self.palette_offset
    }
}
