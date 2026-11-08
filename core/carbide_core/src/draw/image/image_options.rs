use crate::draw::image::image_mode::ImageMode;
use crate::draw::Rect;

#[derive(Clone, Debug)]
pub struct ImageOptions {
    pub source_rect: Option<Rect>,
    pub mode: ImageMode,
}