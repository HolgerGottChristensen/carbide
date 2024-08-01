use std::ops::Range;
use crate::pbr_material::WgpuPbrMaterialTextures;

#[derive(Debug, Clone)]
pub enum RenderPassCommand {
    SetPbrMaterial(WgpuPbrMaterialTextures),
    DrawIndexed(Range<u32>),
}