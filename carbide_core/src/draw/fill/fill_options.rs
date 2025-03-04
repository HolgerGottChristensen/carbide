use crate::draw::fill::fill_rule::FillRule;

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct FillOptions {
    pub fill_rule: FillRule,
}

impl Default for FillOptions {
    fn default() -> Self {
        FillOptions {
            fill_rule: FillRule::EvenOdd,
        }
    }
}