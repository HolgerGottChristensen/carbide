use std::fmt::Debug;
use carbide::draw::Scalar;
use carbide::math::{One, Zero};

/// One coordinate of a data point. `PartialOrd` is for finding a dataset's bounds.
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't be a coordinate of a data point",
    label = "not a coordinate",
    note = "coordinates are `Scalar` (f64) or `usize`",
)]
pub trait DataValue: Copy + Zero + One + Clone + Debug + PartialOrd + 'static {}

impl DataValue for Scalar {}
impl DataValue for usize {}