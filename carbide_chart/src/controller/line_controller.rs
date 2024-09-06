use carbide::widget::EdgeInsets;
use carbide_core::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::scale::{Axis, LinearScale, Scale};

#[derive(Clone, Debug)]
pub struct LineController<X: Scale, Y: Scale> {
    x_scale: X,
    y_scale: Y,
}

impl LineController<LinearScale, LinearScale> {
    pub fn new() -> LineController<LinearScale, LinearScale> {
        LineController {
            x_scale: LinearScale::new(Axis::Horizontal),
            y_scale: LinearScale::new(Axis::Vertical),
        }
    }
}

impl<X: Scale, Y: Scale> DatasetController for LineController<X, Y> {
    fn draw(&self, ctx: &mut CanvasContext, env: &mut Environment, padding: EdgeInsets) {
        //self.x_scale.draw(ctx, env, padding);
        //self.y_scale.draw(ctx, env, padding);
    }

    fn update_scales_min_max(&mut self) {
        todo!()
    }
}