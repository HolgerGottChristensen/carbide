



#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct WgpuDashes {
    pub dashes: [f32; 32],
    pub dash_count: u32,
    pub start_cap: u32,
    pub end_cap: u32,
    pub total_dash_width: f32,
    pub dash_offset: f32,
}

impl WgpuDashes {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(bytemuck::cast_slice(&self.dashes));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.dash_count));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.start_cap));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.end_cap));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.total_dash_width));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.dash_offset));
        bytes
    }
}