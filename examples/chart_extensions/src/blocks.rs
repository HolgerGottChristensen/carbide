use carbide::chart::{BarCanvas, BarMark, BarSlots, Coordinates, DataSet, Mark, Marked, Overlay, Primary};
use carbide::draw::{Position, Scalar};
use carbide::environment::Environment;
use carbide::state::{IntoReadState, ReadState};

/// The gap between two blocks, in pixels.
const BLOCK_GAP: Scalar = 2.0;

/// The most blocks a bar is split into; a bar that would need more is drawn whole.
const MAX_BLOCKS: usize = 200;

/// How wide a value tick is, in pixels.
const TICK_WIDTH: Scalar = 2.0;

/// A bar mark of this crate's own that replaces each bar with a column of blocks, each worth
/// `unit`, from zero to the point's value, a little apart, so the bar can be counted. The last block
/// is cut short where the value isn't a whole number of units. Attach it with `DatasetExt::draw_as`,
/// and set its unit, a value or a state of one, before attaching (`Blocks::new().unit(..)`) or after
/// (`BlocksExt::unit`). A unit in a state is synced with the dataset, so the blocks follow it.
///
/// It reads plain points, so it can be attached to any dataset a bar controller draws, with options
/// or other marks.
#[derive(Clone, Debug)]
pub struct Blocks<U: ReadState<T=Scalar> = Scalar> {
    unit: U,
}

impl Blocks {
    /// Blocks worth 1 each.
    pub fn new() -> Blocks {
        Blocks { unit: 1.0 }
    }
}

impl<U: ReadState<T=Scalar>> Blocks<U> {
    /// Makes each block worth `unit`, a value or a state of one. A unit that isn't a positive number
    /// draws the bar whole.
    pub fn unit<U2: IntoReadState<Scalar>>(self, unit: U2) -> Blocks<U2::Output> {
        Blocks { unit: unit.into_read_state() }
    }

    /// What each block is worth, as last synced.
    pub fn block_unit(&self) -> Scalar {
        *self.unit.value()
    }
}

impl Default for Blocks {
    fn default() -> Blocks {
        Blocks::new()
    }
}

impl<U: ReadState<T=Scalar>> Mark for Blocks<U> {
    type Role = Primary;

    /// Syncs the unit, when it is a state.
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.unit.sync(env)
    }
}

/// The unit of blocks already attached to a dataset, so it can be set after `draw_as`, as the
/// built-in marks' options are.
pub trait BlocksExt<D> {
    /// Makes each block worth `unit`, a value or a state of one (see `Blocks::unit`).
    fn unit<U2: IntoReadState<Scalar>>(self, unit: U2) -> Marked<D, Blocks<U2::Output>>;
}

impl<D, U: ReadState<T=Scalar>> BlocksExt<D> for Marked<D, Blocks<U>> {
    fn unit<U2: IntoReadState<Scalar>>(self, unit: U2) -> Marked<D, Blocks<U2::Output>> {
        self.map_mark(|blocks| blocks.unit(unit))
    }
}

impl<D: DataSet + ?Sized, U: ReadState<T=Scalar>> BarMark<D> for Blocks<U> {
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

            let unit = self.block_unit();
            let count = (y.abs() / unit).ceil();
            if !(unit > 0.0) || !count.is_finite() || count as usize > MAX_BLOCKS {
                canvas.fill_rect(slot.side, slot.thickness, zero, value, color);
                return;
            }

            for block in 0..count as usize {
                let start = y.signum() * block as Scalar * unit;
                let end = y.signum() * ((block + 1) as Scalar * unit).min(y.abs());
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
