use carbide::color::{GREEN, RED, WHITE};
use carbide::draw::{Dimension, Position, Rect, Scalar};
use carbide::widget::EdgeInsets;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::{DataColor, DataPoint, DataSet, DataSetOptions, DataSetSequence};
use crate::element::Stepped;
use crate::scale::{Axis, LinearScale, Scale};

#[derive(Clone, Debug)]
pub struct ScatterController<X: Scale, Y: Scale, D: DataSetSequence> {
    default_x_scale: X,
    default_y_scale: Y,
    dataset_sequence: D
}

impl ScatterController<LinearScale, LinearScale, Vec<(Scalar, Scalar)>> {
    pub fn new<D: DataSetSequence>(dataset: D) -> ScatterController<LinearScale, LinearScale, D> {
        ScatterController {
            default_x_scale: LinearScale::new(Axis::Horizontal),
            default_y_scale: LinearScale::new(Axis::Vertical),
            dataset_sequence: dataset,
        }
    }
}

impl<X: Scale, Y: Scale, D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar> + Clone> DatasetController for ScatterController<X, Y, D> {
    fn draw(&self, ctx: &mut CanvasContext, padding: EdgeInsets) {

        let x_ticks_width = if self.default_x_scale.display_ticks() { 10.0 } else { 0.0 };
        let y_ticks_width = if self.default_x_scale.display_ticks() { 10.0 } else { 0.0 };

        let chart_area = Rect::new(
            Position::new(padding.left + x_ticks_width, padding.top),
            Dimension::new(ctx.dimension().width - padding.left - padding.right, ctx.dimension().height - padding.top - padding.bottom - y_ticks_width)
        );

        self.default_x_scale.draw_grid(ctx, chart_area);
        self.default_y_scale.draw_grid(ctx, chart_area);
        self.default_x_scale.draw_border(ctx, chart_area);
        self.default_y_scale.draw_border(ctx, chart_area);

        ctx.save();

        ctx.set_fill_style(WHITE);

        let x_min = self.default_x_scale.min();
        let x_max = self.default_x_scale.max();

        let y_min = self.default_y_scale.min();
        let y_max = self.default_y_scale.max();

        let colors = vec![WHITE, GREEN, RED];

        self.dataset_sequence.datasets(&mut |dataset_index, dataset: &dyn DataSet<X=Scalar, Y=Scalar, Z=Scalar>| {
            let options = dataset.options(ctx.env());

            ctx.begin_path();

            let mut first_in_dataset = true;
            let mut prev_x = 0.0;
            let mut prev_y = 0.0;

            dataset.points(&mut |index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
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
            dataset.points(&mut |index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
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

    fn update_scales_min_max(&mut self) {
        let min = self.dataset_sequence.min();
        let max = self.dataset_sequence.max();

        self.default_x_scale.set_range(min.x(), max.x());
        self.default_y_scale.set_range(min.y(), max.y());
    }
}