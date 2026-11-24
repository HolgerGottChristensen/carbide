use crate::Value;

/// One of a point's values, as a tooltip shows it (see `DataPoint::describe`). Values along the
/// value scale are written as the scale labels its ticks; other values as plain numbers.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PointValue<'a> {
    /// A value along the value scale, named when the point has several, such as `"open"`.
    Value { name: Option<&'static str>, value: Value<'a> },
    /// A span along the value scale from `low` to `high`, such as a floating bar's.
    Span { name: Option<&'static str>, low: Value<'a>, high: Value<'a> },
    /// A value that isn't along the value scale, such as a bubble's size.
    Other { name: &'static str, value: Value<'a> },
}
