//! What the pointer does in a chart: the data it finds, which hover and clicks report, and the
//! zoom windows its gestures move.

use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use carbide::draw::{Position, Rect, Scalar};
use carbide::environment::Environment;
use carbide::event::ModifierKey;
use crate::controller::orientation::pixel_normalized;
use crate::geometry::DatumKey;
use crate::scale::{Axis, Scale, ScaleId, ScaleValue, ZoomWindow};

/// What the pointer found in a chart: where it is, in the chart's own coordinates, from its
/// top-left corner; the data found there, as the chart's pick mode finds them (see
/// `Chart::hover_mode`); and the value under it on each of the chart's scales.
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub position: Position,
    pub data: Vec<DatumKey>,
    pub values: Vec<(ScaleId, ScaleValue)>,
}

impl Pick {
    /// The value under the pointer on the scale `id`, if the chart has it.
    pub fn value(&self, id: &ScaleId) -> Option<&ScaleValue> {
        self.values.iter().find(|(scale, _)| scale == id).map(|(_, value)| value)
    }
}

/// What a chart's click action is given (see `Chart::on_click`): what was found under the
/// pointer, the environment, and the modifier keys held.
pub struct ChartClick<'a, 'b: 'a> {
    pub pick: &'a Pick,
    pub env: &'a mut Environment<'b>,
    pub modifier_key: ModifierKey,
}

/// A chart's click action.
#[derive(Clone)]
pub(crate) struct ClickAction(pub(crate) Rc<dyn Fn(ChartClick)>);

impl Debug for ClickAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("ClickAction")
    }
}

/// A drag in progress: panning from where the pointer last was, or drawing a box to zoom to from
/// where it started, each moving `scales`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Gesture {
    Pan { last: Position, scales: Vec<ScaleId> },
    Box { origin: Position, to: Position, scales: Vec<ScaleId> },
}

/// How much one line of the wheel, 10 of carbide's scroll units, zooms: a tenth.
pub(crate) const WHEEL_ZOOM_PER_LINE: Scalar = 1.1;

/// The share of `scale`'s fitted range at `position`, a point in the chart's own coordinates,
/// with `window` shown in `area`: along x for a horizontal scale and along y for a vertical one.
pub(crate) fn share_at(scale: &dyn Scale, window: ZoomWindow, position: Position, area: Rect) -> Scalar {
    let normalized = match scale.axis() {
        Axis::Vertical => pixel_normalized(Axis::Vertical, position.y, area),
        _ => pixel_normalized(Axis::Horizontal, position.x, area),
    };
    let normalized = if scale.is_reversed() { 1.0 - normalized } else { normalized };

    window.unzoom(normalized)
}

/// `window` zoomed by `factor` about `anchor`, a share of the fitted range, which stays where it
/// is: a factor below 1 zooms in. Kept within the range and at least `min_span` wide.
pub(crate) fn zoom_about(window: ZoomWindow, anchor: Scalar, factor: Scalar, min_span: Scalar) -> ZoomWindow {
    let start = anchor - (anchor - window.start) * factor;
    let end = anchor + (window.end - anchor) * factor;
    let zoomed = ZoomWindow::new(start, end);

    // At the smallest span, zooming in further keeps the window as it is.
    if zoomed.span() < min_span && factor < 1.0 {
        return window.limited(min_span);
    }

    zoomed.limited(min_span)
}

/// `window` moved by `delta`, a share of the fitted range, stopping at the range's ends.
pub(crate) fn pan(window: ZoomWindow, delta: Scalar, min_span: Scalar) -> ZoomWindow {
    let span = window.span();
    let start = (window.start + delta).clamp(0.0, (1.0 - span).max(0.0));
    ZoomWindow::new(start, start + span).limited(min_span)
}

#[cfg(test)]
mod tests {
    use super::{pan, zoom_about};
    use crate::scale::ZoomWindow;

    fn close(a: ZoomWindow, b: ZoomWindow) -> bool {
        (a.start - b.start).abs() < 1e-9 && (a.end - b.end).abs() < 1e-9
    }

    /// Zooming keeps the anchor where it is: in by a half about a quarter of the way, the quarter
    /// stays a quarter of the way across the window.
    #[test]
    fn zooming_keeps_the_anchor_in_place() {
        let zoomed = zoom_about(ZoomWindow::WHOLE, 0.25, 0.5, 0.01);
        assert!(close(zoomed, ZoomWindow::new(0.125, 0.625)), "{zoomed:?}");
        assert!((zoomed.zoom(0.25) - 0.25).abs() < 1e-9);

        let out = zoom_about(zoomed, 0.25, 2.0, 0.01);
        assert!(close(out, ZoomWindow::WHOLE), "zooming back out returns to the whole range: {out:?}");
    }

    /// Zooming stops at the smallest span, and zooming out stops at the whole range, moved back
    /// inside it.
    #[test]
    fn zooming_stops_at_the_limits() {
        let narrow = ZoomWindow::new(0.5, 0.52);
        assert_eq!(zoom_about(narrow, 0.51, 0.1, 0.02), narrow);

        let out = zoom_about(ZoomWindow::new(0.7, 0.9), 0.8, 3.0, 0.01);
        assert!(close(out, ZoomWindow::new(0.4, 1.0)), "moved back inside, keeping its span: {out:?}");
    }

    /// Panning moves the window, keeping its span, and stops at the range's ends.
    #[test]
    fn panning_stops_at_the_ends() {
        assert!(close(pan(ZoomWindow::new(0.2, 0.4), 0.1, 0.01), ZoomWindow::new(0.3, 0.5)));
        assert!(close(pan(ZoomWindow::new(0.2, 0.4), -0.5, 0.01), ZoomWindow::new(0.0, 0.2)));
        assert!(close(pan(ZoomWindow::new(0.2, 0.4), 0.9, 0.01), ZoomWindow::new(0.8, 1.0)));
        assert_eq!(pan(ZoomWindow::WHOLE, 0.3, 0.01), ZoomWindow::WHOLE, "the whole range can't move");
    }
}
