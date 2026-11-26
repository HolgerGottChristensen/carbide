use carbide::color::Color;

/// How a mesh is drawn, besides its material: whether it casts and receives shadows, and its
/// outline. Read from the environment by [`RenderContext3d::mesh`](crate::render::RenderContext3d::mesh).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MeshSettings {
    pub casts_shadows: bool,
    pub receives_shadows: bool,
    pub outline: Option<Outline>,
}

impl MeshSettings {
    /// No shadows and no outline: for helpers, such as axes.
    pub const PLAIN: MeshSettings = MeshSettings { casts_shadows: false, receives_shadows: false, outline: None };
}

/// A band around the silhouette of what is outlined.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Outline {
    /// The band's colour, in sRGB, as carbide's colours are.
    pub color: Color,
    /// The band's width, in logical pixels.
    pub thickness: f32,
}
