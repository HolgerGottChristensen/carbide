//! The tooltip: a box next to the hovered data, naming what it is and its values (see
//! `Chart::tooltip`). Its content comes from the data's descriptions (`Pick::items`); it is placed
//! next to the data, on the first side it fits on, and drawn on the chart's canvas, above
//! everything else.

use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use carbide::color::Color;
use carbide::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide_derive::StateValue;
use carbide_core::text::FontWeight;
use carbide_core::widget::canvas::CanvasContext;
use crate::geometry::PickMode;
use crate::interaction::{DatumInfo, Pick};
use crate::legend::{draw_marker, LegendMarker, MARKER_GAP, MARKER_SIZE};
use crate::style::tooltip_style;

/// The space between the tooltip's edge and its text.
const PADDING: Scalar = 8.0;
/// The space between two rows.
const ROW_GAP: Scalar = 4.0;
/// The space between the title and the first row.
const TITLE_GAP: Scalar = 6.0;
/// How far the caret reaches out from the tooltip, and half its width.
const CARET: Scalar = 6.0;
/// The space between the caret's tip and the data it points at.
const ANCHOR_GAP: Scalar = 2.0;
/// The least space kept between the tooltip and the chart's edges.
const MARGIN: Scalar = 4.0;

/// A tooltip's title text, from what the pointer found (see `Chart::tooltip_title`).
#[derive(Clone)]
pub(crate) struct TooltipTitle(pub(crate) Rc<dyn Fn(&Pick) -> String>);

impl Debug for TooltipTitle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TooltipTitle")
    }
}

/// A tooltip row's text, from the datum it shows (see `Chart::tooltip_text`).
#[derive(Clone)]
pub(crate) struct TooltipText(pub(crate) Rc<dyn Fn(&DatumInfo) -> String>);

impl Debug for TooltipText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TooltipText")
    }
}

/// What a tooltip shows: a title, and a row per datum, each with its dataset's marker in its
/// colour, or none for the row saying how many more there are.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TooltipContent {
    pub(crate) title: Option<String>,
    pub(crate) rows: Vec<TooltipRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TooltipRow {
    pub(crate) marker: Option<(LegendMarker, Color)>,
    pub(crate) text: String,
}

/// The tooltip for `pick`, found with `mode`, or `None` when it found nothing to describe. The
/// title is `title`'s text, or else the first datum's index value, its category, or in
/// `PickMode::Dataset` its dataset's name. Each datum is a row, `text`'s text or else
/// `DatumInfo::row_text`, or in `PickMode::Dataset`, where the title names the dataset, its index
/// and its values; there, at most `max_rows` of them, then a row saying how many more there are.
pub(crate) fn content(pick: &Pick, mode: PickMode, max_rows: usize, title: Option<&TooltipTitle>, text: Option<&TooltipText>) -> Option<TooltipContent> {
    let first = pick.items.first()?;

    let title = match title {
        Some(title) => Some((title.0)(pick)),
        None if mode == PickMode::Dataset => first.dataset.clone(),
        None => first.index.clone(),
    }.filter(|title| !title.is_empty());

    let shown = if mode == PickMode::Dataset { max_rows.max(1).min(pick.items.len()) } else { pick.items.len() };
    let mut rows: Vec<TooltipRow> = pick.items[..shown].iter()
        .map(|item| TooltipRow {
            marker: Some((item.marker, item.color)),
            text: match text {
                Some(text) => (text.0)(item),
                // The title names the dataset, so each row names its datum's index instead.
                None if mode == PickMode::Dataset => match &item.index {
                    Some(index) => format!("{index}: {}", item.values_text()),
                    None => item.values_text(),
                },
                None => item.row_text(),
            },
        })
        .collect();

    let more = pick.items.len() - shown;
    if more > 0 {
        rows.push(TooltipRow { marker: None, text: format!("and {more} more") });
    }

    Some(TooltipContent { title, rows })
}

/// The side of the tooltip its caret is on, pointing at the data.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Caret {
    Top,
    Bottom,
    Left,
    Right,
}

/// Where a tooltip goes: its top-left corner, and the side of its caret, or `None` when no side
/// fits and it was moved inside the chart.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Placement {
    pub(crate) origin: Position,
    pub(crate) caret: Option<Caret>,
}

