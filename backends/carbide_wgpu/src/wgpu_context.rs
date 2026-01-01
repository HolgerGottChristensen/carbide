use std::collections::HashMap;
use dashmap::DashMap;
use log::{error, info};
use wgpu::{Adapter, BindGroup, BindGroupLayout, Device, Features, Instance, PipelineLayout, Queue, RenderPipeline, Sampler, ShaderModule, Texture, TextureFormat};
use carbide_core::draw::ImageId;
use carbide_core::environment::EnvironmentKey;
use carbide_core::widget::FilterId;
use crate::bind_group_layouts::{atlas_bind_group_layout, filter_buffer_bind_group_layout, filter_texture_bind_group_layout, gradient_buffer_bind_group_layout, texture_bind_group_layout, uniform_bind_group_layout, uniform_bind_group_layout2};
use crate::bind_groups::create_bind_groups;
use crate::image_context::BindGroupExtended;
use crate::pipeline::{filter_pipeline_layout, main_pipeline_layout, RenderPipelines};
use crate::samplers::main_sampler;
use crate::textures::{create_atlas_cache_bind_group, create_atlas_cache_texture};

#[derive(Debug)]
pub struct WgpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,

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
    pub async fn new() -> WgpuContext {
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            backend_options: Default::default(),
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }).await.unwrap();

        let mut limits = wgpu::Limits::default();
        limits.max_bind_groups = 5;

        info!("{:#?}", adapter.limits());

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("carbide_device"),
                // Required for multiple layer textures, where we clear the same allocated texture using intermediate steps.
                required_features: Features::CLEAR_TEXTURE,
                required_limits: limits,
                memory_hints: Default::default(),
                trace: Default::default(),
            }
        ).await.unwrap();

        info!("{:#?}", device.limits());
        info!("{:#?}", device.features());

        let main_sampler = main_sampler(&device);

        let texture_bind_group_layout = texture_bind_group_layout(&device);

        let bind_groups = create_bind_groups(&device, &queue, &texture_bind_group_layout);

        let main_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));
        let filter_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/filter.wgsl"));

        let atlas_cache_texture = create_atlas_cache_texture(&device, 1024, 1024);
        let atlas_cache_bind_group_layout = atlas_bind_group_layout(&device);
        let atlas_cache_bind_group = create_atlas_cache_bind_group(&device, &atlas_cache_texture, &atlas_cache_bind_group_layout);

        let uniform_bind_group_layout = uniform_bind_group_layout(&device);
        let uniform_bind_group_layout2 = uniform_bind_group_layout2(&device);
        let filter_texture_bind_group_layout = filter_texture_bind_group_layout(&device);
        let filter_buffer_bind_group_layout = filter_buffer_bind_group_layout(&device);
        let gradient_buffer_bind_group_layout = gradient_buffer_bind_group_layout(&device);

        let main_pipeline_layout = main_pipeline_layout(
            &device,
            &texture_bind_group_layout,
            &uniform_bind_group_layout,
            &gradient_buffer_bind_group_layout,
            &atlas_cache_bind_group_layout,
        );

        let filter_pipeline_layout = filter_pipeline_layout(
            &device,
            &filter_texture_bind_group_layout,
            &filter_buffer_bind_group_layout,
            &uniform_bind_group_layout,
        );

        let final_render_shader_linear = device.create_shader_module(wgpu::include_wgsl!("../shaders/final_linear.wgsl"));
        let final_render_shader_srgb = device.create_shader_module(wgpu::include_wgsl!("../shaders/final_srgb.wgsl"));

        WgpuContext {
            instance,
            adapter,
            device,
            queue,
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
}