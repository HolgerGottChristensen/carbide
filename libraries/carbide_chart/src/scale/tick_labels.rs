//! A scale's tick labels for one render: their texts and sizes, how far they are turned, the point
//! of each that sits at the end of its tick mark, and which are shown. Built from the scale's
//! label options (`AxisOptions::label_rotation`, `label_alignment` and `label_skip`), fitted to
//! the chart area in two steps by the chart's render: turned when they don't fit flat, before the
//! chart lays out again with the turned bands, and thinned out once the layout is final.

use std::f64::consts::PI;
use carbide::draw::{Alignment, Dimension, Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::scale::{Axis, AxisPosition, LabelRotation};
use crate::scale::axis::tick_label;
use crate::Scale;

/// The space kept between two neighbouring labels, along the axis.
const LABEL_GAP: Scalar = 4.0;

/// How close, in degrees, the way to the axis must be to a label's own up or down for the label
/// to hang centred off its tick, rather than from its end.
const CENTRED_WITHIN: Scalar = 10.0;

/// A scale's tick labels, as measured, turned, aligned and thinned out for one render.
#[derive(Clone, Debug)]
pub(crate) struct TickLabels {
    labels: Vec<TickLabel>,
    /// How far every label is turned, in radians, the way the canvas turns: positive clockwise
    /// on screen.
    rotation: Scalar,
    /// The point of each label at the end of its tick mark, in the label's own frame.
    alignment: Alignment,
}

/// One tick's label: the tick's value, its text and the text's size, and whether it is drawn.
#[derive(Clone, Debug)]
pub(crate) struct TickLabel {
    pub(crate) value: Scalar,
    pub(crate) text: String,
    pub(crate) size: Dimension,
    pub(crate) shown: bool,
}

impl TickLabels {
    /// The labels of `scale`'s ticks, measured with the canvas's text style, all shown, turned
    /// by a fixed rotation or flat, before they are fitted. A polar scale has none here: its
    /// labels go around the circle.
    pub(crate) fn new(scale: &dyn Scale, ctx: &mut CanvasContext) -> TickLabels {
        if scale.axis() == Axis::Radial {
            return TickLabels { labels: vec![], rotation: 0.0, alignment: Alignment::Center };
        }

        let labels = scale.ticks().iter()
            .map(|value| {
                let text = tick_label(scale, *value, ctx.env());
                let size = ctx.measure_text(&text);
                TickLabel { value: *value, text, size, shown: true }
            })
            .collect();

        let rotation = match scale.axis_options().label_rotation() {
            LabelRotation::Fixed(angle) => angle.radians(),
            LabelRotation::Auto { .. } => 0.0,
        };

        TickLabels::with_labels(labels, scale.axis(), scale.axis_options().position(), rotation, scale.axis_options().label_alignment())
    }

    /// Labels turned by `rotation`, aligned by `alignment`, or, when that is `None`, so they point
    /// at their ticks on `side` (see `automatic_alignment`).
    fn with_labels(labels: Vec<TickLabel>, axis: Axis, side: AxisPosition, rotation: Scalar, alignment: Option<Alignment>) -> TickLabels {
        let alignment = alignment.unwrap_or_else(|| automatic_alignment(inward(axis, side), rotation));
        TickLabels { labels, rotation, alignment }
    }

    pub(crate) fn labels(&self) -> &[TickLabel] {
        &self.labels
    }

    pub(crate) fn rotation(&self) -> Scalar {
        self.rotation
    }

    pub(crate) fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// How far the labels reach out from the end of the tick marks, away from the chart area: the
    /// furthest corner of any label, as turned around its anchor.
    pub(crate) fn depth(&self, axis: Axis, side: AxisPosition) -> Scalar {
        let outward = inward(axis, side) * -1.0;
        let (along_x, along_y) = alignment_fractions(self.alignment);

        self.labels.iter()
            .map(|label| {
                let (width, height) = (label.size.width, label.size.height);
                let corners = [
                    Position::new(-along_x * width, -along_y * height),
                    Position::new((1.0 - along_x) * width, -along_y * height),
                    Position::new(-along_x * width, (1.0 - along_y) * height),
                    Position::new((1.0 - along_x) * width, (1.0 - along_y) * height),
                ];

                corners.into_iter()
                    .map(|corner| {
                        let turned = turn(corner, self.rotation);
                        turned.x * outward.x + turned.y * outward.y
                    })
                    .fold(0.0, Scalar::max)
            })
            .fold(0.0, Scalar::max)
    }

    /// Turns automatically rotated labels on a horizontal axis by their maximum when they don't
    /// fit flat along `area`, and returns whether they turned, so the chart lays out again with
    /// the deeper band.
    pub(crate) fn turn_to_fit(&mut self, scale: &dyn Scale, area: Rect) -> bool {
        let options = scale.axis_options();
        let LabelRotation::Auto { max } = options.label_rotation() else {
            return false;
        };

        if scale.axis() != Axis::Horizontal || self.fit_step(scale, area) == 1 {
            return false;
        }

        let turned = TickLabels::with_labels(std::mem::take(&mut self.labels), scale.axis(), options.position(), max.radians(), options.label_alignment());
        *self = turned;
        true
    }

    /// Shows every n-th label, from the first, for the smallest n that keeps the shown labels
    /// clear of each other along `area`, when the scale's labels are thinned out (see
    /// `AxisOptions::label_skip`).
    pub(crate) fn skip_to_fit(&mut self, scale: &dyn Scale, area: Rect) {
        let step = if scale.axis_options().label_skip() { self.fit_step(scale, area) } else { 1 };

        for (index, label) in self.labels.iter_mut().enumerate() {
            label.shown = index % step == 0;
        }
    }

    /// The smallest n for which every n-th label, from the first, is clear of the next shown one.
    fn fit_step(&self, scale: &dyn Scale, area: Rect) -> usize {
        let positions: Vec<Scalar> = self.labels.iter().map(|label| along(scale, area, label.value)).collect();
        let horizontal = scale.axis() == Axis::Horizontal;

        (1..self.labels.len().max(1))
            .find(|step| {
                (0..self.labels.len()).step_by(*step)
                    .zip((*step..self.labels.len()).step_by(*step))
                    .all(|(first, second)| {
                        let room = (positions[second] - positions[first]).abs();
                        room >= spacing(self.labels[first].size, self.labels[second].size, self.rotation, horizontal)
                    })
            })
            .unwrap_or(self.labels.len().max(1))
    }
}

/// The distance two neighbouring labels need between their anchors along the axis, turned by
/// `rotation`, so they don't overlap: the thickness of the larger of the two across their own
/// direction, measured along the axis, plus the gap. Flat labels need their width along a
/// horizontal axis and their height along a vertical one; turned labels lie side by side, and need
/// their height over the sine of the angle, until the turned width is the narrower.
fn spacing(first: Dimension, second: Dimension, rotation: Scalar, horizontal: bool) -> Scalar {
    let width = first.width.max(second.width) + LABEL_GAP;
    let height = first.height.max(second.height) + LABEL_GAP;
    let cos = rotation.cos().abs();
    let sin = rotation.sin().abs();

    if horizontal {
        if height * cos > width * sin { width / cos } else { height / sin }
    } else if height * sin < width * cos {
        height / cos
    } else {
        width / sin
    }
}

/// The alignment that makes a label turned by `rotation` point at its tick, `inward` being the
/// unit direction from the label towards the axis on screen: that direction, in the label's own
/// frame, names the edge of the label facing the axis. A label hangs centred off that edge when
/// the axis is within `CENTRED_WITHIN` degrees of the label's up or down, and from the middle of
/// its start or end otherwise, so a turned label's end sits at the tick.
fn automatic_alignment(inward: Position, rotation: Scalar) -> Alignment {
    let towards = turn(inward, -rotation);

    if towards.x.abs() <= (CENTRED_WITHIN * PI / 180.0).sin() {
        if towards.y < 0.0 { Alignment::Top } else { Alignment::Bottom }
    } else if towards.x > 0.0 {
        Alignment::Trailing
    } else {
        Alignment::Leading
    }
}

/// The unit direction on screen from an axis's labels towards the chart area: up for labels below
/// the area, right for labels left of it, and so on.
fn inward(axis: Axis, side: AxisPosition) -> Position {
    match (axis, side) {
        (Axis::Horizontal, AxisPosition::Top) => Position::new(0.0, 1.0),
        (Axis::Horizontal, _) => Position::new(0.0, -1.0),
        (Axis::Vertical, AxisPosition::Right) => Position::new(-1.0, 0.0),
        (Axis::Vertical, _) => Position::new(1.0, 0.0),
        (Axis::Radial, _) => Position::new(0.0, 0.0),
    }
}

/// `point` turned by `rotation` radians around the origin, as the canvas turns: clockwise on the
/// screen, whose y grows downwards.
fn turn(point: Position, rotation: Scalar) -> Position {
    let (sin, cos) = rotation.sin_cos();
    Position::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos)
}

