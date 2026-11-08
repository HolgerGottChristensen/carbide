use carbide::color::{GREEN, RED, WHITE};
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, DataSetSequence};
use crate::element::Stepped;
use crate::scale::{ScaleBounds, Scales};

#[derive(Clone, Debug)]
pub struct ScatterController<D: DataSetSequence> {
    dataset_sequence: D
}

impl<D: DataSetSequence> ScatterController<D> {
    pub fn new(dataset: D) -> ScatterController<D> {
        ScatterController {
            dataset_sequence: dataset,
        }
    }
}

impl<D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar> + Clone> DatasetController for ScatterController<D> {
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment) {
        self.dataset_sequence.datasets(|_dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
            // An empty dataset has no bounds and contributes nothing.
            let (Some((x_min, y_min, _)), Some((x_max, y_max, _))) = (dataset.min(), dataset.max()) else {
                return;
            };

            let options = dataset.options(env);

            bounds.include(options.x_scale, x_min, x_max);
            bounds.include(options.y_scale, y_min, y_max);
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        ctx.set_fill_style(WHITE);

        let colors = [WHITE, GREEN, RED];

        self.dataset_sequence.datasets(&mut |dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
            let options = dataset.options(ctx.env());

            let Some((x_min, x_max, y_min, y_max)) = scale_ranges(scales, &options) else {
                return;
            };

            ctx.begin_path();

            let mut first_in_dataset = true;
            let mut prev_x = 0.0;
            let mut prev_y = 0.0;

            dataset.points(&mut |_index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
                let x = (point.x() - x_min) / (x_max - x_min);
                let y = (point.y() - y_min) / (y_max - y_min);

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

            let color = match options.color {
                DataColor::Inherit => colors[dataset_index % colors.len()],
                DataColor::Color(c) => c
            };

            ctx.set_stroke_style(color);
            ctx.stroke();
        });

        self.dataset_sequence.datasets(&mut |dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
            let options = dataset.options(ctx.env());

            let Some((x_min, x_max, y_min, y_max)) = scale_ranges(scales, &options) else {
                return;
            };

            dataset.points(&mut |_index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
                let x = (point.x() - x_min) / (x_max - x_min);
                let y = (point.y() - y_min) / (y_max - y_min);
                ctx.begin_path();
                ctx.circle(
                    chart_area.width() * x + chart_area.left(),
                    chart_area.height() * (1.0 - y) + chart_area.bottom(),
                    7.0
                );

                match point.color() {
                    DataColor::Inherit => {
                        match options.color {
                            DataColor::Inherit => ctx.set_fill_style(colors[dataset_index % colors.len()]),
                            DataColor::Color(color) => ctx.set_fill_style(color)
                        }
                    },
                    DataColor::Color(color) => ctx.set_fill_style(color)
                }

                ctx.fill();
            });
        });


        /*self.dataset_sequence.datasets(&mut |point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {

        });*/

        ctx.restore();
    }
}

/// The fitted ranges (x min, x max, y min, y max) of the scales a dataset is bound to, or `None`
/// when the chart has no scale with one of those ids. The chart warns about unknown ids when it
/// fits its scales (see `Chart::fit_scales`).
fn scale_ranges(scales: &Scales, options: &DataSetOptions) -> Option<(Scalar, Scalar, Scalar, Scalar)> {
    let x_scale = scales.get(options.x_scale)?;
    let y_scale = scales.get(options.y_scale)?;

    Some((x_scale.min(), x_scale.max(), y_scale.min(), y_scale.max()))
}