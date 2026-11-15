use carbide::draw::Scalar;
use carbide::impl_state_value;

/// Whether a dataset's line breaks or joins across a gap: a missing value (see
/// `DataPoint::is_missing`), or with `Within`, a wide stretch along x between two points. Set it
/// with `DatasetExt::span_gaps`. Each unbroken run of points is drawn on its own: a curve is
/// computed per run, and a filled area fills under each run, so neither is drawn across a break.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum SpanGaps {
    /// The line breaks at every missing value. The default.
    #[default]
    Never,
    /// The line joins the points on either side of missing values, as if they weren't there.
    Always,
    /// The line joins across gaps up to this wide along x, in data units, and breaks across
    /// wider ones, also between two points with no missing value between them.
    Within(Scalar),
}

impl_state_value!(SpanGaps);