/// What a tooltip points at (see `Chart::tooltip_anchor`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, StateValue)]
pub enum TooltipAnchor {
    /// The hovered data: the average of their shapes' middles. The default.
    #[default]
    Data,
    /// The pointer, following it as it moves over the chart area.
    Pointer,
}

/// Places a tooltip of `size` next to `anchor`, inside `bounds` (from `bounds.position`, `y`
/// growing downwards, as the canvas draws), preferring `alignment`'s side of the anchor (see
/// `Chart::tooltip_alignment`). It tries `alignment`, then its mirror, then the sides in turn
/// (above, below, after, before), taking the first that fits; a side slides along to stay inside,
/// keeping its caret pointing at `anchor`. When none fits, `alignment`'s placement is moved
/// inside, without a caret.
pub(crate) fn place(size: Dimension, anchor: Position, bounds: Rect, alignment: Alignment) -> Placement {
    let area = Inside {
        left: bounds.position.x + MARGIN,
        top: bounds.position.y + MARGIN,
        right: bounds.position.x + bounds.dimension.width - MARGIN,
        bottom: bounds.position.y + bounds.dimension.height - MARGIN,
    };

    let mut tried = vec![alignment, mirror(alignment), Alignment::Top, Alignment::Bottom, Alignment::Trailing, Alignment::Leading];
    let mut seen = vec![];
    tried.retain(|alignment| {
        let new = !seen.contains(alignment);
        seen.push(*alignment);
        new
    });

    for alignment in tried {
        if let Some(placement) = candidate(size, anchor, &area, alignment) {
            return placement;
        }
    }

    let origin = origin_for(size, anchor, alignment);
    Placement { origin: Position::new(area.slide_x(origin.x, size.width), area.slide_y(origin.y, size.height)), caret: None }
}

/// Where a tooltip may go: its edges' limits.
struct Inside {
    left: Scalar,
    top: Scalar,
    right: Scalar,
    bottom: Scalar,
}

impl Inside {
    /// The left edge that keeps a tooltip `width` wide inside, moving `x` as little as it can.
    fn slide_x(&self, x: Scalar, width: Scalar) -> Scalar {
        x.min(self.right - width).max(self.left)
    }

    /// The top edge that keeps a tooltip `height` tall inside, moving `y` as little as it can.
    fn slide_y(&self, y: Scalar, height: Scalar) -> Scalar {
        y.min(self.bottom - height).max(self.top)
    }

    /// Whether a tooltip at `origin` of `size` lies inside.
    fn contains(&self, origin: Position, size: Dimension) -> bool {
        origin.x >= self.left - 1e-9 && origin.x + size.width <= self.right + 1e-9 && origin.y >= self.top - 1e-9 && origin.y + size.height <= self.bottom + 1e-9
    }
}

/// `alignment` mirrored: top and bottom swapped, and leading and trailing.
fn mirror(alignment: Alignment) -> Alignment {
    match alignment {
        Alignment::TopLeading => Alignment::BottomTrailing,
        Alignment::Top => Alignment::Bottom,
        Alignment::TopTrailing => Alignment::BottomLeading,
        Alignment::Leading => Alignment::Trailing,
        Alignment::Center => Alignment::Center,
        Alignment::Trailing => Alignment::Leading,
        Alignment::BottomLeading => Alignment::TopTrailing,
        Alignment::Bottom => Alignment::Top,
        Alignment::BottomTrailing => Alignment::TopLeading,
        Alignment::Custom(x, y) => Alignment::Custom(1.0 - x, 1.0 - y),
    }
}

