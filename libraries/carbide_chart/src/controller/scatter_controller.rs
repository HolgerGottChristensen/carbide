use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::drawing::{collect_point_bounds, draw_fills, draw_lines, draw_points, drawn_lines, Palette};
use crate::{DataSet, DataSetSequence};
use crate::sequence::IntoSequence;
use crate::scale::{ScaleBounds, Scales};

/// Draws each dataset's points: points only by default. With `show_line(true)` it also draws a line
/// through them in order, following the dataset's `interpolation`, as a line chart does.
///
/// Its x and y scales stay linear, so it suits data with numeric x values. For lists of values at
/// their indices, use a `LineController`.
#[derive(Clone, Debug)]
pub struct ScatterController<D> {
    dataset_sequence: D,
    show_line: bool,
    palette_offset: usize,
}

impl<D: DataSetSequence> ScatterController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet, M, Sequence=D>, M>(datasets: I) -> ScatterController<D> {
        ScatterController {
            dataset_sequence: datasets.into_sequence(),
            show_line: false,
            palette_offset: 0,
        }
    }

    /// Starts the colours of this controller's datasets `offset` colours into the palette:
    /// dataset `i` takes palette colour `offset + i`, and so do the points of a dataset coloured
    /// by point. Defaults to 0. A chart with several controllers sets it so their datasets don't
    /// share colours, as each controller counts its datasets from 0. Datasets that pick a palette
    /// colour themselves (`DataColor::Palette`) aren't shifted.
    pub fn palette_offset(mut self, offset: usize) -> ScatterController<D> {
        self.palette_offset = offset;
        self
    }

    /// Also draws a line through each dataset's points, under the points. Defaults to `false`.
    pub fn show_line(mut self, show_line: bool) -> ScatterController<D> {
        self.show_line = show_line;
        self
    }
}

impl<D: DataSetSequence> DatasetController for ScatterController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        collect_point_bounds(&self.dataset_sequence, bounds, self.show_line, false);
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        // Scatter charts don't stack: each dataset's points are drawn at their own values.
        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, false);

        let palette = Palette::new(ctx, self.palette_offset);

        if self.show_line {
            draw_fills(&lines, ctx, scales, chart_area, &palette);
            draw_lines(&lines, ctx, chart_area, &palette);
        }

        draw_points(&lines, ctx, &palette);

        ctx.restore();
    }
}
