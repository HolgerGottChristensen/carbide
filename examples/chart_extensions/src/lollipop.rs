use carbide::chart::{
    DataColor, DataSet, DatasetController, IntoSequence, LegendItem, LegendMarker, ScaleBounds, Scales, Shapes,
    Sequence, Value,
};
use carbide::color::{Color, ORANGE};
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide::widget::canvas::CanvasContext;

/// The colour of a dataset that doesn't set one of its own with `DatasetExt::color`. The chart's
/// palette isn't public, so a controller outside the crate can't follow it.
pub const FALLBACK_COLOR: Color = ORANGE;

/// How wide a stem is, in pixels.
const STEM_WIDTH: Scalar = 2.0;

/// The diameter of the dot at the end of a stem, in pixels.
const DOT_DIAMETER: Scalar = 10.0;

/// Draws each point of each dataset as a lollipop: a stem along y from zero to the point's value,
/// with a dot at the end. Stems start at the end of the y scale nearest zero when zero isn't on
/// it, as on a `LogScale`.
///
/// Points are read as plain numbers, so text categories and dates aren't drawn (see the crate's
/// docs). A missing point has no lollipop.
#[derive(Clone, Debug)]
pub struct LollipopController<D> {
    datasets: D,
}

impl<D: Sequence<dyn DataSet>> LollipopController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet, M, Sequence=D>, M>(datasets: I) -> LollipopController<D> {
        LollipopController { datasets: datasets.into_sequence() }
    }
}

impl<D: Sequence<dyn DataSet>> DatasetController for LollipopController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.datasets.sync(env)
    }

    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        self.datasets.foreach(&mut |_, dataset| {
            let options = dataset.options();

            dataset.points(&mut |_, point| {
                if let Some((x, y)) = numbers(point.x(), point.y()) {
                    bounds.include(&options.x_scale, x, x);
                    bounds.include(&options.y_scale, y, y);
                }
            });
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, area: Rect, _shapes: &mut Shapes) {
        ctx.save();
        ctx.set_line_width(STEM_WIDTH);

        self.datasets.foreach(&mut |_, dataset| {
            let options = dataset.options();
            let (Some(x_scale), Some(y_scale)) = (scales.get(&options.x_scale), scales.get(&options.y_scale)) else {
                return;
            };

            let dataset_color = own_color(options.color).unwrap_or(FALLBACK_COLOR);

            // `Rect::bottom` is the top edge of the area on the screen, so a normalized y of 1
            // is at `area.bottom()`.
            let pixel_x = |x: Scalar| area.left() + x_scale.normalize(x) * area.width();
            let pixel_y = |y: Scalar| area.bottom() + (1.0 - y_scale.normalize(y)) * area.height();

            let (low, high) = (y_scale.min().min(y_scale.max()), y_scale.min().max(y_scale.max()));
            let base = pixel_y(0.0f64.clamp(low, high));

            dataset.points(&mut |_, point| {
                let Some((x, y)) = numbers(point.x(), point.y()) else {
                    return;
                };

                let color = own_color(point.color()).unwrap_or(dataset_color);
                let (x, y) = (pixel_x(x), pixel_y(y));

                ctx.begin_path();
                ctx.move_to(x, base);
                ctx.line_to(x, y);
                ctx.set_stroke_style(color);
                ctx.stroke();

                ctx.begin_path();
                ctx.circle(x, y, DOT_DIAMETER);
                ctx.set_fill_style(color);
                ctx.fill();
            });
        });

        ctx.restore();
    }

    fn legend_items(&self, _env: &Environment, items: &mut Vec<LegendItem>) {
        self.datasets.foreach(&mut |_, dataset| {
            let options = dataset.options();

            if let Some(label) = options.label {
                let color = own_color(options.color).unwrap_or(FALLBACK_COLOR);
                items.push(LegendItem { label, color, marker: LegendMarker::Point });
            }
        });
    }
}

/// A point's x and y as numbers, or `None` when either is missing or is a category's name.
fn numbers(x: Option<Value>, y: Option<Value>) -> Option<(Scalar, Scalar)> {
    Some((x?.number()?, y?.number()?))
}

/// The colour `color` sets itself. Palette colours aren't public, so only a fixed colour counts.
fn own_color(color: DataColor) -> Option<Color> {
    match color {
        DataColor::Color(color) => Some(color),
        DataColor::ByDataSet | DataColor::ByPoint | DataColor::Palette(_) => None,
    }
}
