use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureUsages};
use carbide_core::environment::Environment;
use carbide_wgpu_core::{WgpuContextNew, WgpuRenderTarget, RENDER_TARGET_FORMAT};
use crate::textures::{create_depth_stencil_texture_view, create_msaa_texture_view};
use crate::wgpu_context::WgpuContext;
use crate::wgpu_msaa::WgpuMsaa;

/// Create a render target of the given size, sampled by the 2D renderer through its texture bind
/// group layout and main sampler, with its own MSAA and depth/stencil attachments.
pub(crate) fn create_render_target(width: u32, height: u32, msaa: WgpuMsaa, env: &mut Environment) -> WgpuRenderTarget {
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

    let (wgpu_context, wgpu_context_new) = env.get_many::<(WgpuContext, WgpuContextNew)>().unwrap();

    let texture = wgpu_context_new.device.create_texture(&descriptor);

    let view = texture.create_view(&Default::default());

    let bind_group = wgpu_context_new.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

    let msaa_view = create_msaa_texture_view(&wgpu_context_new.device, width, height, msaa);
    let depth_view = create_depth_stencil_texture_view(&wgpu_context_new.device, width, height, msaa);

    WgpuRenderTarget::from_parts(texture, view, bind_group, msaa_view, depth_view)
}
