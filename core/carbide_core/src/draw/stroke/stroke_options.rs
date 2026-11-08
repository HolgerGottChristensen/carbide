use crate::draw::Scalar;
use crate::draw::stroke::{StrokeAlignment, LineCap, LineJoin};

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct StrokeOptions {
    /// What cap to use at the start of each sub-path.
    ///
    /// Default value: `LineCap::Butt`.
    pub start_cap: LineCap,

    /// What cap to use at the end of each sub-path.
    ///
    /// Default value: `LineCap::Butt`.
    pub end_cap: LineCap,

    /// See the SVG specification.
    ///
    /// Default value: `LineJoin::Miter`.
    pub stroke_join: LineJoin,

    /// Line width
    ///
    /// Default value: `StrokeOptions::DEFAULT_LINE_WIDTH`.
    pub stroke_width: Scalar,

    /// How the stroke should be aligned.
    pub stroke_alignment: StrokeAlignment,
}

impl StrokeOptions {
    pub const MINIMUM_MITER_LIMIT: Scalar = 1.0;
    pub const DEFAULT_MITER_LIMIT: Scalar = 4.0;
    pub const DEFAULT_LINE_CAP: LineCap = LineCap::Butt;
    pub const DEFAULT_LINE_JOIN: LineJoin = LineJoin::Miter;
    pub const DEFAULT_LINE_WIDTH: Scalar = 2.0;
    pub const DEFAULT_ALIGNMENT: StrokeAlignment = StrokeAlignment::Center;

    pub const DEFAULT: Self = StrokeOptions {
        start_cap: Self::DEFAULT_LINE_CAP,
        end_cap: Self::DEFAULT_LINE_CAP,
        stroke_join: Self::DEFAULT_LINE_JOIN,
        stroke_width: Self::DEFAULT_LINE_WIDTH,
        stroke_alignment: Self::DEFAULT_ALIGNMENT,
    };

    #[inline]
    pub const fn with_stroke_cap(mut self, cap: LineCap) -> Self {
        self.start_cap = cap;
        self.end_cap = cap;
        self
    }

    #[inline]
    pub const fn with_start_cap(mut self, cap: LineCap) -> Self {
        self.start_cap = cap;
        self
    }

    #[inline]
    pub const fn with_end_cap(mut self, cap: LineCap) -> Self {
        self.end_cap = cap;
        self
    }

    #[inline]
    pub const fn with_stroke_join(mut self, join: LineJoin) -> Self {
        self.stroke_join = join;
        self
    }

    #[inline]
    pub const fn with_stroke_width(mut self, width: Scalar) -> Self {
        self.stroke_width = width;
        self
    }

    #[inline]
    pub const fn with_alignment(mut self, alignment: StrokeAlignment) -> Self {
        self.stroke_alignment = alignment;
        self
    }
}

impl Default for StrokeOptions {
    fn default() -> Self {
        Self::DEFAULT
    }
}