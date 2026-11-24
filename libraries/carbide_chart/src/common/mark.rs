use std::fmt::Debug;
use carbide::environment::Environment;
use crate::Role;

/// What every mark is, whichever controllers draw it (`BarMark`, `LineMark`, `ScatterMark`): a
/// value that can be cloned, printed, and synced. A mark's options can be states, as a dataset's
/// can, and its dataset syncs them before every update, render and event (see `DataSet::sync`).
///
/// # Marks of your own
///
/// A mark of your own implements `Mark`, with its role, and the family trait of each kind of
/// controller that draws it, and is attached with `DatasetExt::draw_as` when it replaces the
/// controller's default glyph (`type Role = Primary`), or `DatasetExt::overlay` when it is drawn
/// on top of it (`type Role = Overlay`).
///
/// A mark for plain points implements its family trait for every `D: DataSet + ?Sized`, and reads
/// the points through `DataSet::points`. That way it can be attached to any dataset, including
/// one that already has options or other marks. A mark that reads more than an x and a y, such as
/// a low and a high, implements it for `D: Sequence<dyn RangePoint>` and the like, which options
/// and marks keep.
///
/// A mark adds a shape for each point it draws (`BarCanvas::add_rect`, `ScatterCanvas::add_marker`),
/// so the chart finds its points under the pointer. Its options are set before it is attached, or
/// after through an extension trait on `Marked<D, YourMark>` (see `Marked::map_mark`).
///
/// ```
/// use carbide_chart::{BarCanvas, BarController, BarMark, BarSlots, Coordinates, DataPoint, DataSet, DatasetExt, Mark, Primary};
/// use carbide_core::draw::Scalar;
///
/// /// Each bar as a thin stem from zero to its value.
/// #[derive(Clone, Debug)]
/// struct Stem;
///
/// impl Mark for Stem {
///     type Role = Primary;
/// }
///
/// impl<D: DataSet + ?Sized> BarMark<D> for Stem {
///     fn bounds(&self, data: &D, coordinates: &Coordinates, f: &mut dyn FnMut(Scalar, Scalar, Scalar)) {
///         data.points(&mut |_, point| {
///             if let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) {
///                 f(x, y.min(0.0), y.max(0.0));
///             }
///         });
///     }
///
///     fn draw(&self, data: &D, slots: &BarSlots, canvas: &mut BarCanvas) {
///         data.points(&mut |index, point| {
///             let (Some(x), Some(y)) = (canvas.x(point), canvas.y(point)) else {
///                 return;
///             };
///             let slot = slots.slot(x);
///             let color = canvas.point_color(index, point.color());
///             let (from, to) = (canvas.value_pixel(0.0), canvas.value_pixel(y));
///             canvas.add_rect(index, x, &slot, from, to, color);
///             canvas.fill_rect(slot.center - 1.0, 2.0, from, to, color);
///         });
///     }
/// }
///
/// BarController::new((vec![3.0, 5.0, 2.0].draw_as(Stem), vec![2.0, 4.0, 1.0]));
/// ```
pub trait Mark: Clone + Debug + 'static {
    /// Whether the mark replaces the glyph its controller draws by default (`Primary`), or is
    /// drawn on top of it (`Overlay`), in every controller that draws it. As a type, so a dataset
    /// can't be given a second replacing mark (see `WithoutPrimaryMark`).
    type Role: Role;

    /// Syncs the states in the mark's options with the environment, and returns whether anything
    /// may have changed. The default syncs nothing: a mark without states has nothing to sync.
    fn sync(&mut self, _env: &mut Environment) -> bool {
        false
    }
}
