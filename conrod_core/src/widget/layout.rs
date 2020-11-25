use position::Dimensions;
use text;

pub trait Layout {
    /// 0 is the most flexible and the largest number is the least flexible
    fn flexibility(&self) -> u32;
    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &text::font::Map) -> Dimensions;
    fn position_children(&mut self);
}