/// Where `value` is along the axis of `scale`, on the screen, within `area`.
pub(crate) fn along(scale: &dyn Scale, area: Rect, value: Scalar) -> Scalar {
    let fraction = scale.normalize(value);

    if scale.axis() == Axis::Horizontal {
        area.left() + fraction * area.width()
    } else {
        area.bottom() + (1.0 - fraction) * area.height()
    }
}

/// The point of a text box an alignment names, as fractions of its width and height from its
/// top-left corner, as the canvas places text (`CanvasContext::fill_text`).
pub(crate) fn alignment_fractions(alignment: Alignment) -> (Scalar, Scalar) {
    match alignment {
        Alignment::TopLeading => (0.0, 0.0),
        Alignment::Top => (0.5, 0.0),
        Alignment::TopTrailing => (1.0, 0.0),
        Alignment::Leading => (0.0, 0.5),
        Alignment::Center => (0.5, 0.5),
        Alignment::Trailing => (1.0, 0.5),
        Alignment::BottomLeading => (0.0, 1.0),
        Alignment::Bottom => (0.5, 1.0),
        Alignment::BottomTrailing => (1.0, 1.0),
        Alignment::Custom(x, y) => (x, y),
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Alignment, Dimension, Scalar};
    use crate::scale::{Axis, AxisPosition};
    use super::{automatic_alignment, inward, spacing, TickLabel, TickLabels};

    /// Flat labels keep the alignment each side always had: centred under a bottom axis, above a
    /// top one, and level with their tick beside a vertical one, ending at it on the left.
    #[test]
    fn flat_labels_hang_off_their_side() {
        let flat = |axis, side| automatic_alignment(inward(axis, side), 0.0);

        assert_eq!(flat(Axis::Horizontal, AxisPosition::Bottom), Alignment::Top);
        assert_eq!(flat(Axis::Horizontal, AxisPosition::Top), Alignment::Bottom);
        assert_eq!(flat(Axis::Vertical, AxisPosition::Left), Alignment::Trailing);
        assert_eq!(flat(Axis::Vertical, AxisPosition::Right), Alignment::Leading);
    }

    /// Turned labels point at their ticks: below a bottom axis, a label rising to the right ends
    /// at its tick and one falling to the right starts at it, mirrored above a top axis; labels
    /// turned upright beside a vertical axis face it with their bottom edge, and a few degrees
    /// stay centred.
    #[test]
    fn turned_labels_point_at_their_ticks() {
        let turned = |axis, side, degrees: Scalar| automatic_alignment(inward(axis, side), degrees.to_radians());

        assert_eq!(turned(Axis::Horizontal, AxisPosition::Bottom, -45.0), Alignment::Trailing);
        assert_eq!(turned(Axis::Horizontal, AxisPosition::Bottom, 45.0), Alignment::Leading);
        assert_eq!(turned(Axis::Horizontal, AxisPosition::Bottom, -90.0), Alignment::Trailing);
        assert_eq!(turned(Axis::Horizontal, AxisPosition::Bottom, -5.0), Alignment::Top);
        assert_eq!(turned(Axis::Horizontal, AxisPosition::Top, -45.0), Alignment::Leading);
        assert_eq!(turned(Axis::Horizontal, AxisPosition::Top, 45.0), Alignment::Trailing);
        assert_eq!(turned(Axis::Vertical, AxisPosition::Left, -90.0), Alignment::Bottom);
        assert_eq!(turned(Axis::Vertical, AxisPosition::Right, 90.0), Alignment::Bottom);
    }

    /// A band is as deep as its labels reach outwards as turned: a flat label below a bottom axis
    /// its height, beside a left axis its width, and a label turned upright, ending at its tick,
    /// its width below a bottom axis.
    #[test]
    fn bands_are_as_deep_as_the_turned_labels() {
        let label = TickLabel { value: 0.0, text: String::new(), size: Dimension::new(40.0, 10.0), shown: true };
        let labels = |axis, side, degrees: Scalar| {
            TickLabels::with_labels(vec![label.clone()], axis, side, degrees.to_radians(), None).depth(axis, side)
        };

        assert!((labels(Axis::Horizontal, AxisPosition::Bottom, 0.0) - 10.0).abs() < 1e-9);
        assert!((labels(Axis::Vertical, AxisPosition::Left, 0.0) - 40.0).abs() < 1e-9);
        assert!((labels(Axis::Horizontal, AxisPosition::Bottom, -90.0) - 40.0).abs() < 1e-9);

        // At 45°, ending at its tick: its far corner is the width and half the height down, turned.
        let diagonal = (40.0 + 5.0) * std::f64::consts::FRAC_1_SQRT_2;
        assert!((labels(Axis::Horizontal, AxisPosition::Bottom, -45.0) - diagonal).abs() < 1e-9);
    }

    /// Flat labels along a horizontal axis need their width apart, and along a vertical axis
    /// their height; labels turned by 45° lie side by side, and need their height over the sine.
    #[test]
    fn labels_need_their_thickness_along_the_axis() {
        let size = Dimension::new(40.0, 10.0);

        assert!((spacing(size, size, 0.0, true) - 44.0).abs() < 1e-9);
        assert!((spacing(size, size, 0.0, false) - 14.0).abs() < 1e-9);
        assert!((spacing(size, size, (-45.0f64).to_radians(), true) - 14.0 * std::f64::consts::SQRT_2).abs() < 1e-9);
        assert!((spacing(size, Dimension::new(60.0, 10.0), 0.0, true) - 64.0).abs() < 1e-9, "the wider of the two");
    }
}
