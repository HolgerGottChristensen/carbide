use std::borrow::Cow;
use carbide::impl_state_value;

/// Names a stack within one controller (see `DatasetExt::stack`). Datasets with the same stack
/// id, on the same x and y scales, are stacked: bars on top of each other in one bar, each stack
/// beside the others and beside datasets without one; lines at their running totals.
///
/// Ids known at compile time are `const` (`StackId::new("costs")`); ids built from runtime data
/// use `StackId::owned` or `From<String>`.
///
/// A normalised stack (`normalized`) shows each value as its share of its category's stack: the
/// stack at each x is scaled to sum to 100, so every bar is as tall as every other, its parts its
/// shares, as 100% stacked bars and areas are drawn. Two ids with the same name, one normalised
/// and one not, are different stacks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackId {
    name: Cow<'static, str>,
    normalized: bool,
}

impl StackId {
    pub const fn new(name: &'static str) -> StackId {
        StackId { name: Cow::Borrowed(name), normalized: false }
    }

    /// An id named at runtime.
    pub fn owned(name: impl Into<String>) -> StackId {
        StackId { name: Cow::Owned(name.into()), normalized: false }
    }

    /// The stack, normalised: at each x, every dataset in it is scaled so the stack's values sum
    /// to 100, their absolute values counting, so a stack with negative values splits the 100
    /// between its two sides. The controller reports 0 to 100 to the value scale. Every dataset in
    /// the stack is normalised together, as the setting is part of the id.
    pub const fn normalized(mut self) -> StackId {
        self.normalized = true;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the stack is normalised (see `normalized`).
    pub fn is_normalized(&self) -> bool {
        self.normalized
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
