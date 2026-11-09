use carbide::color::WHITE;
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::drawing::{collect_point_bounds, draw_lines, draw_points};
use crate::{DataSet, DataSetSequence};
use crate::sequence::IntoSequence;
use crate::scale::{ScaleBounds, Scales};

/// Draws each dataset's points, like Chart.js's scatter chart (`controller.scatter.js`): points
/// only by default (`showLine: false`). With `show_line(true)` it also draws a line through them
/// in order, following the dataset's `stepped` mode, as a line chart does.
///
/// Its x and y scales stay linear, as Chart.js's scatter `overrides` make them, so it suits data
/// with numeric x values. For lists of values at their indices, use a `LineController`.
#[derive(Clone, Debug)]
pub struct ScatterController<D> {
    dataset_sequence: D,
    show_line: bool,
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> ScatterController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet<Scalar, Scalar, Scalar>, M, Sequence=D>, M>(datasets: I) -> ScatterController<D> {
        ScatterController {
            dataset_sequence: datasets.into_sequence(),
            show_line: false,
        }
    }

    /// Also draws a line through each dataset's points, under the points, like Chart.js's
    /// `showLine`. Defaults to `false`.
    pub fn show_line(mut self, show_line: bool) -> ScatterController<D> {
        self.show_line = show_line;
        self
    }
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> DatasetController for ScatterController<D> {
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment) {
        collect_point_bounds(&self.dataset_sequence, bounds, env);
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();
        ctx.set_fill_style(WHITE);

        if self.show_line {
            draw_lines(&self.dataset_sequence, ctx, scales, chart_area);
        }

        draw_points(&self.dataset_sequence, ctx, scales, chart_area);

        ctx.restore();
    }
}
