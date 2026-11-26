use carbide::color::Color;
use carbide::math::Vector3;

/// How a mesh is drawn, besides its material: whether it casts and receives shadows, and its
/// outline. Read from the environment by [`RenderContext3d::mesh`](crate::render::RenderContext3d::mesh).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MeshSettings {
    pub casts_shadows: bool,
    pub receives_shadows: bool,
    pub outline: Option<Outline>,
    /// The id the mesh is picked by, 1 and up, or 0 when it isn't pickable. Ids are per frame:
    /// the view maps them back to nodes.
    pub pick_id: u32,
}

impl MeshSettings {
    /// No shadows and no outline: for helpers, such as axes.
    pub const PLAIN: MeshSettings = MeshSettings { casts_shadows: false, receives_shadows: false, outline: None, pick_id: 0 };
}

/// A band around the silhouette of what is outlined.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Outline {
    /// The band's colour, in sRGB, as carbide's colours are.
    pub color: Color,
    /// The band's width, in logical pixels.
    pub thickness: f32,
}

/// What the renderer found at a pick: the pick id of the mesh hit, and where.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PickSample {
    /// The [`MeshSettings::pick_id`] of the mesh hit; never 0.
    pub pick_id: u32,
    /// The point hit, in world space.
    pub position: Vector3<f32>,
    /// The surface's normal there, in world space and facing the camera, if the pixels around
    /// tell.
    pub normal: Option<Vector3<f32>>,
    /// The distance from the camera to `position`.
    pub distance: f32,
}
