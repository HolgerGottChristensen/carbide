

pub struct Texture<'a> {
    pub width: u32,
    pub height: u32,
    pub bytes_per_row: u32,
    pub format: TextureFormat,
    pub data: &'a [u8],
}

pub enum TextureFormat {
    /// 8 bits per channel, red first.
    RGBA8,
    /// 8 bits per channel, blue first.
    BGRA8,
    /// A 16-bit float per channel, little-endian, red first: 8 bytes a texel. For HDR images, such
    /// as environments, whose values go above 1. Linear, never sRGB, and not premultiplied.
    RGBA16F,
}