/// Where a tooltip of `size` goes for `alignment`, before it is kept inside: on a side, centred
/// along it, a caret's reach from `anchor`; on a corner, its opposite corner that far away; centred
/// on `anchor`; or for `Custom(x, y)`, its point at `x` and `y` of its size on `anchor`, moved that
/// far away from it, the way `anchor` lies from its middle.
fn origin_for(size: Dimension, anchor: Position, alignment: Alignment) -> Position {
    let (width, height) = (size.width, size.height);
    let reach = CARET + ANCHOR_GAP;
    let (x, y) = (anchor.x, anchor.y);

    match alignment {
        Alignment::Top => Position::new(x - width / 2.0, y - reach - height),
        Alignment::Bottom => Position::new(x - width / 2.0, y + reach),
        Alignment::Leading => Position::new(x - reach - width, y - height / 2.0),
        Alignment::Trailing => Position::new(x + reach, y - height / 2.0),
        Alignment::TopLeading => Position::new(x - reach - width, y - reach - height),
        Alignment::TopTrailing => Position::new(x + reach, y - reach - height),
        Alignment::BottomLeading => Position::new(x - reach - width, y + reach),
        Alignment::BottomTrailing => Position::new(x + reach, y + reach),
        Alignment::Center => Position::new(x - width / 2.0, y - height / 2.0),
        Alignment::Custom(fraction_x, fraction_y) => {
            let (away_x, away_y) = (fraction_x - 0.5, fraction_y - 0.5);
            let length = away_x.hypot(away_y);
            let (shift_x, shift_y) = if length > 1e-9 { (away_x / length * reach, away_y / length * reach) } else { (0.0, 0.0) };
            // Away from the anchor: the anchor lies that way from the tooltip's middle.
            Position::new(x - fraction_x * width - shift_x, y - fraction_y * height - shift_y)
        }
    }
}

/// The placement for `alignment` when it fits `area`, or `None`. A side fits when the tooltip fits
/// across it, sliding along it, with a caret; a corner or `Custom` when it fits where it is,
/// without one; `Center` always, sliding inside, without one.
fn candidate(size: Dimension, anchor: Position, area: &Inside, alignment: Alignment) -> Option<Placement> {
    let origin = origin_for(size, anchor, alignment);
    let (width, height) = (size.width, size.height);

    let (origin, caret) = match alignment {
        Alignment::Top | Alignment::Bottom => {
            let caret = if alignment == Alignment::Top { Caret::Bottom } else { Caret::Top };
            (Position::new(area.slide_x(origin.x, width), origin.y), Some(caret))
        }
        Alignment::Leading | Alignment::Trailing => {
            let caret = if alignment == Alignment::Leading { Caret::Right } else { Caret::Left };
            (Position::new(origin.x, area.slide_y(origin.y, height)), Some(caret))
        }
        Alignment::Center => (Position::new(area.slide_x(origin.x, width), area.slide_y(origin.y, height)), None),
        _ => (origin, None),
    };

    area.contains(origin, size).then_some(Placement { origin, caret })
}

/// Draws `content` next to `anchor`, inside `bounds`, preferring `alignment`'s side of it (see
/// `place`), in the chart's `TooltipStyle`: a rounded box with a caret pointing at `anchor` when it
/// has one, the title in bold, then each row's marker and text.
pub(crate) fn draw(ctx: &mut CanvasContext, content: &TooltipContent, anchor: Position, bounds: Rect, alignment: Alignment) {
    let style = tooltip_style(ctx.env());
    let (background, border) = (style.background_color(ctx.env()), style.border_color(ctx.env()));
    let (title_color, text_color) = (style.title_color(ctx.env()), style.text_color(ctx.env()));
    let radius = style.corner_radius();

    ctx.save();

    // Sizes: the title in bold, the rows as they are, a marker and its gap before each.
    let title_size = content.title.as_ref().map(|title| {
        ctx.save();
        ctx.set_font_weight(FontWeight::Bold);
        let size = ctx.measure_text(title);
        ctx.restore();
        size
    });
    let row_sizes: Vec<Dimension> = content.rows.iter().map(|row| ctx.measure_text(&row.text)).collect();
    let row_height = |size: &Dimension| size.height.max(MARKER_SIZE);

    let marker_width = MARKER_SIZE + MARKER_GAP;
    let content_width = row_sizes.iter().zip(&content.rows)
        .map(|(size, row)| size.width + if row.marker.is_some() { marker_width } else { 0.0 })
        .chain(title_size.map(|size| size.width))
        .fold(0.0, Scalar::max);
    let rows_height: Scalar = row_sizes.iter().map(row_height).sum::<Scalar>() + ROW_GAP * row_sizes.len().saturating_sub(1) as Scalar;
    let title_height = title_size.map_or(0.0, |size| size.height + if row_sizes.is_empty() { 0.0 } else { TITLE_GAP });
    let size = Dimension::new(content_width + 2.0 * PADDING, title_height + rows_height + 2.0 * PADDING);

    let placement = place(size, anchor, bounds, alignment);
    let origin = placement.origin;

    trace_box(ctx, origin, size, radius, placement.caret, anchor);
    ctx.set_fill_style(background);
    ctx.fill();
    ctx.set_line_width(1.0);
    ctx.set_stroke_style(border);
    ctx.stroke();

    ctx.set_text_align(Alignment::Leading);
    let x = origin.x + PADDING;
    let mut y = origin.y + PADDING;

    if let (Some(title), Some(title_size)) = (&content.title, title_size) {
        ctx.save();
        ctx.set_font_weight(FontWeight::Bold);
        ctx.set_fill_style(title_color);
        ctx.fill_text(title, x, y + title_size.height / 2.0);
        ctx.restore();
        y += title_height;
    }

    for (row, size) in content.rows.iter().zip(&row_sizes) {
        let height = row_height(size);
        let middle = y + height / 2.0;
        let text_x = match row.marker {
            Some((marker, color)) => {
                draw_marker(ctx, marker, color, None, x, middle);
                x + marker_width
            }
            None => x,
        };
        ctx.set_fill_style(text_color);
        ctx.fill_text(&row.text, text_x, middle);
        y += height + ROW_GAP;
    }

    ctx.restore();
}

