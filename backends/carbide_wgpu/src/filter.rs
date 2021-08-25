use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Filter<const N: usize> {
    pub texture_size: [f32; 2],
    pub number_of_filter_entries: u32,
    pub filter_entries: [[f32; 4]; N],
}

unsafe impl<const N: usize> Zeroable for Filter<N> {}

unsafe impl<const N: usize> Pod for Filter<N> {}