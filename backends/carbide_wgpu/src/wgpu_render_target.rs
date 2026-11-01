use carbide_core::render::InnerLayer;
use std::fmt::{Debug, Formatter};
use wgpu::{BindGroup, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};
use carbide_core::environment::Environment;
use crate::textures::{create_depth_stencil_texture_view, create_msaa_texture_view};
use crate::wgpu_context::WgpuContext;
use crate::wgpu_msaa::WgpuMsaa;

pub const RENDER_TARGET_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

pub struct WgpuRenderTarget {
    pub(crate) texture: Texture,
    pub(crate) view: TextureView,
    pub(crate) bind_group: BindGroup,
    /// This target's own MSAA scratch texture — `None` at `WgpuMsaa::X1`, where there's no
    /// separate multisample attachment at all.
    pub(crate) msaa_view: Option<TextureView>,
    /// This target's own depth/stencil attachment, sized to match its own (possibly padded, for
    /// shadow/filter scratch targets) dimensions rather than the window's — a shared, window-sized
    /// depth attachment can't be used for a render pass whose color attachment is a different size,
    /// which wgpu requires to match.
    pub(crate) depth_view: TextureView,
}

impl WgpuRenderTarget {
    pub(crate) fn new(width: u32, height: u32, msaa: WgpuMsaa, env: &mut Environment) -> WgpuRenderTarget {
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
            format: RENDER_TARGET_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let wgpu_context = env.get_mut::<WgpuContext>().unwrap();

        let texture = wgpu_context.device.create_texture(&descriptor);

        let view = texture.create_view(&Default::default());

        let bind_group = wgpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &wgpu_context.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&wgpu_context.main_sampler),
                },
            ],
            label: Some("carbide_render_target_bind_group"),
        });

        let msaa_view = create_msaa_texture_view(&wgpu_context.device, width, height, msaa);
        let depth_view = create_depth_stencil_texture_view(&wgpu_context.device, width, height, msaa);

        WgpuRenderTarget {
            texture,
            view,
            bind_group,
            msaa_view,
            depth_view,
        }
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn texture_format(&self) -> TextureFormat {
        self.texture.format()
    }
}

impl InnerLayer for WgpuRenderTarget {
    fn dimensions(&self) -> (u32, u32) {
        (self.texture.width(), self.texture.height())
    }
}

impl Debug for WgpuRenderTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderTarget")
            .finish()
    }
}