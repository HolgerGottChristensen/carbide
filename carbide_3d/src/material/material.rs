use crate::material::pbr_material::PbrMaterial;

#[derive(Debug, Clone)]
pub enum Material {
    PBR(PbrMaterial)
}