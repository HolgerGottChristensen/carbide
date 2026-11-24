use carbide::draw::Scalar;

/// What a coordinate stands for, as a scale places it: a number, a moment in time, or the name of
/// a category. The variant also says which scale suits it, so the chart picks a default scale from
/// the data: a `TimeScale` for times, a `CategoryScale` for names (see
/// `ScaleRequest::default_kind`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value<'a> {
    /// A plain number, placed at itself on a linear or a category scale.
    Number(Scalar),
    /// A moment in time, as seconds since the Unix epoch with fractions, placed at itself and
    /// labelled as a date on a `TimeScale`.
    Time(Scalar),
    /// The name of a category, placed at the category's index on a category scale, which numbers
    /// categories in the order the chart first sees them, after its own labels.
    Text(&'a str),
}

impl Value<'_> {
    /// The number the value stands for: a number, or a time as seconds since the Unix epoch.
    /// `None` for a category's name, which only a scale can place.
    pub fn number(self) -> Option<Scalar> {
        match self {
            Value::Number(number) | Value::Time(number) => Some(number),
            Value::Text(_) => None,
        }
    }
}
