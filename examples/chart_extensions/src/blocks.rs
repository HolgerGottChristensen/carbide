use carbide::chart::{BarCanvas, BarMark, BarSlots, Coordinates, DataSet, Mark, Marked, Overlay, Primary};
use carbide::draw::{Position, Scalar};

/// The gap between two blocks, in pixels.
const BLOCK_GAP: Scalar = 2.0;

/// The most blocks a bar is split into; a bar that would need more is drawn whole.
const MAX_BLOCKS: usize = 200;

/// How wide a value tick is, in pixels.
const TICK_WIDTH: Scalar = 2.0;

/// A bar mark of this crate's own that replaces each bar with a column of blocks, each worth
/// `unit`, from zero to the point's value, a little apart, so the bar can be counted. The last block
/// is cut short where the value isn't a whole number of units. Attach it with `DatasetExt::draw_as`,
/// and set its unit before attaching (`Blocks::new().unit(..)`) or after (`BlocksExt::unit`).
///
/// It reads plain points, so it can be attached to any dataset a bar controller draws, with options
/// or other marks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Blocks {
    unit: Scalar,
}

impl Blocks {
    /// Blocks worth 1 each.
    pub fn new() -> Blocks {
        Blocks { unit: 1.0 }
    }

    /// Makes each block worth `unit`. A unit that isn't a positive number draws the bar whole.
    pub fn unit(self, unit: Scalar) -> Blocks {
        Blocks { unit }
    }

    /// What each block is worth.
    pub fn block_unit(&self) -> Scalar {
        self.unit
    }
}

impl Default for Blocks {
    fn default() -> Blocks {
        Blocks::new()
    }
}

impl Mark for Blocks {
    type Role = Primary;
}

/// The unit of blocks already attached to a dataset, so it can be set after `draw_as`, as the
/// built-in marks' options are.
pub trait BlocksExt {
    /// Makes each block worth `unit` (see `Blocks::unit`).
    fn unit(self, unit: Scalar) -> Self;
}

impl<D> BlocksExt for Marked<D, Blocks> {
    fn unit(self, unit: Scalar) -> Self {
        self.map_mark(|blocks| blocks.unit(unit))
    }
}

impl<D: DataSet + ?Sized> BarMark<D> for Blocks {
    /// From zero to each point's value, as a bar.
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.points(&mut |_, point| {
            if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                f(x, y.min(0.0), y.max(0.0));
            }
        });
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        data.points(&mut |point_index, point| {
            let (Some(x), Some(y)) = (canvas.x(point), canvas.y(point)) else {
                return;
            };

            let slot = slots.slot(x);
            let color = canvas.point_color(point_index, point.color());

            // The whole bar is the point's shape, gaps included, so the pointer finds it anywhere
            // along the bar.
            let (zero, value) = (canvas.value_pixel(0.0), canvas.value_pixel(y));
            canvas.add_rect(point_index, x, &slot, zero, value, color);

            let count = (y.abs() / self.unit).ceil();
            if !(self.unit > 0.0) || !count.is_finite() || count as usize > MAX_BLOCKS {
                canvas.fill_rect(slot.side, slot.thickness, zero, value, color);
                return;
            }

            for block in 0..count as usize {
                let start = y.signum() * block as Scalar * self.unit;
                let end = y.signum() * ((block + 1) as Scalar * self.unit).min(y.abs());
                let (a, b) = (canvas.value_pixel(start), canvas.value_pixel(end));
                let (low, high) = (a.min(b), a.max(b));

                if high - low > BLOCK_GAP {
                    canvas.fill_rect(slot.side, slot.thickness, low + BLOCK_GAP / 2.0, high - BLOCK_GAP / 2.0, color);
                }
            }
        });
    }
}

/// An overlay of this crate's own that marks each point's value with a short line across its bar,
/// in the chart's ink, on top of whatever the bar is drawn as. Attach it with
/// `DatasetExt::overlay`. On stacked bars, the ticks move with the stack.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ValueTicks;

impl Mark for ValueTicks {
    type Role = Overlay;
}

impl<D: DataSet + ?Sized> BarMark<D> for ValueTicks {
    fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
        data.points(&mut |_, point| {
            if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
                f(x, y, y);
            }
        });
    }

    fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
        canvas.begin_lines();
        data.points(&mut |_, point| {
            let (Some(x), Some(y)) = (canvas.x(point), canvas.y(point)) else {
                return;
            };

            let slot = slots.slot(x);
            let at = canvas.value_pixel(y + slot.stack_offset);
            canvas.add_line(Position::new(slot.side, at), Position::new(slot.side + slot.thickness, at));
        });
        let ink = canvas.ink();
        canvas.stroke_lines_added(TICK_WIDTH, ink);
    }
}
