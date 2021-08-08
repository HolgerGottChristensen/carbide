use crate::draw::Dimension;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::layouter::Layouter;
use crate::prelude::Environment;
use crate::widget::CommonWidget;

pub trait Layout {
    /// 0 is the most flexible and the largest number is the least flexible
    fn flexibility(&self) -> u32;
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension;
    fn position_children(&mut self);
}