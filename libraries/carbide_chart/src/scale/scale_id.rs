use carbide::impl_state_value;

/// Names a scale owned by a chart. Datasets refer to the scales they are drawn on by id, so
/// datasets in different controllers can share one scale.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScaleId(&'static str);

impl ScaleId {
    /// The default horizontal scale.
    pub const X: ScaleId = ScaleId("x");
    /// The default vertical scale.
    pub const Y: ScaleId = ScaleId("y");

    pub const fn new(name: &'static str) -> ScaleId {
        ScaleId(name)
    }

    pub fn name(&self) -> &'static str {
        self.0
    }
}

impl_state_value!(ScaleId);
