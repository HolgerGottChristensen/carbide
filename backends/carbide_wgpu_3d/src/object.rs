use encase::ShaderType;
use carbide_core::math::{Matrix4, Vector4};

#[derive(Debug, Copy, Clone, ShaderType, PartialEq)]
pub struct WgpuObject {
    pub transform: Matrix4<f32>,
    pub material_index: u32,
    /// 1 when shadows fall on the object, 0 when not.
    pub receives_shadows: u32,
    /// The id the object is picked by, or 0 when it isn't pickable.
    pub pick_id: u32,
    /// The outline's linear colour in `xyz`, and its thickness in `w`: in logical pixels when
    /// recorded, in physical pixels when uploaded. 0 for no outline.
    pub outline: Vector4<f32>,
}
