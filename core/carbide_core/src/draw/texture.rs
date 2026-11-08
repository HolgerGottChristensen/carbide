

pub struct Texture<'a> {
    pub width: u32,
    pub height: u32,
    pub bytes_per_row: u32,
    pub format: TextureFormat,
    pub data: &'a [u8],
}

pub enum TextureFormat {
    RGBA8,
    BGRA8,
}