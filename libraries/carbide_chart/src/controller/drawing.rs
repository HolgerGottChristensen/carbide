//! Drawing shared by the line and scatter controllers, which differ only in which parts they draw
//! by default: a line chart draws the line through each dataset's points and the points; a scatter
//! chart draws the points, and the line only with `show_line`. Where a line is drawn, so is the
//! area under it, for datasets that `fill`.

use carbide::color::{Color, GREEN, RED, WHITE};
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, DataSetSequence, Fill};
use crate::controller::line_path::trace_line;
use crate::scale::{Scale, ScaleBounds, Scales};

/// The colours datasets get, by index, when they don't set one.
const PALETTE: [Color; 3] = [WHITE, GREEN, RED];

/// How opaque a filled area is, so the grid and the areas behind it show through.
const FILL_OPACITY: f32 = 0.3;

/// Reports every dataset's x and y range to the scales it is bound to. With `fills`, when the
/// controller draws lines, a filled dataset's baseline value is included too, so the area's base
/// is on the chart. Empty datasets have no bounds and contribute nothing.
pub(crate) fn collect_point_bounds<D>(datasets: &D, bounds: &mut ScaleBounds, fills: bool)
where
    D: DataSetSequence<Scalar, Scalar, Scalar>,
{
    datasets.foreach(&mut |_dataset_index, dataset| {
        let (Some((x_min, y_min, _)), Some((x_max, y_max, _))) = (dataset.min(), dataset.max()) else {
            return;
        };

        let options = dataset.options();

        bounds.include(&options.x_scale, x_min, x_max);
        bounds.include(&options.y_scale, y_min, y_max);

        if fills && let Some(baseline) = options.fill.baseline_value() {
            bounds.include(&options.y_scale, baseline, baseline);
        }
    });
}

/// The dataset's points in pixels, mapped through its scales into `chart_area`.
fn pixel_points(dataset: &dyn DataSet<Scalar, Scalar, Scalar>, x_scale: &dyn Scale, y_scale: &dyn Scale, chart_area: Rect) -> Vec<Position> {
    let mut points = vec![];
    dataset.points(&mut |_index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
        points.push(Position::new(
            chart_area.width() * x_scale.normalize(point.x()) + chart_area.left(),
            chart_area.height() * (1.0 - y_scale.normalize(point.y())) + chart_area.bottom(),
        ));
    });
    points
}

/// Fills the area between each filled dataset's line and its `Fill` baseline, in the dataset's
/// colour, translucent. The area's top follows the line exactly, through `trace_line`, then
/// closes along the baseline. A dataset needs two points to have an area.
pub(crate) fn draw_fills<D>(datasets: &D, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect)
where
    D: DataSetSequence<Scalar, Scalar, Scalar>,
{
    datasets.foreach(&mut |dataset_index, dataset| {
        let options = dataset.options();

        if options.fill == Fill::None {
            return;
        }

        let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
            return;
        };

        let points = pixel_points(dataset, x_scale, y_scale, chart_area);
        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            return;
        };

        if points.len() < 2 {
            return;
        }

        // `Rect`'s top and bottom are swapped on screen (see `Scale::draw_grid`): the larger y is
        // the bottom edge.
        let screen_bottom = chart_area.top().max(chart_area.bottom());
        let screen_top = chart_area.top().min(chart_area.bottom());

        let baseline = match options.fill {
            Fill::Bottom => screen_bottom,
            Fill::Top => screen_top,
            fill => {
                // A value outside the scale's range (possible on a scale set with `Chart::scale`)
                // fills to the nearest end of it, as bars grow from the end nearest zero.
                let value = fill.baseline_value().unwrap_or(0.0);
                let (low, high) = (y_scale.min().min(y_scale.max()), y_scale.min().max(y_scale.max()));
                chart_area.height() * (1.0 - y_scale.normalize(value.clamp(low, high))) + chart_area.bottom()
            }
        };

        ctx.begin_path();
        trace_line(ctx, &points, options.interpolation, chart_area);
        ctx.line_to(last.x, baseline);
        ctx.line_to(first.x, baseline);
        ctx.close_path();

        let Rgba(r, g, b, a) = Rgba::from(dataset_color(&options, dataset_index));
        ctx.set_fill_style(rgba(r, g, b, a * FILL_OPACITY));
        ctx.fill();
    });
}

