use std::any::TypeId;
use std::collections::HashMap;
use std::pin::Pin;
use dashmap::DashMap;
use wgpu::{BindGroup, BindGroupLayout, PipelineLayout, Sampler, ShaderModule, Texture, TextureFormat};
use carbide::environment::{Environment, Exclusive};
use carbide_core::draw::ImageId;
use carbide_core::environment::EnvironmentKey;
use carbide_core::plugin::Plugin;
use carbide_core::register;
use carbide_core::widget::FilterId;
use carbide_wgpu_core::WgpuContextNew;
use crate::bind_group_layouts::{atlas_bind_group_layout, filter_buffer_bind_group_layout, filter_texture_bind_group_layout, gradient_buffer_bind_group_layout, texture_bind_group_layout, uniform_bind_group_layout, uniform_bind_group_layout2};
use crate::bind_groups::create_bind_groups;
use crate::image_context::BindGroupExtended;
use crate::pipeline::{filter_pipeline_layout, main_pipeline_layout, RenderPipelines};
use crate::samplers::main_sampler;
use crate::textures::{create_atlas_cache_bind_group, create_atlas_cache_texture};

#[derive(Debug)]
pub struct WgpuContext {
    pub main_sampler: Sampler,
    pub main_shader: ShaderModule,
    pub filter_shader: ShaderModule,
    pub final_render_shader_srgb: ShaderModule,
    pub final_render_shader_linear: ShaderModule,

    pub texture_bind_group_layout: BindGroupLayout,
    pub uniform_bind_group_layout: BindGroupLayout,
    pub uniform_bind_group_layout2: BindGroupLayout,
    pub filter_texture_bind_group_layout: BindGroupLayout,
    pub filter_buffer_bind_group_layout: BindGroupLayout,
    pub gradient_buffer_bind_group_layout: BindGroupLayout,

    pub main_pipeline_layout: PipelineLayout,
    pub filter_pipeline_layout: PipelineLayout,

    pub bind_groups: HashMap<ImageId, BindGroupExtended>,
    pub filter_bind_groups: HashMap<FilterId, BindGroup>,

    pub pipelines: DashMap<TextureFormat, RenderPipelines>,

    pub atlas_cache_texture: Texture,
    pub atlas_cache_bind_group_layout: BindGroupLayout,
    pub atlas_cache_bind_group: BindGroup,
}

impl WgpuContext {
    pub async fn new(env: &mut Environment<'_>) -> WgpuContext {
        let context = env.get::<WgpuContextNew>().unwrap();

        let main_sampler = main_sampler(&context.device);

        let texture_bind_group_layout = texture_bind_group_layout(&context.device);

        let bind_groups = create_bind_groups(&context.device, &context.queue, &texture_bind_group_layout);

        let main_shader = context.device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));
        let filter_shader = context.device.create_shader_module(wgpu::include_wgsl!("../shaders/filter.wgsl"));

        let atlas_cache_texture = create_atlas_cache_texture(&context.device, 1024, 1024);
        let atlas_cache_bind_group_layout = atlas_bind_group_layout(&context.device);
        let atlas_cache_bind_group = create_atlas_cache_bind_group(&context.device, &atlas_cache_texture, &atlas_cache_bind_group_layout);

        let uniform_bind_group_layout = uniform_bind_group_layout(&context.device);
        let uniform_bind_group_layout2 = uniform_bind_group_layout2(&context.device);
        let filter_texture_bind_group_layout = filter_texture_bind_group_layout(&context.device);
        let filter_buffer_bind_group_layout = filter_buffer_bind_group_layout(&context.device);
        let gradient_buffer_bind_group_layout = gradient_buffer_bind_group_layout(&context.device);

        let main_pipeline_layout = main_pipeline_layout(
            &context.device,
            &texture_bind_group_layout,
            &uniform_bind_group_layout,
            &gradient_buffer_bind_group_layout,
            &atlas_cache_bind_group_layout,
        );

        let filter_pipeline_layout = filter_pipeline_layout(
            &context.device,
            &filter_texture_bind_group_layout,
            &filter_buffer_bind_group_layout,
            &uniform_bind_group_layout,
        );

        let final_render_shader_linear = context.device.create_shader_module(wgpu::include_wgsl!("../shaders/final_linear.wgsl"));
        let final_render_shader_srgb = context.device.create_shader_module(wgpu::include_wgsl!("../shaders/final_srgb.wgsl"));

        WgpuContext {
            main_sampler,
            main_shader,
            filter_shader,
            final_render_shader_srgb,
            final_render_shader_linear,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            uniform_bind_group_layout2,
            filter_texture_bind_group_layout,
            filter_buffer_bind_group_layout,
            gradient_buffer_bind_group_layout,
            main_pipeline_layout,
            filter_pipeline_layout,
            bind_groups,
            filter_bind_groups: HashMap::new(),
            pipelines: DashMap::new(),
            atlas_cache_texture,
            atlas_cache_bind_group_layout,
            atlas_cache_bind_group,
        }
    }
}

impl EnvironmentKey for WgpuContext {
    type Value = WgpuContext;
    type Access = Exclusive;
}

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        let context = WgpuContext::new(env).await;

        env.insert::<WgpuContext>(Box::leak(Box::new(context)))
    })
}

register!(Plugin {
    // `WgpuContext::new` builds on the device and queue from `WgpuContextNew`.
    dependencies: &[TypeId::of::<WgpuContextNew>()],
    registers: TypeId::of::<WgpuContext>(),
    register
});
