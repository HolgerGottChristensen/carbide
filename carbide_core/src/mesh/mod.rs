pub use atlas::texture_atlas::AtlasEntry;
pub use atlas::texture_atlas::AtlasId;
pub use atlas::texture_atlas::TextureAtlas;
pub use atlas::texture_atlas::TextureAtlasIndex;
pub use draw_command::DrawCommand;
pub use draw_command::VertexRange;

mod atlas;
pub mod mesh;
pub mod pre_multiply;
pub mod vertex;
mod draw_command;

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;
/// Draw colored icons from main images and not the glyph atlas.
pub const MODE_ICON: u32 = 3;
/// Draw colored bitmap glyphs.
pub const MODE_TEXT_COLOR: u32 = 4;

/// Default dimensions to use for the glyph cache.
pub const DEFAULT_GLYPH_CACHE_DIMS: [u32; 2] = [1024; 2];

const GLYPH_CACHE_SCALE_TOLERANCE: f32 = 1.0;
const GLYPH_CACHE_POSITION_TOLERANCE: f32 = 1.0;
