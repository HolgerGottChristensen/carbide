pub mod mesh;
pub mod vertex;

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

/// Default dimensions to use for the glyph cache.
pub const DEFAULT_GLYPH_CACHE_DIMS: [u32; 2] = [1024; 2];

const GLYPH_CACHE_SCALE_TOLERANCE: f32 = 0.1;
const GLYPH_CACHE_POSITION_TOLERANCE: f32 = 0.1;