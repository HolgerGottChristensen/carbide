use crate::draw::image::image_id::ImageId;

pub type ImageMap<Img> = fxhash::FxHashMap<ImageId, Img>;
