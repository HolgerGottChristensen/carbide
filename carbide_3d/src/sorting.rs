/// Description of how this object should be sorted.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Sorting {
    pub reason: SortingReason,
    pub order: SortingOrder,
}

impl Sorting {
    /// Default sorting for opaque and cutout objects
    pub const OPAQUE: Self = Self { reason: SortingReason::Optimization, order: SortingOrder::FrontToBack };

    /// Default sorting for any objects using blending
    pub const BLENDING: Self = Self { reason: SortingReason::Requirement, order: SortingOrder::BackToFront };
}

/// Reason why object need sorting
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SortingReason {
    /// Objects should be sorted for optimization purposes.
    Optimization,
    /// If objects aren't sorted, things will render incorrectly.
    Requirement,
}

/// An object sorting order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SortingOrder {
    /// Sort with the nearest objects first.
    FrontToBack,
    /// Sort with the furthest objects first.
    BackToFront,
}