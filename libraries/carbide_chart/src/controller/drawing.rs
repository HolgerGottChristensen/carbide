//! Drawing shared by the line and scatter controllers, which differ only in which parts they draw
//! by default: a line chart draws the line through each dataset's points and the points; a scatter
//! chart draws the points, and the line only with `show_line`.

use carbide::color::{Color, GREEN, RED, WHITE};
use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions, DataSetSequence};
use crate::controller::line_path::trace_line;
use crate::scale::{Scale, ScaleBounds, Scales};

/// The colours datasets get, by index, when they don't set one.
const PALETTE: [Color; 3] = [WHITE, GREEN, RED];

/// Reports every dataset's x and y range to the scales it is bound to. Empty datasets have no
/// bounds and contribute nothing.
pub(crate) fn collect_point_bounds<D>(datasets: &D, bounds: &mut ScaleBounds)
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
        let mut points = vec![];
        dataset.points(&mut |_index, point| {
            points.push(Position::new(
                chart_area.width() * x_scale.normalize(point.x()) + chart_area.left(),
                chart_area.height() * (1.0 - y_scale.normalize(point.y())) + chart_area.bottom(),
            ));
        });

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
