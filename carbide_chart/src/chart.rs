use carbide::update::{Update, UpdateContext};
use carbide::widget::EdgeInsets;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::environment::Environment;
use carbide_core::render::{Render, RenderContext};
use carbide_core::widget::{CommonWidget, WidgetExt, WidgetId};
use carbide_core::widget::canvas::{Canvas, CanvasContext};
use carbide_derive::Widget;
use crate::controller::{DatasetController, LineController, ScatterController};
use crate::scale::LinearScale;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render, Update)]
pub struct Chart<C> where C: DatasetController {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    dataset_controller: C,
    padding: EdgeInsets
}

impl Chart<LineController<LinearScale, LinearScale>> {
    pub fn new() -> Chart<LineController<LinearScale, LinearScale>> {
        Chart {
            id: Default::default(),
            position: Default::default(),
            dimension: Default::default(),
            dataset_controller: LineController::new(),
            padding: EdgeInsets::all(30.0),
        }
    }

    pub fn scatter(points: Vec<Position>) -> Chart<ScatterController<LinearScale, LinearScale>>{
        println!("{:#?}", points);
        Chart {
            id: Default::default(),
            position: Default::default(),
            dimension: Default::default(),
            dataset_controller: ScatterController::new(points),
            padding: EdgeInsets::all(30.0),
        }
    }
}

impl<C: DatasetController> Chart<C> {

}

impl<C: DatasetController> CommonWidget for Chart<C> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<C: DatasetController> Update for Chart<C> {
    fn update(&mut self, ctx: &mut UpdateContext) {
        self.dataset_controller.update_scales_min_max();
    }
}

impl<C: DatasetController> Render for Chart<C> {
    fn render(&mut self, context: &mut RenderContext) {
        let mut canvas = CanvasContext::new(self.position, self.dimension);

        self.dataset_controller.draw(&mut canvas, context.env, self.padding);

        canvas.render(context);
    }
}

impl<C: DatasetController> WidgetExt for Chart<C> {}
