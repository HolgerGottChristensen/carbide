//! The legend: each named dataset's colour and name, laid out on a side of the chart (see
//! `Chart::legend`). Controllers say what to show (`DatasetController::legend_items`): a dataset
//! is in the legend when it has a name (`DatasetExt::label`).

use carbide::color::Color;
use carbide::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use carbide_derive::StateValue;
use crate::style::axis_style;

/// The side of the chart the legend is drawn on, or `Hidden`, the default. On the top or bottom
/// the items wrap into centred rows across the chart; on the left or right they stack in a column,
/// centred down the chart.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, StateValue)]
pub enum LegendPosition {
    #[default]
    Hidden,
    Top,
    Bottom,
    Left,
    Right,
}

/// One entry of the legend: a dataset's name, and a marker in the colour it is drawn in.
#[derive(Debug, Clone, PartialEq)]
pub struct LegendItem {
    pub label: String,
    pub color: Color,
    pub marker: LegendMarker,
}

/// The shape of a legend entry's marker, after what its controller draws.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LegendMarker {
    /// A filled square, for bars and filled areas.
    Square,
    /// A short line with a point on it, for lines.
    Line,
    /// A point, for points without a line.
    Point,
}

/// The width and height of a marker.
const MARKER_SIZE: Scalar = 12.0;
/// The diameter of a point marker, the same as a chart's points.
const POINT_DIAMETER: Scalar = 7.0;
/// The gap between a marker and its name.
const MARKER_GAP: Scalar = 6.0;
/// The gap between two items side by side.
const ITEM_GAP: Scalar = 16.0;
/// The gap between two rows, or two items one above the other.
const ROW_GAP: Scalar = 4.0;

/// Where a legend's items go: each item's top-left corner, relative to the legend's top-left, its
/// height, and the legend's size.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LegendLayout {
    pub(crate) size: Dimension,
    items: Vec<(Position, Scalar)>,
}

/// Lays `items` out for a legend on `position`, measuring their names with `measure`. On the top
/// or bottom, items wrap into rows no wider than `max_width`, each row centred in it, and the
/// legend is `max_width` wide; on the left or right they stack in one column as wide as the widest
/// item.
pub(crate) fn layout(items: &[LegendItem], position: LegendPosition, max_width: Scalar, mut measure: impl FnMut(&str) -> Dimension) -> LegendLayout {
    let sizes: Vec<Dimension> = items.iter()
        .map(|item| {
            let text = measure(&item.label);
            Dimension::new(MARKER_SIZE + MARKER_GAP + text.width, text.height.max(MARKER_SIZE))
        })
        .collect();

    let mut placed = vec![];

    match position {
        LegendPosition::Hidden => {}
        LegendPosition::Left | LegendPosition::Right => {
            let mut y = 0.0;
            for (index, size) in sizes.iter().enumerate() {
                if index > 0 {
                    y += ROW_GAP;
                }
                placed.push((Position::new(0.0, y), size.height));
                y += size.height;
            }

            let width = sizes.iter().map(|size| size.width).fold(0.0, Scalar::max);
            return LegendLayout { size: Dimension::new(width, y), items: placed };
        }
        LegendPosition::Top | LegendPosition::Bottom => {
            // Greedily fill rows, then centre each one.
            let mut rows: Vec<Vec<usize>> = vec![];
            let mut row_width = 0.0;

            for (index, size) in sizes.iter().enumerate() {
                let width_with_item = if rows.last().is_some_and(|row| !row.is_empty()) { row_width + ITEM_GAP + size.width } else { size.width };

                if rows.is_empty() || (width_with_item > max_width && row_width > 0.0) {
                    rows.push(vec![index]);
                    row_width = size.width;
                } else {
                    rows.last_mut().expect("a row was pushed").push(index);
                    row_width = width_with_item;
                }
            }

            placed = vec![(Position::new(0.0, 0.0), 0.0); items.len()];
            let mut y = 0.0;

            for (row_index, row) in rows.iter().enumerate() {
                if row_index > 0 {
                    y += ROW_GAP;
                }

                let width: Scalar = row.iter().map(|index| sizes[*index].width).sum::<Scalar>() + ITEM_GAP * (row.len().saturating_sub(1)) as Scalar;
                let height = row.iter().map(|index| sizes[*index].height).fold(0.0, Scalar::max);

                let mut x = (max_width - width) / 2.0;
                for index in row {
                    placed[*index] = (Position::new(x, y), height);
                    x += sizes[*index].width + ITEM_GAP;
                }

                y += height;
            }

            return LegendLayout { size: Dimension::new(max_width, y), items: placed };
        }
    }

    LegendLayout { size: Dimension::new(0.0, 0.0), items: placed }
}

