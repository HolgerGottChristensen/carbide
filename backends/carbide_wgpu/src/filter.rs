use std::convert::TryInto;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Filter<const N: usize> {
    pub texture_size: [f32; 2],
    pub number_of_filter_entries: u32,
    pub filter_entries: [[f32; 4]; N],
}

impl<const N: usize> Filter<N> {
    pub(crate) fn set_texture_size(mut self, width: u32, height: u32) -> Self {
        self.texture_size = [width as f32, height as f32];
        self
    }
}


unsafe impl<const N: usize> Zeroable for Filter<N> {}

unsafe impl<const N: usize> Pod for Filter<N> {}

impl<const N: usize> From<carbide_core::widget::Filter> for Filter<N> {
    fn from(filter: carbide_core::widget::Filter) -> Self {
        let filter_len = filter.filter.len();
        let converted_filters = filter.filter.iter().map(|f| {
            [0.0, f.offset_x as f32, f.offset_y as f32, f.weight]
        }).collect::<Vec<_>>();

        Filter {
            texture_size: [100.0, 100.0],
            number_of_filter_entries: filter_len as u32,
            filter_entries: converted_filters.try_into().unwrap(),
        }
    }
}