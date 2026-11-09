//! Drawing shared by the line and scatter controllers, which differ only in which parts they
//! draw by default, as in Chart.js: a line chart draws the line through each dataset's points and
//! the points; a scatter chart draws the points, and the line only with `show_line`.

use carbide::color::{Color, GREEN, RED, WHITE};
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, DataSetSequence};
use crate::element::Stepped;
use crate::scale::{Scale, ScaleBounds, Scales};

/// The colours datasets get, by index, when they don't set one.
const PALETTE: [Color; 3] = [WHITE, GREEN, RED];

/// Reports every dataset's x and y range to the scales it is bound to. Empty datasets have no
/// bounds and contribute nothing.
pub(crate) fn collect_point_bounds<D>(datasets: &D, bounds: &mut ScaleBounds, env: &mut Environment)
where
    D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar>,
{
    datasets.datasets(|_dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
        let (Some((x_min, y_min, _)), Some((x_max, y_max, _))) = (dataset.min(), dataset.max()) else {
            return;
        };

        let options = dataset.options(env);

        bounds.include(options.x_scale, x_min, x_max);
        bounds.include(options.y_scale, y_min, y_max);
    });
}

/// Draws a line through each dataset's points, in order, with the dataset's `stepped` mode (a
/// line option in Chart.js).
pub(crate) fn draw_lines<D>(datasets: &D, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect)
where
    D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar>,
{
    datasets.datasets(&mut |dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
        let options = dataset.options(ctx.env());

        let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
            return;
        };

        ctx.begin_path();

        let mut first_in_dataset = true;
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        dataset.points(&mut |_index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
            let x = x_scale.normalize(point.x());
            let y = y_scale.normalize(point.y());

            if first_in_dataset {
                ctx.move_to(
                    chart_area.width() * x + chart_area.left(),
                    chart_area.height() * (1.0 - y) + chart_area.bottom(),
                );
                first_in_dataset = false;
                prev_x = x;
                prev_y = y;
            } else {
                match options.stepped {
                    Stepped::Before => {
                        ctx.line_to(
                            chart_area.width() * prev_x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                    }
                    Stepped::After => {
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - prev_y) + chart_area.bottom(),
                        );
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                    }
                    Stepped::Middle => {
                        let middle_x = (x - prev_x) / 2.0 + prev_x;
                        ctx.line_to(
                            chart_area.width() * middle_x + chart_area.left(),
                            chart_area.height() * (1.0 - prev_y) + chart_area.bottom(),
                        );
                        ctx.line_to(
                            chart_area.width() * middle_x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                    }
                    Stepped::MiddleVertical => {
                        let middle_y = (y - prev_y) / 2.0 + prev_y;
                        ctx.line_to(
                            chart_area.width() * prev_x + chart_area.left(),
                            chart_area.height() * (1.0 - middle_y) + chart_area.bottom(),
                        );
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - middle_y) + chart_area.bottom(),
                        );
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                    }
                    Stepped::None => {
                        ctx.line_to(
                            chart_area.width() * x + chart_area.left(),
                            chart_area.height() * (1.0 - y) + chart_area.bottom(),
                        );
                    }
                }

                prev_x = x;
                prev_y = y;
            }
        });

        ctx.set_stroke_style(dataset_color(&options, dataset_index));
        ctx.stroke();
    });
}

/// Draws a filled circle at each of each dataset's points, in the point's colour if it has one,
/// otherwise the dataset's.
pub(crate) fn draw_points<D>(datasets: &D, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect)
where
    D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar>,
{
    datasets.datasets(&mut |dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
        let options = dataset.options(ctx.env());

        let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
            return;
        };

        dataset.points(&mut |_index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
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
    Some((scales.get(options.x_scale)?, scales.get(options.y_scale)?))
}
