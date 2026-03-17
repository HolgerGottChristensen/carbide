use crate::draw::Dimension;

#[derive(Clone, Debug)]
pub enum ImageMetrics {
    Unknown,
    Raster {
        width: u32,
        height: u32
    },
    Vector {
        dimension: Dimension
    }
}