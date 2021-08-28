#[repr(C)]
#[derive(Clone, Debug)]
pub struct Filter {
    pub texture_size: [f32; 2],
    pub number_of_filter_entries: u32,
    pub filter_entries: Vec<[f32; 4]>,
}

impl Filter {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(bytemuck::bytes_of(&self.texture_size));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.number_of_filter_entries));
        bytes.extend_from_slice(bytemuck::cast_slice(self.filter_entries.as_slice()));
        bytes
    }
}

impl From<carbide_core::widget::ImageFilter> for Filter {
    fn from(filter: carbide_core::widget::ImageFilter) -> Self {
        let filter_len = filter.filter.len();
        let converted_filters = filter.filter.iter().map(|f| {
            [0.0, f.offset_x as f32, f.offset_y as f32, f.weight]
        }).collect::<Vec<_>>();

        Filter {
            texture_size: [100.0, 100.0],
            number_of_filter_entries: filter_len as u32,
            filter_entries: converted_filters,
        }
    }
}