use cgmath::{Matrix, Matrix4, Vector2, Vector3, Vector4};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Gradient {
    pub colors: [[f32; 4]; 16],
    pub ratios: [f32; 16],

    pub num_colors: u32,
    pub gradient_type: i32,
    pub repeat_mode: i32,

    pub start: [f32; 2],
    pub end: [f32; 2],
}

impl Gradient {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(bytemuck::cast_slice(&self.colors));
        bytes.extend_from_slice(bytemuck::cast_slice(&self.ratios));

        bytes.extend_from_slice(bytemuck::bytes_of(&self.num_colors));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.gradient_type));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.repeat_mode));

        // If gradients start to look funny, take a look at this. I think it has to do with
        // some alignment but I dont fully understand why this has to be here.
        // My guess is that the three u32 has a size of 12. Since alignment of vec2<f32> is
        // 8 the next position to read from will be 16 which is a byte after the start of the
        // vec<32>. This should be corrected by inserting 4 bytes that will not be used.
        bytes.extend_from_slice(bytemuck::bytes_of(&self.repeat_mode));

        bytes.extend_from_slice(bytemuck::cast_slice(&self.start));
        bytes.extend_from_slice(bytemuck::cast_slice(&self.end));
        bytes
    }

    pub fn convert(gradient: carbide_core::prelude::Gradient) -> Self {

        let mut colors = [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0]
        ];

        for (index, color) in gradient.colors.iter().enumerate() {
            let rgb = color.to_rgb();
            colors[index] = [rgb.0, rgb.1, rgb.2, rgb.3];
        }

        let mut ratios = [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ];

        for (index, ratio) in gradient.ratios.iter().enumerate() {
            ratios[index] = *ratio;
        }

        assert_ne!(
            gradient.start,
            gradient.end,
            "The start and end points should not be equal, because we can have division by 0 in the shader."
        );

        Self {
            colors,
            ratios,
            num_colors: 2,
            gradient_type: 0,
            repeat_mode: 0,
            start: [gradient.start.x() as f32, gradient.start.y() as f32],
            end: [gradient.end.x() as f32, gradient.end.y() as f32]
        }
    }
}


// impl From<carbide_core::widget::ImageFilter> for Gradient {
//     fn from(filter: carbide_core::widget::ImageFilter) -> Self {
//         let filter_len = filter.filter.len();
//         let converted_filters = filter.filter.iter().map(|f| {
//             [0.0, f.offset_x as f32, f.offset_y as f32, f.weight]
//         }).collect::<Vec<_>>();
//
//         Gradient {
//             texture_size: [100.0, 100.0],
//             number_of_filter_entries: filter_len as u32,
//             filter_entries: converted_filters,
//         }
//     }
// }