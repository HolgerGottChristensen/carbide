use std::borrow::Cow;
use carbide::impl_state_value;

/// Names a scale owned by a chart. Datasets refer to the scales they are drawn on by id, so
/// datasets in different controllers can share one scale.
///
/// Ids known at compile time are `const` (`ScaleId::new("percent")`); ids built from runtime
/// data, for example one scale per data source, use `ScaleId::owned` or `From<String>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScaleId(Cow<'static, str>);

impl ScaleId {
    /// The default horizontal scale.
    pub const X: ScaleId = ScaleId::new("x");
    /// The default vertical scale.
    pub const Y: ScaleId = ScaleId::new("y");

    pub const fn new(name: &'static str) -> ScaleId {
        ScaleId(Cow::Borrowed(name))
    }

    /// An id named at runtime.
    pub fn owned(name: impl Into<String>) -> ScaleId {
        ScaleId(Cow::Owned(name.into()))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for ScaleId {
    fn from(name: &'static str) -> ScaleId {
        ScaleId::new(name)
    }
}

impl From<String> for ScaleId {
    fn from(name: String) -> ScaleId {
        ScaleId::owned(name)
    }
}

impl_state_value!(ScaleId);

#[cfg(test)]
mod tests {
    use super::ScaleId;

    /// A borrowed and an owned id with the same name are the same id, so a scale added with a
    /// `const` id can be found from a runtime string and the other way round.
    #[test]
    fn borrowed_and_owned_ids_with_the_same_name_are_equal() {
        let owned = ScaleId::owned(String::from("percent"));
        assert_eq!(owned, ScaleId::new("percent"));
        assert_eq!(owned.name(), "percent");
        assert_ne!(owned, ScaleId::Y);
    }
}
