use carbide::draw::{Rect, Scalar};
use carbide::environment::Environment;
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::DatasetController;
use crate::controller::orientation::Orientation;
use crate::controller::drawing::{collect_point_bounds, dataset_scales, draw_fills, draw_lines, draw_points, drawn_lines, legend_items, Palette};
use crate::legend::{LegendItem, LegendMarker};
use crate::Fill;
use crate::mark::{MarkRole, ScatterCanvas, ScatterDataSet, SizeScale};
use crate::sequence::{IntoSequence, Sequence};
use crate::scale::{ScaleBounds, Scales};

/// Draws each dataset's points: points only by default. With `show_line(true)` it also draws a line
/// through them in order, following the dataset's `interpolation`, as a line chart does.
///
/// A dataset with a mark (see `ScatterDataSet`) draws the mark in place of its points, such as
/// bubbles (`SizedDataSetExt::bubbles`). Marks that draw sizes share one size scale across the
/// controller's datasets, set with `radius_range`.
///
/// Its x and y scales stay linear, so it suits data with numeric x values. For lists of values at
/// their indices, use a `LineController`.
#[derive(Clone, Debug)]
pub struct ScatterController<D> {
    dataset_sequence: D,
    show_line: bool,
    palette_offset: usize,
    min_radius: Scalar,
    max_radius: Scalar,
}

impl<D: Sequence<dyn ScatterDataSet>> ScatterController<D> {
    /// `datasets` is a sequence of datasets, or a single dataset.
    pub fn new<I: IntoSequence<dyn ScatterDataSet, M, Sequence=D>, M>(datasets: I) -> ScatterController<D> {
        ScatterController {
            dataset_sequence: datasets.into_sequence(),
            show_line: false,
            palette_offset: 0,
            min_radius: 4.0,
            max_radius: 30.0,
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

    /// The radii sizes are drawn at, in pixels, such as bubbles' (see `SizedDataSetExt::bubbles`):
    /// the largest size among the controller's datasets gets `max`, the others an area in
    /// proportion to it, and none is smaller than `min`. Defaults to 4 and 30.
    pub fn radius_range(mut self, min: Scalar, max: Scalar) -> ScatterController<D> {
        self.min_radius = min;
        self.max_radius = max;
        self
    }

    /// Also draws a line through each dataset's points, under the points. Defaults to `false`.
    pub fn show_line(mut self, show_line: bool) -> ScatterController<D> {
        self.show_line = show_line;
        self
    }
}

impl<D: Sequence<dyn ScatterDataSet>> ScatterController<D> {
    /// The indices of the datasets whose points a mark replaces.
    fn replaced_points(&self) -> Vec<usize> {
        let mut replaced = vec![];
        self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
            let mut primary = false;
            dataset.scatter_marks(&mut |mark| primary |= mark.role() == MarkRole::Primary);
            if primary {
                replaced.push(dataset_index);
            }
        });
        replaced
    }
}

impl<D: Sequence<dyn ScatterDataSet>> DatasetController for ScatterController<D> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.dataset_sequence.sync(env)
    }

    /// Reports each dataset's points, and what its marks draw. A dataset whose points a mark
    /// replaces reports only what the mark draws, as its other points aren't drawn, unless a line
    /// runs through all its points (`show_line`).
    fn collect_bounds(&self, bounds: &mut ScaleBounds, _env: &mut Environment) {
        let replaced = self.replaced_points();
        collect_point_bounds(&self.dataset_sequence, bounds, self.show_line, false, Orientation::Vertical, |dataset_index| {
            self.show_line || !replaced.contains(&dataset_index)
        });

        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            let options = dataset.options();
            dataset.scatter_marks(&mut |mark| {
                mark.bounds(&mut |x, y| {
                    bounds.include(&options.x_scale, x, x);
                    bounds.include(&options.y_scale, y, y);
                });
            });
        });
    }

    /// Each named dataset as a point in its colour, or, when lines are shown, a line, or a square
    /// when it is filled.
    fn legend_items(&self, env: &Environment, items: &mut Vec<LegendItem>) {
        legend_items(&self.dataset_sequence, &Palette::from_env(env, self.palette_offset), items, |options| {
            match (self.show_line, options.fill) {
                (false, _) => LegendMarker::Point,
                (true, Fill::None) => LegendMarker::Line,
                (true, _) => LegendMarker::Square,
            }
        });
    }

    fn draw(&self, ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
        ctx.save();

        // Scatter charts don't stack: each dataset's points are drawn at their own values.
        let lines = drawn_lines(&self.dataset_sequence, scales, chart_area, false, Orientation::Vertical);

        let palette = Palette::new(ctx, self.palette_offset);

        if self.show_line {
            draw_fills(&lines, ctx, scales, chart_area, &palette);
            draw_lines(&lines, ctx, chart_area, &palette);
        }

        // The points of datasets whose points a mark replaces are drawn by the mark instead.
        let replaced = self.replaced_points();
        draw_points(&lines, ctx, &palette, |dataset_index| !replaced.contains(&dataset_index));

        // One size scale for every dataset's marks, so their sizes compare.
        let mut max_size: Option<Scalar> = None;
        self.dataset_sequence.foreach(&mut |_dataset_index, dataset| {
            dataset.scatter_marks(&mut |mark| {
                mark.sizes(&mut |size| {
                    if size > 0.0 {
                        max_size = Some(max_size.map_or(size, |max| max.max(size)));
                    }
                });
            });
        });
        let sizes = SizeScale { min_radius: self.min_radius, max_radius: self.max_radius, max_size };

        // The marks replacing points, then the overlays on top of everything.
        for role in [MarkRole::Primary, MarkRole::Overlay] {
            self.dataset_sequence.foreach(&mut |dataset_index, dataset| {
                let options = dataset.options();
                let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
                    return;
                };

                let mut canvas = ScatterCanvas {
                    ctx: &mut *ctx,
                    x_scale,
                    y_scale,
                    area: chart_area,
                    options: &options,
                    dataset_index,
                    palette: &palette,
                    sizes,
                };

                dataset.scatter_marks(&mut |mark| {
                    if mark.role() == role {
                        mark.draw(&mut canvas);
                    }
                });
            });
        }

        ctx.restore();
    }
}
