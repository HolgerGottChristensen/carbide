use carbide::color::{GREEN, RED, WHITE};
use carbide::draw::{Dimension, Position, Rect, Scalar};
use carbide::widget::EdgeInsets;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::{DataColor, DataPoint, DataSetSequence};
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

impl<X: Scale, Y: Scale, D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar>> DatasetController for ScatterController<X, Y, D> {
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

        self.dataset_sequence.foreach(&mut |index, point: &dyn DataPoint<X=Scalar, Y=Scalar, Z=Scalar>| {
            let x = (point.x() - x_min) / (x_max - x_min);
            let y = (point.y() - y_min) / (y_max - y_min);
            ctx.begin_path();
            ctx.circle(
                chart_area.width() * x + chart_area.left(),
                chart_area.height() * (1.0 - y) + chart_area.bottom(),
                10.0
            );

            match point.color() {
                DataColor::Inherit => ctx.set_fill_style(colors[index % colors.len()]),
                DataColor::Color(color) => ctx.set_fill_style(color)
            }

            ctx.fill();
        });

        ctx.restore();
    }

    fn update_scales_min_max(&mut self) {
        let min = self.dataset_sequence.min();
        let max = self.dataset_sequence.max();

        self.default_x_scale.set_range(min.x(), max.x());
        self.default_y_scale.set_range(min.y(), max.y());
    }
}