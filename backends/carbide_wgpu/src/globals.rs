use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::RwLock;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use wgpu::{BindGroup, PipelineLayout, Sampler, ShaderModule, Texture, TextureFormat};
use carbide_core::draw;
use carbide_core::draw::ImageId;
use carbide_core::widget::FilterId;
use crate::bind_group_layouts::{ATLAS_BIND_GROUP_LAYOUT, FILTER_BUFFER_BIND_GROUP_LAYOUT, FILTER_TEXTURE_BIND_GROUP_LAYOUT, GRADIENT_DASHES_BIND_GROUP_LAYOUT, MAIN_TEXTURE_BIND_GROUP_LAYOUT, UNIFORM_BIND_GROUP_LAYOUT};
use crate::DEVICE;
use crate::image_context::{create_bind_group, BindGroupExtended};
use crate::pipeline::{filter_pipeline_layout, main_pipeline_layout, RenderPipelines};
use crate::samplers::main_sampler;
use crate::textures::create_atlas_cache_texture;

pub(crate) static MAIN_SHADER: Lazy<ShaderModule> = Lazy::new(|| {
    DEVICE.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"))
});

pub(crate) static FILTER_SHADER: Lazy<ShaderModule> = Lazy::new(|| {
    DEVICE.create_shader_module(wgpu::include_wgsl!("../shaders/filter.wgsl"))
});

pub(crate) static RENDER_PIPELINE_LAYOUT: Lazy<PipelineLayout> = Lazy::new(|| {
    main_pipeline_layout(
        &DEVICE,
        &MAIN_TEXTURE_BIND_GROUP_LAYOUT,
        &UNIFORM_BIND_GROUP_LAYOUT,
        &GRADIENT_DASHES_BIND_GROUP_LAYOUT,
        &ATLAS_BIND_GROUP_LAYOUT,
    )
});

pub(crate) static FILTER_RENDER_PIPELINE_LAYOUT: Lazy<PipelineLayout> = Lazy::new(|| {
    filter_pipeline_layout(
        &DEVICE,
        &FILTER_TEXTURE_BIND_GROUP_LAYOUT,
        &FILTER_BUFFER_BIND_GROUP_LAYOUT,
        &UNIFORM_BIND_GROUP_LAYOUT,
    )
});

pub(crate) static MAIN_SAMPLER: Lazy<Sampler> = Lazy::new(|| {
    main_sampler(&DEVICE)
});

pub(crate) static ATLAS_CACHE_TEXTURE: Lazy<Texture> = Lazy::new(|| {
    create_atlas_cache_texture(&DEVICE, 1024, 1024)
});

pub(crate) static ATLAS_CACHE_BIND_GROUP: Lazy<BindGroup> = Lazy::new(|| {
    let view = ATLAS_CACHE_TEXTURE.create_view(&Default::default());

    DEVICE.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &ATLAS_BIND_GROUP_LAYOUT,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
        ],
        label: None,
    })
});

pub(crate) static PIPELINES: Lazy<DashMap<TextureFormat, RenderPipelines>> = Lazy::new(|| {
    DashMap::new()
});

thread_local!(pub static BIND_GROUPS: RefCell<HashMap<ImageId, BindGroupExtended>> = {
    let mut map = HashMap::new();

    let texture = draw::Texture {
        width: 1,
        height: 1,
        bytes_per_row: 4,
        format: draw::TextureFormat::RGBA8,
        data: &[0u8, 0u8, 0u8, 255u8],
    };

    let bind_group = create_bind_group(texture);

    map.insert(ImageId::default(), bind_group);
    RefCell::new(map)
});

pub(crate) static FILTER_BIND_GROUPS: Lazy<RwLock<HashMap<FilterId, BindGroup>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});