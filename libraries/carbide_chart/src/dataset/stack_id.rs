use std::borrow::Cow;
use carbide::impl_state_value;

/// Names a stack of bars within one controller (see `DatasetExt::stack`). Datasets with the same
/// stack id, on the same x and y scales, are stacked on top of each other in one bar; each stack
/// sits beside the others, and beside datasets without one.
///
/// Ids known at compile time are `const` (`StackId::new("costs")`); ids built from runtime data
/// use `StackId::owned` or `From<String>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackId(Cow<'static, str>);

impl StackId {
    pub const fn new(name: &'static str) -> StackId {
        StackId(Cow::Borrowed(name))
    }

    /// An id named at runtime.
    pub fn owned(name: impl Into<String>) -> StackId {
        StackId(Cow::Owned(name.into()))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for StackId {
    fn from(name: &'static str) -> StackId {
        StackId::new(name)
    }
}

impl From<String> for StackId {
    fn from(name: String) -> StackId {
        StackId::owned(name)
    }
}

impl_state_value!(StackId);
