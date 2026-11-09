use carbide::color::WHITE;
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::drawing::{collect_point_bounds, draw_lines, draw_points};
use crate::{DataSet, DataSetSequence};
use crate::sequence::IntoSequence;
use crate::scale::{Axis, CategoryScale, Scale, ScaleBounds, ScaleId, Scales};

/// Draws each dataset as a line through its points, in order, with the points on top, like
/// Chart.js's line chart (`controller.line.js`, `showLine: true`). The line follows the dataset's
/// `stepped` mode.
///
/// Its default x scale is a `CategoryScale`, as Chart.js's line `overrides` make the index scale a
/// category scale: datasets are lists of values, each at its index. For data with numeric x
/// values, use a `ScatterController` with `show_line(true)`, or set a `LinearScale` with
/// `Chart::scale`.
#[derive(Clone, Debug)]
pub struct LineController<D> {
    dataset_sequence: D,
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> LineController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet<Scalar, Scalar, Scalar>, M, Sequence=D>, M>(datasets: I) -> LineController<D> {
        LineController {
            dataset_sequence: datasets.into_sequence(),
        }
    }
}

impl<D: DataSetSequence<Scalar, Scalar, Scalar>> DatasetController for LineController<D> {
    fn collect_bounds(&self, bounds: &mut ScaleBounds, env: &mut Environment) {
        collect_point_bounds(&self.dataset_sequence, bounds, env);
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();
        ctx.set_fill_style(WHITE);

        draw_lines(&self.dataset_sequence, ctx, scales, chart_area);
        draw_points(&self.dataset_sequence, ctx, scales, chart_area);

        ctx.restore();
    }

    /// A category x scale, without the bar chart's offset or grid offset: Chart.js's line
    /// `overrides` (`_index_: {type: 'category'}`).
    fn default_scale(&self, id: &ScaleId) -> Option<Box<dyn Scale>> {
        (*id == ScaleId::X).then(|| Box::new(CategoryScale::new(Axis::Horizontal)) as Box<dyn Scale>)
    }
}
