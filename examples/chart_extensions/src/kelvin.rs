use carbide::chart::{DataValue, Value};
use carbide::draw::Scalar;

/// A temperature in kelvin, placed along a scale in degrees Celsius. Absolute zero and below
/// stand for a broken sensor, so they are missing values: not drawn, and a gap in a line.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Kelvin(pub Scalar);

impl DataValue for Kelvin {
    fn value(&self) -> Option<Value<'_>> {
        (self.0 > 0.0).then_some(Value::Number(self.0 - 273.15))
    }
}
