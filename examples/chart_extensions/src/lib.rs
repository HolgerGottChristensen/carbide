//! Chart elements of this crate's own, built only on the public API of `carbide::chart`, as an
//! application outside the carbide workspace would build them:
//!
//! - `Kelvin`: a coordinate type (`DataValue`), a temperature placed along a scale in degrees
//!   Celsius.
//! - `LogScale`: a scale (`Scale`) that places values by their base 10 logarithm.
//! - `LollipopController`: a controller (`DatasetController`) that draws each point as a stem from
//!   zero to its value, with a dot at the end.
//!
//! The tests in `tests/` snapshot each one, and `main` shows them in a window.
//!
//! Not covered yet, as the public API doesn't allow it (see
//! `libraries/carbide_chart/docs/api-review.md`, step 1):
//!
//! - A mark of its own: `BarMark`, `LineMark` and `ScatterMark` can be implemented, but there is no
//!   public way to attach one to a dataset.
//! - Text categories and dates in `LollipopController`: placing them needs `Coordinates`, which a
//!   controller outside the crate can't make, so it reads plain numbers only.
//! - Palette colours in `LollipopController`: the chart's palette isn't public, so a dataset
//!   without a colour of its own falls back to `FALLBACK_COLOR`.

mod kelvin;
mod log_scale;
mod lollipop;

pub use kelvin::Kelvin;
pub use log_scale::LogScale;
pub use lollipop::{LollipopController, FALLBACK_COLOR};
