use carbide::draw::Rect;
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::orientation::Orientation;
use crate::controller::drawing::{collect_point_bounds, draw_fills, draw_lines, draw_points, drawn_lines, first_mark_entry, legend_items, Palette};
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::mark::{LineCanvas, LineDataSet, MarkRole};
use crate::controller::stacking::{stack_offset, StackKey, StackOffsets, StackSums};
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

    /// How far stacking moves the lines of the stacked datasets with overlay marks (see
    /// `StackOffsets`), with one running sum per stack, as the lines are stacked. Empty when no
    /// dataset has an overlay, and for datasets that don't stack, including those whose line a mark
    /// replaces (`replaced`).
    fn stack_offsets(&self, replaced: &[usize]) -> StackOffsets {
        let mut offsets = StackOffsets::default();

        let mut overlays = false;
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| overlays |= has_mark(dataset, MarkRole::Overlay));
        if !overlays {
            return offsets;
        }

        let mut sums = StackSums::single();

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            if replaced.contains(&dataset_index) {
                return;
            }

            let key = StackKey::new(&dataset.options(), dataset_index);
            if !matches!(key, StackKey::Stacked { .. }) {
                return;
            }

            let recorded = has_mark(dataset, MarkRole::Overlay);
            dataset.points(&mut |_index, point| {
                if let (Some(x), Some(y)) = (point.x(), point.y()) {
                    let (_, total) = sums.add(&key, x, y);
                    if recorded {
                        offsets.record(dataset_index, x, total - y);
                    }
                }
            });
        });

        offsets
    }

    /// Draws every dataset's marks with `role`, in dataset order, each on its dataset's scales.
    /// Datasets without such marks are passed over without reading their options.
    fn draw_marks(&self, role: MarkRole, offsets: &StackOffsets, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, palette: &Palette) {
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            if !has_mark(dataset, role) {
                return;
            }

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
                stack_offsets: offsets.dataset(dataset_index),
            };

            dataset.line_marks(&mut |mark| {
                if mark.role() == role {
                    mark.draw(&mut canvas);
                }
            });
        });
    }
}

/// Whether one of the dataset's marks has `role`.
fn has_mark(dataset: &dyn LineDataSet, role: MarkRole) -> bool {
    let mut found = false;
    dataset.line_marks(&mut |mark| found |= mark.role() == role);
    found
}

impl<D: Sequence<dyn LineDataSet>> DatasetController for LineController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Asks for a category index scale (x, or y when `horizontal`), without the bar chart's offset
    /// or grid offset, then reports each dataset's range: its points, or, when a mark replaces its
    /// line, what the mark draws.
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        let replaced = self.replaced_lines();
        let offsets = self.stack_offsets(&replaced);

        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let options = dataset.options();
            let (index_scale, value_scale) = self.orientation.index_and_value_scales(&options);
            bounds.request_kind(index_scale, ScaleKind::Category);

            // A mark's values move with the dataset's line where it is stacked.
            dataset.line_marks(&mut |mark| {
                mark.bounds(&mut |x, low, high| {
                    let offset = stack_offset(offsets.dataset(dataset_index), x);
                    bounds.include(index_scale, x, x);
                    bounds.include(value_scale, low + offset, high + offset);
                });
            });
        });

        collect_point_bounds(&self.dataset_sequence, bounds, true, true, self.orientation, |dataset_index| !replaced.contains(&dataset_index));
    }

    /// Each named dataset as a line in its colour, or a square when it is filled or a mark, such
    /// as a band, replaces its line, unless one of its marks gives an entry of its own
    /// (`LineMark::legend`).
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        let replaced = self.replaced_lines();
        legend_items(
            &self.dataset_sequence,
            env,
            &Palette::from_env(env, self.palette_offset),
            items,
            |dataset_index, options| {
                if options.fill == Fill::None && !replaced.contains(&dataset_index) { LegendMarker::Line } else { LegendMarker::Square }
            },
            |dataset, context| first_mark_entry(|f| dataset.line_marks(&mut |mark| f(mark.legend(context)))),
        );
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        let replaced = self.replaced_lines();
        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, true, self.orientation, |dataset_index| !replaced.contains(&dataset_index));
        let palette = Palette::new(ctx, self.palette_offset);

        // Marks replacing lines are usually areas, so they go first, under every line; overlays
        // go on top of everything.
        let offsets = self.stack_offsets(&replaced);
        self.draw_marks(MarkRole::Primary, &offsets, ctx, scales, chart_area, &palette);
        draw_fills(&lines, ctx, scales, chart_area, &palette);
        draw_lines(&lines, ctx, chart_area, &palette);
        draw_points(&lines, ctx, &palette, |_| true);
        self.draw_marks(MarkRole::Overlay, &offsets, ctx, scales, chart_area, &palette);

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

    /// The stack offsets are only worked out for stacked datasets with an overlay mark, as for bar
    /// controllers. Plain stacked lines, and error bars on lines that don't stack, get none.
    #[test]
    fn stack_offsets_are_only_kept_for_stacked_overlays() {
        use crate::{Estimate, RangeDataSetExt, StackId};
        use crate::controller::stacking::stack_offset;

        const ALL: StackId = StackId::new("all");
        let measured = vec![Estimate { x: 0.0, y: 5.0, low: 4.0, high: 7.0 }];

        let plain = LineController::new((vec![10.0].stack(ALL), vec![5.0].stack(ALL)));
        assert!(plain.stack_offsets(&plain.replaced_lines()).is_empty());

        let unstacked = LineController::new((vec![10.0], measured.clone().error_bars()));
        assert!(unstacked.stack_offsets(&unstacked.replaced_lines()).is_empty());

        let stacked = LineController::new((vec![10.0].stack(ALL), measured.error_bars().stack(ALL)));
        let offsets = stacked.stack_offsets(&stacked.replaced_lines());
        assert!(offsets.dataset(0).is_none());
        assert_eq!(stack_offset(offsets.dataset(1), 0.0), 10.0, "the error bars sit on the line at the total of 15");
    }

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
