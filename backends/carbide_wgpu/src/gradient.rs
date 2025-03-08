use carbide_core::color::ColorExt;
use carbide_core::draw::{ColorSpace, DrawGradient};
use carbide_core::draw::gradient::{GradientRepeat, GradientType};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Dashes {
    pub dashes: [f32; 32],
    pub dash_count: u32,
    pub start_cap: u32,
    pub end_cap: u32,
    pub total_dash_width: f32,
    pub dash_offset: f32,
}

impl Dashes {
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

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub colors: [[f32; 4]; 16],
    pub ratios: [f32; 16],

    pub num_colors: u32,
    pub gradient_type: i32,
    pub repeat_mode: i32,

    pub start: [f32; 2],
    pub end: [f32; 2],
    pub mode: u32,
}

impl Gradient {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(bytemuck::cast_slice(&self.colors));
        bytes.extend_from_slice(bytemuck::cast_slice(&self.ratios));

        bytes.extend_from_slice(bytemuck::bytes_of(&self.num_colors));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.mode));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.gradient_type));
        bytes.extend_from_slice(bytemuck::bytes_of(&self.repeat_mode));
        bytes.extend_from_slice(bytemuck::cast_slice(&self.start));
        bytes.extend_from_slice(bytemuck::cast_slice(&self.end));
        bytes
    }

    pub fn convert(gradient: &DrawGradient) -> Self {
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
            [0.0, 0.0, 0.0, 0.0],
        ];

        for (index, color) in gradient.colors.iter().enumerate() {
            colors[index] = color.gamma_srgb_to_linear()
                .pre_multiply()
                .to_fsa();
        }

        let mut ratios = [
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        for (index, ratio) in gradient.ratios.iter().enumerate() {
            ratios[index] = *ratio;
        }

        let gradient_type = match gradient.gradient_type {
            GradientType::Linear => 0,
            GradientType::Radial => 1,
            GradientType::Diamond => 2,
            GradientType::Conic => 3,
        };

        let repeat_mode = match gradient.gradient_repeat {
            GradientRepeat::Clamp => 0,
            GradientRepeat::Repeat => 1,
            GradientRepeat::Mirror => 2,
        };

        let mode = match gradient.color_space {
            ColorSpace::Linear => 0,
            ColorSpace::OkLAB => 1,
            ColorSpace::Srgb => 2,
            ColorSpace::Xyz => 3,
            ColorSpace::Cielab => 4,
            ColorSpace::HSL => 5,
        };

        if gradient.start == gradient.end && gradient.gradient_type == GradientType::Radial {
            Self {
                colors,
                ratios,
                num_colors: gradient.colors.len() as u32,
                gradient_type,
                repeat_mode,
                start: [gradient.start.x as f32, gradient.start.y as f32],
                end: [(gradient.end.x + 1.0) as f32, gradient.end.y as f32],
                mode,
            }
        } else if gradient.start == gradient.end {
            Self {
                colors,
                ratios,
                num_colors: gradient.colors.len() as u32,
                gradient_type: 100, // This means we should hit the default course.
                repeat_mode,
                start: [gradient.start.x as f32, gradient.start.y as f32],
                end: [gradient.end.x as f32, gradient.end.y as f32],
                mode,
            }
        } else {
            Self {
                colors,
                ratios,
                num_colors: gradient.colors.len() as u32,
                gradient_type,
                repeat_mode,
                start: [gradient.start.x as f32, gradient.start.y as f32],
                end: [gradient.end.x as f32, gradient.end.y as f32],
                mode,
            }
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
