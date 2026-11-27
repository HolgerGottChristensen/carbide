use crate::material::pbr_material::PbrMaterial;
use carbide::impl_state_value;

/// A material, as a plain value: how a mesh's surface interacts with light.
#[derive(Debug, Clone, PartialEq)]
pub enum Material {
    PBR(PbrMaterial)
}

impl_state_value!(Material);
