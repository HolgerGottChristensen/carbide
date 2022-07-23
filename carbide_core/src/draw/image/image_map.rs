use crate::draw::image::image_id::ImageId;
use std;

pub type ImageMap<Img> = fxhash::FxHashMap<ImageId, Img>;
