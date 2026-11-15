use crate::scale::{Axis, AxisPosition};

/// How a scale's axis is laid out and drawn: the side of the chart area it sits on, its order
/// among the scales on that side, and whether it draws grid lines and tick marks. Every scale type
/// holds one and hands it out with `Scale::axis_options`; the axis code (`scale/axis.rs`) reads it,
/// so it is laid out and drawn the same way for every scale. The scales' builder methods
/// (`LinearScale::position` and so on) set it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisOptions {
    axis: Axis,
    position: AxisPosition,
    weight: i32,
    grid_lines: bool,
    tick_marks: bool,
}

impl AxisOptions {
    /// The defaults for a scale along `axis`: on its default side (see `Axis::default_position`),
    /// weight 0, with grid lines and tick marks.
    pub fn new(axis: Axis) -> AxisOptions {
        AxisOptions {
            axis,
            position: axis.default_position(),
            weight: 0,
            grid_lines: true,
            tick_marks: true,
        }
    }

    /// The side of the chart area the axis's ticks, labels and border are drawn on, and where the
    /// chart reserves space for them.
    pub fn position(&self) -> AxisPosition {
        self.position
    }

    /// Orders the axis among those on the same side: a higher weight puts it further away from the
    /// chart area. Axes with the same weight keep the order their scales were added to the chart
    /// in, the first one closest to the area.
    pub fn weight(&self) -> i32 {
        self.weight
    }

    /// Whether the axis draws grid lines across the chart area.
    pub fn grid_lines(&self) -> bool {
        self.grid_lines
    }

    /// Whether the axis draws tick marks in its band, between the chart area and its labels.
    pub fn tick_marks(&self) -> bool {
        self.tick_marks
    }

    /// Puts the axis on `position`. Horizontal axes go above or below the chart area, vertical
    /// axes left or right of it; any other side is logged and the axis keeps its current side.
    pub fn with_position(mut self, position: AxisPosition) -> AxisOptions {
        if self.axis.allows(position) {
            self.position = position;
        } else {
            log::warn!("A {:?} scale can't be positioned {:?}; keeping {:?}", self.axis, position, self.position);
        }

        self
    }

    pub fn with_weight(mut self, weight: i32) -> AxisOptions {
        self.weight = weight;
        self
    }

    pub fn with_grid_lines(mut self, grid_lines: bool) -> AxisOptions {
        self.grid_lines = grid_lines;
        self
    }

    pub fn with_tick_marks(mut self, tick_marks: bool) -> AxisOptions {
        self.tick_marks = tick_marks;
        self
    }
}
