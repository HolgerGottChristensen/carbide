use std::collections::HashMap;
use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::orientation::Orientation;
use crate::controller::drawing::{collect_point_bounds, draw_fills, draw_lines, draw_points, drawn_lines, legend_items, Palette};
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::mark::{LineCanvas, LineDataSet, MarkRole};
use crate::controller::stacking::{StackKey, StackSums};
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
/// A dataset with a mark (see `LineDataSet`) draws the mark in place of its line and points, such
/// as a band between each point's low and high value (`RangeDataSetExt::range`). Those datasets
/// don't stack, and are drawn before the other datasets' lines, as bands are areas.
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

impl<D: Sequence<dyn LineDataSet>> LineController<D> {
    /// The indices of the datasets whose lines a mark replaces.
    fn replaced_lines(&self) -> Vec<usize> {
        let mut replaced = vec![];
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let mut primary = false;
            dataset.line_marks(&mut |mark| primary |= mark.role() == MarkRole::Primary);
            if primary {
                replaced.push(dataset_index);
            }
        });
        replaced
    }

    /// How far stacking moves each dataset's line, by dataset index and then by x value: where
    /// the line is drawn, less the point's own value, with one running sum per stack, as the lines
    /// are stacked. Datasets whose line a mark replaces don't stack, and aren't in it.
    fn stack_offsets(&self) -> Vec<HashMap<u64, Scalar>> {
        let replaced = self.replaced_lines();
        let mut sums = StackSums::single();
        let mut offsets = vec![];

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let mut dataset_offsets = HashMap::new();

            if !replaced.contains(&dataset_index) {
                let key = StackKey::new(&dataset.options(), dataset_index);
                dataset.points(&mut |_index, point| {
                    if let (Some(x), Some(y)) = (point.x(), point.y()) {
                        let (_, total) = sums.add(&key, x, y);
                        dataset_offsets.insert((x + 0.0).to_bits(), total - y);
                    }
                });
            }

            offsets.push(dataset_offsets);
        });

        offsets
    }

    /// Draws every dataset's marks with `role`, in dataset order, each on its dataset's scales.
    fn draw_marks(&self, role: MarkRole, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, palette: &Palette) {
        let offsets = self.stack_offsets();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_id, value_id) = self.orientation.index_and_value_scales(&options);
            let (Some(index_scale), Some(value_scale)) = (scales.get(index_id), scales.get(value_id)) else {
                return;
            };

            let mut canvas = LineCanvas {
                ctx: &mut *ctx,
                orientation: self.orientation,
                index_scale,
                value_scale,
                area: chart_area,
                options: &options,
                dataset_index,
                palette,
                stack_offsets: &offsets[dataset_index],
            };

            dataset.line_marks(&mut |mark| {
                if mark.role() == role {
                    mark.draw(&mut canvas);
                }
            });
        });
    }
}

impl<D: Sequence<dyn LineDataSet>> DatasetController for LineController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Asks for a category index scale (x, or y when `horizontal`), without the bar chart's offset
    /// or grid offset, then reports each dataset's range: its points, or, when a mark replaces its
    /// line, what the mark draws.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        let offsets = self.stack_offsets();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_scale, value_scale) = self.orientation.index_and_value_scales(&options);
            bounds.request_kind(index_scale, ScaleKind::Category);

            // A mark's values move with the dataset's line where it is stacked.
            dataset.line_marks(&mut |mark| {
                mark.bounds(&mut |x, low, high| {
                    let offset = offsets[dataset_index].get(&(x + 0.0).to_bits()).copied().unwrap_or(0.0);
                    bounds.include(index_scale, x, x);
                    bounds.include(value_scale, low + offset, high + offset);
                });
            });
        });

        let replaced = self.replaced_lines();
        collect_point_bounds(&self.dataset_sequence, bounds, true, true, self.orientation, |dataset_index| !replaced.contains(&dataset_index));
    }

    /// Each named dataset as a line in its colour, or a square when it is filled or a mark, such
    /// as a band, replaces its line.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        let replaced = self.replaced_lines();
        legend_items(&self.dataset_sequence, &Palette::from_env(env, self.palette_offset), items, |dataset_index, options| {
            if options.fill == Fill::None && !replaced.contains(&dataset_index) { LegendMarker::Line } else { LegendMarker::Square }
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        let replaced = self.replaced_lines();
        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, true, self.orientation, |dataset_index| !replaced.contains(&dataset_index));
        let palette = Palette::new(ctx, self.palette_offset);

        // Marks replacing lines are usually areas, so they go first, under every line; overlays
        // go on top of everything.
        self.draw_marks(MarkRole::Primary, ctx, scales, chart_area, &palette);
        draw_fills(&lines, ctx, scales, chart_area, &palette);
        draw_lines(&lines, ctx, chart_area, &palette);
        draw_points(&lines, ctx, &palette, |_| true);
        self.draw_marks(MarkRole::Overlay, ctx, scales, chart_area, &palette);

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