/// Traces the tooltip's outline: a box at `origin` of `size` with corners of `radius`, and, on the
/// side `caret`, a caret pointing at `anchor`, kept clear of the corners.
fn trace_box(ctx: &mut CanvasContext, origin: Position, size: Dimension, radius: Scalar, caret: Option<Caret>, anchor: Position) {
    let (x0, y0) = (origin.x, origin.y);
    let (x1, y1) = (origin.x + size.width, origin.y + size.height);
    let radius = radius.min(size.width / 2.0).min(size.height / 2.0).max(0.0);
    let caret_x = anchor.x.clamp(x0 + radius + CARET, (x1 - radius - CARET).max(x0 + radius + CARET));
    let caret_y = anchor.y.clamp(y0 + radius + CARET, (y1 - radius - CARET).max(y0 + radius + CARET));

    ctx.begin_path();
    ctx.move_to(x0 + radius, y0);
    if caret == Some(Caret::Top) {
        ctx.line_to(caret_x - CARET, y0);
        ctx.line_to(caret_x, y0 - CARET);
        ctx.line_to(caret_x + CARET, y0);
    }
    ctx.line_to(x1 - radius, y0);
    ctx.quadratic_curve_to(Position::new(x1, y0), Position::new(x1, y0 + radius));
    if caret == Some(Caret::Right) {
        ctx.line_to(x1, caret_y - CARET);
        ctx.line_to(x1 + CARET, caret_y);
        ctx.line_to(x1, caret_y + CARET);
    }
    ctx.line_to(x1, y1 - radius);
    ctx.quadratic_curve_to(Position::new(x1, y1), Position::new(x1 - radius, y1));
    if caret == Some(Caret::Bottom) {
        ctx.line_to(caret_x + CARET, y1);
        ctx.line_to(caret_x, y1 + CARET);
        ctx.line_to(caret_x - CARET, y1);
    }
    ctx.line_to(x0 + radius, y1);
    ctx.quadratic_curve_to(Position::new(x0, y1), Position::new(x0, y1 - radius));
    if caret == Some(Caret::Left) {
        ctx.line_to(x0, caret_y + CARET);
        ctx.line_to(x0 - CARET, caret_y);
        ctx.line_to(x0, caret_y - CARET);
    }
    ctx.line_to(x0, y0 + radius);
    ctx.quadratic_curve_to(Position::new(x0, y0), Position::new(x0 + radius, y0));
    ctx.close_path();
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Alignment, Dimension, Position, Rect};
    use super::{place, Caret, Placement, ANCHOR_GAP, CARET, MARGIN};

    fn bounds() -> Rect {
        Rect::new(Position::new(0.0, 0.0), Dimension::new(600.0, 400.0))
    }

    /// A tooltip goes above the data, centred on it, when it fits there, its caret on its bottom.
    #[test]
    fn above_when_it_fits() {
        let placement = place(Dimension::new(100.0, 50.0), Position::new(300.0, 200.0), bounds(), Alignment::Top);
        let reach = CARET + ANCHOR_GAP;
        assert_eq!(placement, Placement { origin: Position::new(250.0, 200.0 - reach - 50.0), caret: Some(Caret::Bottom) });
    }

    /// Near the top it flips below the data; near the left edge it slides right to stay inside,
    /// keeping its caret.
    #[test]
    fn below_near_the_top_and_sliding_along() {
        let placement = place(Dimension::new(100.0, 50.0), Position::new(20.0, 30.0), bounds(), Alignment::Top);
        assert_eq!(placement.caret, Some(Caret::Top));
        assert_eq!(placement.origin.x, MARGIN, "slid right, inside the margin");
        assert_eq!(placement.origin.y, 30.0 + CARET + ANCHOR_GAP);
    }

    /// Too tall to go above or below, it goes beside the data: to the right, or to the left near
    /// the right edge.
    #[test]
    fn beside_when_too_tall() {
        let tall = Dimension::new(100.0, 300.0);
        assert_eq!(place(tall, Position::new(200.0, 200.0), bounds(), Alignment::Top).caret, Some(Caret::Left), "to the right, its caret on its left");
        assert_eq!(place(tall, Position::new(550.0, 200.0), bounds(), Alignment::Top).caret, Some(Caret::Right), "to the left");
    }

    /// Each side and corner, where it fits, goes on that side of the anchor: the sides with a caret
    /// pointing back at it, the corners diagonally from it without one.
    #[test]
    fn each_side_and_corner_where_it_fits() {
        let (size, anchor) = (Dimension::new(100.0, 50.0), Position::new(300.0, 200.0));
        let reach = CARET + ANCHOR_GAP;
        let cases = [
            (Alignment::Bottom, Position::new(250.0, 200.0 + reach), Some(Caret::Top)),
            (Alignment::Leading, Position::new(300.0 - reach - 100.0, 175.0), Some(Caret::Right)),
            (Alignment::Trailing, Position::new(300.0 + reach, 175.0), Some(Caret::Left)),
            (Alignment::TopLeading, Position::new(300.0 - reach - 100.0, 200.0 - reach - 50.0), None),
            (Alignment::TopTrailing, Position::new(300.0 + reach, 200.0 - reach - 50.0), None),
            (Alignment::BottomLeading, Position::new(300.0 - reach - 100.0, 200.0 + reach), None),
            (Alignment::BottomTrailing, Position::new(300.0 + reach, 200.0 + reach), None),
        ];
        for (alignment, origin, caret) in cases {
            assert_eq!(place(size, anchor, bounds(), alignment), Placement { origin, caret }, "{alignment:?}");
        }
    }

    /// Near an edge a corner falls back to its mirror: `TopLeading` near the top left goes below and
    /// after the anchor; a side does too, `Leading` near the left edge going after it.
    #[test]
    fn falling_back_to_the_mirror() {
        let size = Dimension::new(100.0, 50.0);
        let reach = CARET + ANCHOR_GAP;
        assert_eq!(place(size, Position::new(30.0, 30.0), bounds(), Alignment::TopLeading), Placement { origin: Position::new(30.0 + reach, 30.0 + reach), caret: None });
        assert_eq!(place(size, Position::new(30.0, 200.0), bounds(), Alignment::Leading).caret, Some(Caret::Left), "after the anchor");
    }

    /// `Center` centres the tooltip on the anchor, sliding inside, without a caret; `Custom` puts
    /// its point at the fractions on the anchor, moved a little away from it, without one.
    #[test]
    fn centre_and_custom() {
        let size = Dimension::new(100.0, 50.0);
        assert_eq!(place(size, Position::new(300.0, 200.0), bounds(), Alignment::Center), Placement { origin: Position::new(250.0, 175.0), caret: None });
        assert_eq!(place(size, Position::new(10.0, 10.0), bounds(), Alignment::Center).origin, Position::new(MARGIN, MARGIN), "slid inside");

        let reach = CARET + ANCHOR_GAP;
        let custom = place(size, Position::new(300.0, 200.0), bounds(), Alignment::Custom(0.5, 1.0));
        assert_eq!(custom, Placement { origin: Position::new(250.0, 200.0 - 50.0 - reach), caret: None }, "its bottom middle above the anchor");
    }

    /// When no side fits, it is moved inside the chart, without a caret.
    #[test]
    fn clamped_without_a_caret_when_nothing_fits() {
        let placement = place(Dimension::new(580.0, 380.0), Position::new(300.0, 200.0), bounds(), Alignment::Top);
        assert_eq!(placement.caret, None);
        assert!(placement.origin.x >= MARGIN && placement.origin.y >= MARGIN, "{placement:?}");
    }
}
