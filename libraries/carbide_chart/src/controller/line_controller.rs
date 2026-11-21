use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::orientation::Orientation;
use crate::controller::drawing::{collect_point_bounds, draw_fills, draw_lines, draw_points, drawn_lines, legend_items, Palette};
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::mark::LineDataSet;
use crate::sequence::{IntoSequence, Sequence};
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
///
/// `horizontal` lays the line down, with the same data, as `BarController::horizontal` does for
/// bars: the categories run down the dataset's y scale, the first at the top on a category scale,
/// and the values along its x scale. Everything that follows the categories follows them down:
/// monotone curves stay monotone along them, steps step along them, and areas fill sideways to
/// their baseline.
#[derive(Clone, Debug)]
pub struct LineController<D> {
    pub(crate) dataset_sequence: D,
    palette_offset: usize,
    orientation: Orientation,
}

impl<D: Sequence<dyn LineDataSet>> LineController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn LineDataSet, M, Sequence=D>, M>(datasets: I) -> LineController<D> {
        LineController {
            dataset_sequence: datasets.into_sequence(),
            palette_offset: 0,
            orientation: Orientation::Vertical,
        }
    }

    /// Lays the line down: each point's category runs down the dataset's y scale, the first at
    /// the top on a category scale, and its value along the x scale. The data is read as for an
    /// upright line, so the same datasets draw either way, and line up with horizontal bars.
    pub fn horizontal(mut self) -> LineController<D> {
        self.orientation = Orientation::Horizontal;
        self
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

impl<D: Sequence<dyn LineDataSet>> DatasetController for LineController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Asks for a category index scale (x, or y when `horizontal`), without the bar chart's offset
    /// or grid offset, then reports each dataset's range.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            let options = dataset.options();
            let (index_scale, _) = self.orientation.index_and_value_scales(&options);
            bounds.request_kind(index_scale, ScaleKind::Category);
        });

        collect_point_bounds(&self.dataset_sequence, bounds, true, true, self.orientation);
    }

    /// Each named dataset as a line in its colour, or a square when it is filled.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        legend_items(&self.dataset_sequence, &Palette::from_env(env, self.palette_offset), items, |options| {
            if options.fill == Fill::None { LegendMarker::Line } else { LegendMarker::Square }
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, true, self.orientation);
        let palette = Palette::new(ctx, self.palette_offset);
        draw_fills(&lines, ctx, scales, chart_area, &palette);
        draw_lines(&lines, ctx, chart_area, &palette);
        draw_points(&lines, ctx, &palette, |_| true);

        ctx.restore();
    }
}

#[cfg(test)]
mod tests {
    use carbide::environment::Environment;
    use crate::{DatasetExt, Fill};
    use crate::controller::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId, ScaleKind};
    use super::LineController;

    /// A horizontal line reads the same data as an upright one, with the scales swapped: the
    /// categories go on the dataset's y scale, which is asked to be a category scale, without the
    /// bars' offset, and the values, with the fill's baseline, on its x scale.
    #[test]
    fn horizontal_lines_put_the_categories_on_the_y_scale() {
        let line = LineController::new(vec![12.0, 19.0, 8.0].fill(Fill::Value(-5.0))).horizontal();

        let mut bounds = ScaleBounds::default();
        line.collect_bounds(&mut bounds, &mut Environment::new());

        let y = bounds.get(&ScaleId::Y);
        assert_eq!(y.range(), Some((0.0, 2.0)), "the categories are the points' x values");
        assert_eq!(y.kind(), Some(ScaleKind::Category));
        assert!(!y.offset());

        let x = bounds.get(&ScaleId::X);
        assert_eq!(x.range(), Some((-5.0, 19.0)), "the values, and the baseline they fill to");
        assert_eq!(x.kind(), None);
    }
}
