use std::ops::Range;

#[derive(Debug, Clone)]
pub enum RenderPassCommand {
    DrawIndexed(Range<u32>)
}