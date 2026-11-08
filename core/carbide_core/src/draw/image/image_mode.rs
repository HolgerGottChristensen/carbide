use crate::draw::ImageOptions;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum ImageMode {
    Image,
    Icon
}

impl From<ImageMode> for ImageOptions {
    fn from(value: ImageMode) -> Self {
        ImageOptions {
            source_rect: None,
            mode: value,
        }
    }
}