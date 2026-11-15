use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::drawing::{collect_point_bounds, draw_fills, draw_lines, draw_points, drawn_lines, legend_items, Palette};
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::{DataSet, DataSetSequence};
use crate::sequence::IntoSequence;
use crate::scale::{ScaleBounds, ScaleKind, Scales};

/// Draws each dataset as a line through its points, in order, with the points on top. The line
/// follows the dataset's `interpolation`: straight by default, or smooth, monotone or stepped.
///
/// Datasets in the same stack (`DatasetExt::stack`) are drawn at their running totals, with one
/// sum for positive and negative values alike, so a negative value lowers the lines above it, and
/// the bands between neighbouring lines never have gaps. With `Fill::Stack` they make a stacked
/// area chart.
///
/// Its default x scale is a `CategoryScale`, as its datasets are usually lists of values, each at
/// its index. For data with numeric x values, use a `ScatterController` with `show_line(true)`, or
/// set a `LinearScale` with `Chart::scale`.
#[derive(Clone, Debug)]
pub struct LineController<D> {
    pub(crate) dataset_sequence: D,
    palette_offset: usize,
}

impl<D: DataSetSequence> LineController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn DataSet, M, Sequence=D>, M>(datasets: I) -> LineController<D> {
        LineController {
            dataset_sequence: datasets.into_sequence(),
            palette_offset: 0,
        }
    }

    /// Starts the colours of this controller's datasets `offset` colours into the palette:
    /// dataset `i` takes palette colour `offset + i`, and so do the points of a dataset coloured
    /// by point. Defaults to 0. A chart with several controllers sets it so their datasets don't
    /// share colours, as each controller counts its datasets from 0. Datasets that pick a palette
    /// colour themselves (`DataColor::Palette`) aren't shifted.
    pub fn palette_offset(mut self, offset: usize) -> LineController<D> {
        self.palette_offset = offset;
        self
    }
}

impl<D: DataSetSequence> DatasetController for LineController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Asks for a category x scale, without the bar chart's offset or grid offset, then reports
    /// each dataset's range.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            bounds.request_kind(&dataset.options().x_scale, ScaleKind::Category);
        });

        collect_point_bounds(&self.dataset_sequence, bounds, true, true);
    }

    /// Each named dataset as a line in its colour, or a square when it is filled.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        legend_items(&self.dataset_sequence, &Palette::from_env(env, self.palette_offset), items, |options| {
            if options.fill == Fill::None { LegendMarker::Line } else { LegendMarker::Square }
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, true);
        let palette = Palette::new(ctx, self.palette_offset);
        draw_fills(&lines, ctx, scales, chart_area, &palette);
        draw_lines(&lines, ctx, chart_area, &palette);
        draw_points(&lines, ctx, &palette);

        ctx.restore();
    }
}
