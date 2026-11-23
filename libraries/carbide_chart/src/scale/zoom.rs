//! Zooming into part of a scale: a window over its fitted range, which the scale maps values
//! through, so everything drawn on it, and everything found under a position, moves together.

use carbide::draw::Scalar;
use carbide_derive::StateValue;

/// The smallest share of a scale's fitted range a window can show, on scales without categories.
pub(crate) const MIN_ZOOM_SPAN: Scalar = 0.01;

/// The part of a scale shown: from `start` to `end`, each a share of the scale's fitted range,
/// from 0 at its start to 1 at its end, in the order the values run. `ZoomWindow::WHOLE`, the
/// default, shows all of it.
///
/// A scale keeps fitting to its data, and shows the window of that fit (see
/// `LinearScale::zoom`). The window is kept within the fitted range and shows at least a small
/// part of it: two categories on a category scale, whose window snaps to whole categories, and
/// a hundredth of the range on the others.
#[derive(Copy, Clone, Debug, PartialEq, StateValue)]
pub struct ZoomWindow {
    pub start: Scalar,
    pub end: Scalar,
}

impl ZoomWindow {
    /// The whole fitted range.
    pub const WHOLE: ZoomWindow = ZoomWindow { start: 0.0, end: 1.0 };

    pub fn new(start: Scalar, end: Scalar) -> ZoomWindow {
        ZoomWindow { start, end }
    }

    /// Whether the window shows the whole fitted range.
    pub fn is_whole(&self) -> bool {
        self.start <= 0.0 && self.end >= 1.0
    }

    /// How much of the fitted range the window shows.
    pub fn span(&self) -> Scalar {
        self.end - self.start
    }

    /// The window as it can be shown: its ends in order, at least `min_span` apart, widened about
    /// its middle, and moved, not cut, to lie within the fitted range. A window that isn't a
    /// number shows the whole range.
    pub fn limited(self, min_span: Scalar) -> ZoomWindow {
        if !self.start.is_finite() || !self.end.is_finite() {
            return ZoomWindow::WHOLE;
        }

        let (low, high) = (self.start.min(self.end), self.start.max(self.end));
        if low >= 0.0 && high <= 1.0 && high - low >= min_span.min(1.0) {
            return ZoomWindow { start: low, end: high };
        }

        let span = (high - low).clamp(min_span.min(1.0), 1.0);
        let middle = (low + high) / 2.0;
        let start = (middle - span / 2.0).clamp(0.0, 1.0 - span);

        ZoomWindow { start, end: start + span }
    }

    /// The window snapped to whole steps of `1 / steps` of the range, at least `min_steps` steps
    /// wide, within the range: a category scale's window, whose steps are its categories' bands,
    /// or, without an offset, the spaces between its categories.
    pub(crate) fn snapped(self, steps: Scalar, min_steps: Scalar) -> ZoomWindow {
        if steps <= 0.0 {
            return ZoomWindow::WHOLE;
        }

        let limited = self.limited(min_steps / steps);
        let mut start = (limited.start * steps).round();
        let mut end = (limited.end * steps).round();
        if end - start < min_steps.min(steps) {
            end = (start + min_steps).min(steps);
            start = end - min_steps.min(steps);
        }

        ZoomWindow { start: start / steps, end: end / steps }
    }

    /// Where `share`, a share of the fitted range, falls in the window: 0 at its start and 1 at
    /// its end.
    pub fn zoom(&self, share: Scalar) -> Scalar {
        (share - self.start) / self.span()
    }

    /// The share of the fitted range at `normalized` in the window: the inverse of `zoom`.
    pub fn unzoom(&self, normalized: Scalar) -> Scalar {
        self.start + normalized * self.span()
    }
}

/// How a category scale's zoom window lines up with its categories (see
/// `CategoryScale::snap_mode`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, StateValue)]
pub enum SnapMode {
    /// The window snaps to whole categories, so the categories at its ends are shown whole and
    /// panning moves a category at a time. The default.
    #[default]
    Categories,
    /// The window is shown as asked for, so panning scrolls smoothly and the categories at its
    /// ends can be cut by the edges of the chart area.
    Free,
}

impl Default for ZoomWindow {
    fn default() -> ZoomWindow {
        ZoomWindow::WHOLE
    }
}

#[cfg(test)]
mod tests {
    use super::ZoomWindow;

    /// A window is put in order, widened to the smallest span about its middle, and moved back
    /// inside the range without changing its span.
    #[test]
    fn windows_stay_inside_the_range_and_wide_enough() {
        assert_eq!(ZoomWindow::new(0.6, 0.2).limited(0.01), ZoomWindow::new(0.2, 0.6), "ends in order");
        assert_eq!(ZoomWindow::new(0.5, 0.5).limited(0.1), ZoomWindow::new(0.45, 0.55), "widened about its middle");

        let moved = ZoomWindow::new(0.8, 1.3).limited(0.01);
        assert!((moved.start - 0.5).abs() < 1e-9 && moved.end == 1.0, "moved back inside, keeping its span: {moved:?}");

        assert_eq!(ZoomWindow::new(-1.0, 3.0).limited(0.01), ZoomWindow::WHOLE, "no wider than the range");
        assert_eq!(ZoomWindow::new(f64::NAN, 0.5).limited(0.01), ZoomWindow::WHOLE);
    }

    /// A snapped window starts and ends on whole steps, and shows at least the smallest number of
    /// them.
    #[test]
    fn snapped_windows_cover_whole_steps() {
        assert_eq!(ZoomWindow::new(0.27, 0.52).snapped(8.0, 2.0), ZoomWindow::new(0.25, 0.5), "to the nearest eighths");
        assert_eq!(ZoomWindow::new(0.27, 0.52).snapped(4.0, 2.0), ZoomWindow::new(0.25, 0.75), "widened to two quarters");
        assert_eq!(ZoomWindow::new(0.3, 0.45).snapped(10.0, 2.0), ZoomWindow::new(0.3, 0.5), "widened to two steps");
        assert_eq!(ZoomWindow::new(0.9, 1.0).snapped(4.0, 2.0), ZoomWindow::new(0.5, 1.0), "two steps, inside the range");
        assert_eq!(ZoomWindow::new(0.2, 0.8).snapped(1.0, 2.0), ZoomWindow::WHOLE, "a range of one step shows it whole");
    }

    /// `unzoom` undoes `zoom`.
    #[test]
    fn zoom_and_unzoom_are_inverses() {
        let window = ZoomWindow::new(0.25, 0.75);
        assert_eq!(window.zoom(0.5), 0.5);
        assert_eq!(window.zoom(0.25), 0.0);
        assert_eq!(window.unzoom(1.0), 0.75);
        assert_eq!(window.unzoom(window.zoom(0.1)), 0.1);
    }
}