/// Draws the legend's items in `area`, as laid out by `layout`: a legend on the left or right is
/// centred down `area`. Names are drawn in the `AxisStyle`'s label colour.
pub(crate) fn draw(items: &[LegendItem], layout: &LegendLayout, position: LegendPosition, ctx: &mut CanvasContext, area: Rect) {
    // `Rect`'s `bottom()` is the screen's top edge (see `scale/axis.rs`).
    let top = match position {
        LegendPosition::Left | LegendPosition::Right => area.bottom() + (area.height() - layout.size.height).max(0.0) / 2.0,
        _ => area.bottom(),
    };

    let style = axis_style(ctx.env());
    let label_color = style.label_color(ctx.env());

    ctx.save();
    ctx.set_text_align(Alignment::Leading);

    for (item, (corner, height)) in items.iter().zip(&layout.items) {
        let x = area.left() + corner.x;
        let middle = top + corner.y + height / 2.0;

        ctx.begin_path();
        match item.marker {
            LegendMarker::Square => {
                let (top, bottom) = (middle - MARKER_SIZE / 2.0, middle + MARKER_SIZE / 2.0);
                ctx.move_to(x, top);
                ctx.line_to(x + MARKER_SIZE, top);
                ctx.line_to(x + MARKER_SIZE, bottom);
                ctx.line_to(x, bottom);
                ctx.close_path();
                ctx.set_fill_style(item.color);
                ctx.fill();
            }
            LegendMarker::Line => {
                ctx.move_to(x, middle);
                ctx.line_to(x + MARKER_SIZE, middle);
                ctx.set_line_width(2.0);
                ctx.set_stroke_style(item.color);
                ctx.stroke();
                ctx.begin_path();
                ctx.circle(x + MARKER_SIZE / 2.0, middle, POINT_DIAMETER);
                ctx.set_fill_style(item.color);
                ctx.fill();
            }
            LegendMarker::Point => {
                ctx.circle(x + MARKER_SIZE / 2.0, middle, POINT_DIAMETER);
                ctx.set_fill_style(item.color);
                ctx.fill();
            }
        }

        ctx.set_fill_style(label_color);
        ctx.fill_text(&item.label, x + MARKER_SIZE + MARKER_GAP, middle);
    }

    ctx.restore();
}

#[cfg(test)]
mod tests {
    use carbide::color::RED;
    use carbide::draw::{Dimension, Position};
    use super::{layout, LegendItem, LegendMarker, LegendPosition, ITEM_GAP, MARKER_GAP, MARKER_SIZE, ROW_GAP};

    fn items(labels: &[&str]) -> Vec<LegendItem> {
        labels.iter().map(|label| LegendItem { label: label.to_string(), color: RED, marker: LegendMarker::Square }).collect()
    }

    /// Text 10px per character and 10px tall, so sizes are easy to follow.
    fn measure(text: &str) -> Dimension {
        Dimension::new(10.0 * text.len() as f64, 10.0)
    }

    /// On the top or bottom, items fill a row and wrap when the next one wouldn't fit, and each row
    /// is centred across the legend's width.
    #[test]
    fn top_legends_wrap_into_centred_rows() {
        // Each item is 12 + 6 + 40 = 58 wide; two fit in 130 (58 + 16 + 58 = 132 doesn't).
        let laid_out = layout(&items(&["abcd", "efgh", "ijkl"]), LegendPosition::Top, 130.0, measure);

        let item_width = MARKER_SIZE + MARKER_GAP + 40.0;
        assert!(item_width + ITEM_GAP + item_width > 130.0);
        assert_eq!(laid_out.items[0].0, Position::new((130.0 - item_width) / 2.0, 0.0), "a row of one is centred");
        assert_eq!(laid_out.items[1].0.y, MARKER_SIZE + ROW_GAP, "the second item wraps onto the next row");
        assert_eq!(laid_out.size, Dimension::new(130.0, 3.0 * MARKER_SIZE + 2.0 * ROW_GAP));

        let wide = layout(&items(&["abcd", "efgh"]), LegendPosition::Bottom, 300.0, measure);
        assert_eq!(wide.items[1].0.y, 0.0, "with room, both sit on one row");
        assert_eq!(wide.items[0].0.x, (300.0 - (2.0 * item_width + ITEM_GAP)) / 2.0, "and the row is centred");
    }

    /// On the left or right, items stack in one column as wide as the widest item.
    #[test]
    fn side_legends_stack_in_a_column() {
        let laid_out = layout(&items(&["ab", "abcdef"]), LegendPosition::Right, 1000.0, measure);

        assert_eq!(laid_out.items[0].0, Position::new(0.0, 0.0));
        assert_eq!(laid_out.items[1].0, Position::new(0.0, MARKER_SIZE + ROW_GAP));
        assert_eq!(laid_out.size, Dimension::new(MARKER_SIZE + MARKER_GAP + 60.0, 2.0 * MARKER_SIZE + ROW_GAP));

        assert_eq!(layout(&items(&["ab"]), LegendPosition::Hidden, 1000.0, measure).size, Dimension::new(0.0, 0.0));
    }
}
