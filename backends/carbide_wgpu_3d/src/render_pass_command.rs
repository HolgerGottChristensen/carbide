use carbide_3d::Mesh;
use carbide_core::math::Vector3;
use crate::pbr_material::WgpuPbrMaterialTextures;

#[derive(Debug, Clone)]
pub enum RenderPassCommand {
    SetPbrMaterial(WgpuPbrMaterialTextures),
    /// Draw a mesh as the object with this index; into the shadow maps too when it casts shadows.
    /// A double-sided mesh is drawn without culling its back faces. A mirrored mesh, whose
    /// transform has a negative determinant, has its front faces clockwise on screen, as glTF
    /// requires. A blended mesh (`Transparency::Blend`) is drawn after the others, sorted by
    /// `centre`, its bounds' centre in world space.
    DrawMesh { mesh: Mesh, object: u32, casts_shadows: bool, double_sided: bool, mirrored: bool, blended: bool, centre: Vector3<f32> },
}

/// Which of the four pipeline variants draws a mesh, as an index: whether its back faces are
/// culled, and which winding faces the viewer.
pub(crate) fn pipeline_variant(double_sided: bool, mirrored: bool) -> usize {
    double_sided as usize * 2 + mirrored as usize
}

/// The culling and front face of each blending pipeline, by `blend_variant`: blended meshes cull
/// back faces, and double-sided ones draw twice, culling front faces first (their back faces,
/// the far side), then back faces.
pub(crate) const BLEND_VARIANTS: [(Option<wgpu::Face>, wgpu::FrontFace); 4] = [
    (Some(wgpu::Face::Back), wgpu::FrontFace::Ccw),
    (Some(wgpu::Face::Back), wgpu::FrontFace::Cw),
    (Some(wgpu::Face::Front), wgpu::FrontFace::Ccw),
    (Some(wgpu::Face::Front), wgpu::FrontFace::Cw),
];

/// Which blending pipeline draws a mesh's `back` or front faces.
pub(crate) fn blend_variant(back: bool, mirrored: bool) -> usize {
    back as usize * 2 + mirrored as usize
}

/// The culling and front face of each variant, in `pipeline_variant` order.
pub(crate) const PIPELINE_VARIANTS: [(Option<wgpu::Face>, wgpu::FrontFace); 4] = [
    (Some(wgpu::Face::Back), wgpu::FrontFace::Ccw),
    (Some(wgpu::Face::Back), wgpu::FrontFace::Cw),
    (None, wgpu::FrontFace::Ccw),
    (None, wgpu::FrontFace::Cw),
];
