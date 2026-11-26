use carbide_3d::Mesh;
use crate::pbr_material::WgpuPbrMaterialTextures;

#[derive(Debug, Clone)]
pub enum RenderPassCommand {
    SetPbrMaterial(WgpuPbrMaterialTextures),
    /// Draw a mesh as the object with this index; into the shadow maps too when it casts shadows.
    /// A double-sided mesh is drawn without culling its back faces.
    DrawMesh { mesh: Mesh, object: u32, casts_shadows: bool, double_sided: bool },
}