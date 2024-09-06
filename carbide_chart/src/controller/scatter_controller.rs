use carbide::color::{BLUE, LIGHT_BLUE, WHITE};
use carbide::draw::{Dimension, Position, Rect, Scalar};
use carbide::widget::EdgeInsets;
use carbide_core::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::scale::{Axis, LinearScale, Scale};

#[derive(Clone, Debug)]
pub struct ScatterController<X: Scale, Y: Scale> {
    x_scale: X,
    y_scale: Y,
    points: Vec<Position>
}

impl ScatterController<LinearScale, LinearScale> {
    pub fn new(points: Vec<Position>) -> ScatterController<LinearScale, LinearScale> {
        ScatterController {
            x_scale: LinearScale::new(Axis::Horizontal),
            y_scale: LinearScale::new(Axis::Vertical),
            points,
        }
    }
}

impl<X: Scale, Y: Scale> DatasetController for ScatterController<X, Y> {
    fn draw(&self, ctx: &mut CanvasContext, env: &mut Environment, padding: EdgeInsets) {

        let x_ticks_width = if self.x_scale.display_ticks() { 10.0 } else { 0.0 };
        let y_ticks_width = if self.x_scale.display_ticks() { 10.0 } else { 0.0 };

        let chart_area = Rect::new(
            Position::new(padding.left + x_ticks_width, padding.top),
            Dimension::new(ctx.dimension().width - padding.left - padding.right, ctx.dimension().height - padding.top - padding.bottom - y_ticks_width)
        );

        self.x_scale.draw_grid(ctx, env, chart_area);
        self.y_scale.draw_grid(ctx, env, chart_area);
        self.x_scale.draw_border(ctx, env, chart_area);
        self.y_scale.draw_border(ctx, env, chart_area);

        ctx.save();

        ctx.set_fill_style(WHITE);

        let x_min = self.x_scale.min();
        let x_max = self.x_scale.max();

        let y_min = self.y_scale.min();
        let y_max = self.y_scale.max();

        for point in &self.points {
            let x = (point.x - x_min) / (x_max - x_min);
            let y = (point.y - y_min) / (y_max - y_min);
            ctx.circle(
                chart_area.width() * x + chart_area.left(),
                chart_area.height() * y + chart_area.bottom(),
                10.0
            );
            ctx.fill()
        }

        ctx.restore();
    }

    fn update_scales_min_max(&mut self) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for point in &self.points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        self.x_scale.set_range(min_x, max_x);
        self.y_scale.set_range(min_y, max_y);
    }
}