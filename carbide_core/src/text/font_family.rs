use crate::text::{Font, FontId};

#[derive(Copy, Clone, Debug)]
pub struct FontFamily {
    pub(crate) fonts: FontId,
}

impl FontFamily {

    //Todo: create a method to get closest font, specified by weight and bold/italic
}