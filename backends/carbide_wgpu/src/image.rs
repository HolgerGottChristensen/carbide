use wgpu::BindGroup;

pub struct BindGroupExtended {
    pub bind_group: BindGroup,
    pub width: u32,
    pub height: u32,
}