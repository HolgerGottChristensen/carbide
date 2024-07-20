use crate::pbr_material::WgpuPbrMaterial;

#[derive(Debug, PartialEq, Clone)]
pub enum WgpuMaterial {
    PBR(WgpuPbrMaterial)
}