/// Draws a line through each dataset's points, in order, with the dataset's `interpolation`
/// (straight, smooth, monotone or stepped).
pub(crate) fn draw_lines<D>(datasets: &D, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect)
where
    D: DataSetSequence<Scalar, Scalar, Scalar>,
{
    datasets.foreach(&mut |dataset_index, dataset| {
        let options = dataset.options();

        let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
            return;
        };

        // Curves need each point's neighbours, so the points are mapped to pixels first.
        let points = pixel_points(dataset, x_scale, y_scale, chart_area);

        ctx.begin_path();
        trace_line(ctx, &points, options.interpolation, chart_area);
        ctx.set_stroke_style(dataset_color(&options, dataset_index));
        ctx.stroke();
    });
}

/// Draws a filled circle at each of each dataset's points, in the point's colour if it has one,
/// otherwise the dataset's.
pub(crate) fn draw_points<D>(datasets: &D, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect)
where
    D: DataSetSequence<Scalar, Scalar, Scalar>,
{
    datasets.foreach(&mut |dataset_index, dataset| {
        let options = dataset.options();

        let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
            return;
        };

        dataset.points(&mut |_index, point| {
            let x = x_scale.normalize(point.x());
            let y = y_scale.normalize(point.y());
            ctx.begin_path();
            ctx.circle(
                chart_area.width() * x + chart_area.left(),
                chart_area.height() * (1.0 - y) + chart_area.bottom(),
                7.0
            );

            match point.color() {
                DataColor::Inherit => ctx.set_fill_style(dataset_color(&options, dataset_index)),
                DataColor::Color(color) => ctx.set_fill_style(color),
            }

            ctx.fill();
        });
    });
}

/// The dataset's own colour, or the palette's colour for its index in the controller.
pub(crate) fn dataset_color(options: &DataSetOptions, dataset_index: usize) -> Color {
    match options.color {
        DataColor::Inherit => PALETTE[dataset_index % PALETTE.len()],
        DataColor::Color(color) => color,
    }
}

/// The x and y scales a dataset is bound to, or `None` when the chart has no scale with one of
/// those ids. The chart warns about unknown ids when it fits its scales (see `Chart::fit_scales`).
pub(crate) fn dataset_scales<'a>(scales: &'a Scales, options: &DataSetOptions) -> Option<(&'a dyn Scale, &'a dyn Scale)> {
    Some((scales.get(&options.x_scale)?, scales.get(&options.y_scale)?))
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use crate::{DatasetExt, Fill, LineController, ScatterController};
    use crate::controller::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId};

    fn y_range(controller: &dyn DatasetController) -> Option<(f64, f64)> {
        let mut bounds = ScaleBounds::default();
        controller.collect_bounds(&mut bounds, &mut Environment::new());
        bounds.get(&ScaleId::Y).range()
    }

    /// A filled line includes its baseline value in the y bounds, so the area's base is on the
    /// chart; filling to the chart area's edges includes nothing. A scatter chart only does so
    /// when it draws lines, as only then is the area drawn.
    #[test]
    fn filled_lines_include_their_baseline() {
        let values = vec![100.0, 150.0, 200.0];

        assert_eq!(y_range(&LineController::new(values.clone())), Some((100.0, 200.0)));
        assert_eq!(y_range(&LineController::new(values.clone().fill(Fill::Origin))), Some((0.0, 200.0)));
        assert_eq!(y_range(&LineController::new(values.clone().fill(Fill::Value(250.0)))), Some((100.0, 250.0)));
        assert_eq!(y_range(&LineController::new(values.clone().fill(Fill::Bottom))), Some((100.0, 200.0)));

        assert_eq!(y_range(&ScatterController::new(values.clone().fill(Fill::Origin))), Some((100.0, 200.0)), "a scatter chart without lines draws no area");
        assert_eq!(y_range(&ScatterController::new(values.fill(Fill::Origin)).show_line(true)), Some((0.0, 200.0)));
    }
}
