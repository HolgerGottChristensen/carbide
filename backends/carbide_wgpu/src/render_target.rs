use carbide_core::render::InnerLayer;
use std::fmt::{Debug, Formatter};
use wgpu::{BindGroup, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};

use crate::application::DEVICE;
use crate::bind_group_layouts::MAIN_TEXTURE_BIND_GROUP_LAYOUT;
use crate::globals::MAIN_SAMPLER;

pub struct RenderTarget {
    pub(crate) texture: Texture,
    pub(crate) view: TextureView,
    pub(crate) bind_group: BindGroup,
}

impl RenderTarget {
    pub(crate) fn new(width: u32, height: u32) -> RenderTarget {
        let descriptor = TextureDescriptor {
            label: Some("carbide_render_target_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let texture = DEVICE.create_texture(&descriptor);

        let view = texture.create_view(&Default::default());

        let bind_group = DEVICE.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &MAIN_TEXTURE_BIND_GROUP_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&MAIN_SAMPLER),
                },
            ],
            label: Some("carbide_render_target_bind_group"),
        });

        RenderTarget {
            texture,
            view,
            bind_group,
        }
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn texture_format(&self) -> TextureFormat {
        self.texture.format()
    }
}

impl InnerLayer for RenderTarget {
    fn dimensions(&self) -> (u32, u32) {
        (self.texture.width(), self.texture.height())
    }
}

impl Debug for RenderTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderTarget")
            .finish()
    }
}