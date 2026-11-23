//! Chart elements of this crate's own, built only on the public API of `carbide::chart`, as an
//! application outside the carbide workspace would build them:
//!
//! - `Kelvin`: a coordinate type (`DataValue`), a temperature placed along a scale in degrees
//!   Celsius.
//! - `LogScale`: a scale (`Scale`) that places values by their base 10 logarithm.
//! - `LollipopController`: a controller (`DatasetController`) that draws each point as a stem from
//!   zero to its value, with a dot at the end, on numbers, names and dates, upright or horizontal.
//! - `Blocks`: a bar mark (`BarMark`) that replaces each bar with a column of blocks, attached with
//!   `DatasetExt::draw_as`, its unit set after attaching through `BlocksExt`; and `ValueTicks`, an
//!   overlay marking each value with a line across its bar, attached with `DatasetExt::overlay`.
//!
//! The tests in `tests/` snapshot each one. `main` is a gallery of charts, with their options in a
//! side bar, and its "Custom" page shows these elements (`src/gallery/custom.rs`).
//!
//! What the public API doesn't allow yet is listed in `libraries/carbide_chart/docs/api-review.md`,
//! step 1.

mod blocks;
mod kelvin;
mod log_scale;
mod lollipop;

pub use blocks::{Blocks, BlocksExt, ValueTicks};
pub use kelvin::Kelvin;
pub use log_scale::LogScale;
pub use lollipop::LollipopController;
