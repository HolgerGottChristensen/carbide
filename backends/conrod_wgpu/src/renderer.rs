use std::collections::{HashMap, HashSet};
use conrod_core::{mesh::mesh, Scalar, Rect};
use crate::{DEFAULT_IMAGE_TEX_FORMAT, GLYPH_TEX_FORMAT, GLYPH_TEX_COMPONENT_TY};
use crate::image::Image;
use wgpu::util::DeviceExt;
use crate::render::Render;
use conrod_core::text::rt;
use conrod_core::render::primitive_walker::PrimitiveWalker;
use crate::glyph_cache_command::GlyphCacheCommand;
use crate::render_pass_command::RenderPassCommand;
use conrod_core::widget::Id;
use crate::pipeline::Pipeline;
use conrod_core::mesh::mesh::Mesh;
use conrod_core::mesh::vertex::Vertex;

/// A helper type aimed at simplifying the rendering of conrod primitives via wgpu.
pub struct Renderer {
    vs_mod: wgpu::ShaderModule,
    fs_mod: wgpu::ShaderModule,
    glyph_cache_tex: wgpu::Texture,
    _default_image_tex: wgpu::Texture,
    default_bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    mesh: Mesh,
    // The texture format of the output attachment.
    dst_format: wgpu::TextureFormat,
    // The sample count of the output attachment.
    dst_sample_count: u32,
    // In order to support a dynamic number of images we maintain a unique bind group for each.
    bind_groups: HashMap<conrod_core::image::Id, wgpu::BindGroup>,
    // We also need a unique
    render_pipelines: HashMap<wgpu::TextureComponentType, Pipeline>,

    /*
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    */
}

pub fn glyph_cache_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor<'static> {
    let depth = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth,
    };
    wgpu::TextureDescriptor {
        label: Some("conrod_wgpu_glyph_cache_texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: GLYPH_TEX_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    }
}

fn default_image_tex_desc() -> wgpu::TextureDescriptor<'static> {
    let width = 64;
    let height = 64;
    let depth = 1;
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth,
    };
    wgpu::TextureDescriptor {
        label: Some("conrod_wgpu_image_texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEFAULT_IMAGE_TEX_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED,
    }
}

fn sampler_desc() -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("conrod_sample_descriptor"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare: Some(wgpu::CompareFunction::Always),
        anisotropy_clamp: None,
    }
}

fn bind_group_layout(
    device: &wgpu::Device,
    img_tex_component_ty: wgpu::TextureComponentType,
) -> wgpu::BindGroupLayout {
    let glyph_cache_texture_binding = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            component_type: GLYPH_TEX_COMPONENT_TY.into(),
            dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    };
    let sampler_binding = wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::Sampler { comparison: true },
        count: None,
    };
    let image_texture_binding = wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            component_type: img_tex_component_ty,
            dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    };
    let entries = &[
        glyph_cache_texture_binding,
        sampler_binding,
        image_texture_binding,
    ];
    let desc = wgpu::BindGroupLayoutDescriptor {
        label: Some("conrod_bind_group_layout"),
        entries,
    };
    device.create_bind_group_layout(&desc)
}

fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    glyph_cache_tex: &wgpu::Texture,
    sampler: &wgpu::Sampler,
    image: &wgpu::Texture,
) -> wgpu::BindGroup {
    // Glyph cache texture view.
    let glyph_cache_tex_view = glyph_cache_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let glyph_cache_tex_binding = wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::TextureView(&glyph_cache_tex_view),
    };

    // Sampler binding.
    let sampler_binding = wgpu::BindGroupEntry {
        binding: 1,
        resource: wgpu::BindingResource::Sampler(&sampler),
    };

    // Image texture view.
    let image_tex_view = image.create_view(&wgpu::TextureViewDescriptor::default());
    let image_tex_binding = wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::TextureView(&image_tex_view),
    };

    let entries = &[glyph_cache_tex_binding, sampler_binding, image_tex_binding];
    let label = Some("conrod_bind_group");
    let desc = wgpu::BindGroupDescriptor {
        label,
        layout,
        entries,
    };
    device.create_bind_group(&desc)
}

fn pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: Some("conrod_pipeline_layout_descriptor"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

fn render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    dst_sample_count: u32,
) -> wgpu::RenderPipeline {
    let vs_desc = wgpu::ProgrammableStageDescriptor {
        module: &vs_mod,
        entry_point: "main",
    };
    let fs_desc = wgpu::ProgrammableStageDescriptor {
        module: &fs_mod,
        entry_point: "main",
    };
    let raster_desc = wgpu::RasterizationStateDescriptor {
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: wgpu::CullMode::None,
        ..Default::default()
    };
    let color_state_desc = wgpu::ColorStateDescriptor {
        format: dst_format,
        color_blend: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha_blend: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        write_mask: wgpu::ColorWrite::ALL,
    };
    let vertex_buffer_desc = Vertex::desc();

    let vertex_state_desc = wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[vertex_buffer_desc],
    };
    let desc = wgpu::RenderPipelineDescriptor {
        label: Some("conrod_render_pipeline_descriptor"),
        layout: Some(layout),
        vertex_stage: vs_desc,
        fragment_stage: Some(fs_desc),
        rasterization_state: Some(raster_desc),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[color_state_desc],
        depth_stencil_state: None,
        vertex_state: vertex_state_desc,
        sample_count: dst_sample_count,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    };
    device.create_render_pipeline(&desc)
}


impl Renderer {
    /// Construct a new `Renderer`.
    ///
    /// The `dst_sample_count` and `dst_format` refer to the associated properties of the output
    /// attachment to which the `Renderer` will draw. Note that if the `dst_sample_count` or
    /// `dst_format` change at runtime, the `Renderer` should be reconstructed.
    pub fn new(
        device: &wgpu::Device,
        dst_sample_count: u32,
        dst_format: wgpu::TextureFormat,
    ) -> Self {
        let glyph_cache_dims = mesh::DEFAULT_GLYPH_CACHE_DIMS;
        Self::with_glyph_cache_dimensions(device, dst_sample_count, dst_format, glyph_cache_dims)
    }

    /// Create a renderer with a specific size for the glyph cache.
    ///
    /// The `dst_sample_count` and `dst_format` refer to the associated properties of the output
    /// attachment to which the `Renderer` will draw. Note that if the `dst_sample_count` or
    /// `dst_format` change at runtime, the `Renderer` should be reconstructed.
    pub fn with_glyph_cache_dimensions(
        device: &wgpu::Device,
        dst_sample_count: u32,
        dst_format: wgpu::TextureFormat,
        glyph_cache_dims: [u32; 2],
    ) -> Self {
        assert_eq!(
            glyph_cache_dims[0] % 256,
            0,
            "wgpu glyph cache width must be multiple of 256"
        );

        // The mesh for converting primitives into vertices.
        let mesh = Mesh::with_glyph_cache_dimensions(glyph_cache_dims);

        // Load shader modules.
        let vs_mod = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_mod = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

        // Create the glyph cache texture.
        let glyph_cache_tex_desc = glyph_cache_tex_desc(glyph_cache_dims);
        let glyph_cache_tex = device.create_texture(&glyph_cache_tex_desc);

        // Create the default image that is bound to `image_texture` along with a default bind
        // group for use in the case that there are no user supplied images.
        let default_image_tex_desc = default_image_tex_desc();
        let default_image_tex = device.create_texture(&default_image_tex_desc);

        // Create the sampler for sampling from the glyph cache and image textures.
        let sampler_desc = sampler_desc();
        let sampler = device.create_sampler(&sampler_desc);

        // Create at least one render pipeline for the default texture.
        let mut render_pipelines = HashMap::new();

        let default_tex_component_ty = DEFAULT_IMAGE_TEX_FORMAT.into();
        let bind_group_layout = bind_group_layout(device, default_tex_component_ty);
        let pipeline_layout = pipeline_layout(device, &bind_group_layout);
        let render_pipeline = render_pipeline(
            device,
            &pipeline_layout,
            &vs_mod,
            &fs_mod,
            dst_format,
            dst_sample_count,
        );
        let default_bind_group = bind_group(
            device,
            &bind_group_layout,
            &glyph_cache_tex,
            &sampler,
            &default_image_tex,
        );
        let default_pipeline = Pipeline {
            bind_group_layout,
            render_pipeline,
        };
        render_pipelines.insert(default_tex_component_ty, default_pipeline);

        // The empty set of bind groups to be associated with user images.
        let bind_groups = Default::default();

        Self {
            vs_mod,
            fs_mod,
            glyph_cache_tex,
            _default_image_tex: default_image_tex,
            default_bind_group,
            sampler,
            dst_format,
            dst_sample_count,
            bind_groups,
            render_pipelines,
            mesh,
        }
    }

    /// Produce an `Iterator` yielding `Command`s.
    pub fn commands(&self) -> mesh::Commands {
        self.mesh.commands()
    }

    /// Fill the inner vertex and command buffers by translating the given `primitives`.
    ///
    /// This method may return an `Option<GlyphCacheCommand>`, in which case the user should use
    /// the contained `glyph_cpu_buffer_pool` to write the pixel data to the GPU, and then use a
    /// `copy_buffer_to_image` command to write the data to the given `glyph_cache_texture` image.
    pub fn fill<P>(
        &mut self,
        image_map: &conrod_core::image::ImageMap<Image>,
        viewport: [f32; 4],
        scale_factor: f64,
        primitives: P,
    ) -> Result<Option<GlyphCacheCommand>, rt::gpu_cache::CacheWriteErr>
        where
            P: PrimitiveWalker,
    {
        // Convert the given primitives into vertices.
        let [vp_l, vp_t, vp_r, vp_b] = viewport;
        let lt = [vp_l as Scalar, vp_t as Scalar];
        let rb = [vp_r as Scalar, vp_b as Scalar];
        let viewport = Rect::from_corners(lt, rb);

        // Convert the mesh into a fill
        let fill = self
            .mesh
            .fill(viewport, scale_factor, image_map, primitives)?;

        // Check whether or not we need a glyph cache update.
        let glyph_cache_cmd = match fill.glyph_cache_requires_upload {
            false => None,
            true => {
                let (width, height) = self.mesh.glyph_cache().dimensions();
                Some(GlyphCacheCommand {
                    glyph_cache_pixel_buffer: self.mesh.glyph_cache_pixel_buffer(),
                    glyph_cache_texture: &self.glyph_cache_tex,
                    width,
                    height,
                })
            }
        };
        Ok(glyph_cache_cmd)
    }

    /// Converts the inner list of `Command`s generated via `fill` to a list of
    /// `RenderPassCommand`s that are easily digestible by a `wgpu::RenderPass` produced by a
    /// `wgpu::CommandEncoder`.
    pub fn render(&mut self, device: &wgpu::Device, image_map: &conrod_core::image::ImageMap<Image>) -> Render {
        let Renderer {
            ref mut bind_groups,
            ref mut render_pipelines,
            ref mut mesh,
            ref vs_mod,
            ref fs_mod,
            ref default_bind_group,
            ref glyph_cache_tex,
            ref sampler,
            dst_format,
            dst_sample_count,
            ..
        } = *self;

        let mut commands = vec![];

        // Ensure we have:
        // - a bind group layout and render pipeline for each unique texture component type.
        // - a bind group ready for each image in the map.
        let default_tct = DEFAULT_IMAGE_TEX_FORMAT.into();
        let unique_tex_component_types: HashSet<_> = image_map
            .values()
            .map(|img| img.texture_component_type())
            .chain(Some(default_tct))
            .collect();
        bind_groups.retain(|k, _| image_map.contains_key(k));
        render_pipelines.retain(|tct, _| unique_tex_component_types.contains(tct));
        for (id, img) in image_map.iter() {
            // If we already have a bind group for this image move on.
            if bind_groups.contains_key(id) {
                continue;
            }

            // Retrieve the bind group layout and pipeline for the image's texture component type.
            let tct = img.texture_component_type();
            let pipeline = render_pipelines.entry(tct).or_insert_with(|| {
                let bind_group_layout = bind_group_layout(device, tct);
                let pipeline_layout = pipeline_layout(device, &bind_group_layout);
                let render_pipeline = render_pipeline(
                    device,
                    &pipeline_layout,
                    vs_mod,
                    fs_mod,
                    dst_format,
                    dst_sample_count,
                );
                Pipeline {
                    bind_group_layout,
                    render_pipeline,
                }
            });

            // Create the bind
            let bind_group = bind_group(
                device,
                &pipeline.bind_group_layout,
                &glyph_cache_tex,
                sampler,
                &img.texture.texture,
            );
            bind_groups.insert(*id, bind_group);
        }

        // Prepare a single vertex buffer containing all vertices for all geometry.
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(mesh.vertices()),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        // Keep track of the currently set bind group.
        #[derive(PartialEq)]
        enum BindGroup {
            Default,
            Image(conrod_core::image::Id),
        }
        let mut bind_group = None;

        for command in mesh.commands() {
            match command {
                // Update the `scizzor` before continuing to draw.
                mesh::Command::Scizzor(s) => {
                    let top_left = [s.top_left[0] as u32, s.top_left[1] as u32];
                    let dimensions = s.dimensions;
                    let cmd = RenderPassCommand::SetScissor {
                        top_left,
                        dimensions,
                    };
                    commands.push(cmd);
                }

                // Draw to the target with the given `draw` command.
                mesh::Command::Draw(draw) => match draw {
                    // Draw text and plain 2D geometry.
                    mesh::Draw::Plain(vertex_range) => {
                        let vertex_count = vertex_range.len();
                        if vertex_count <= 0 {
                            continue;
                        }
                        // Ensure a render pipeline and bind group is set.
                        if bind_group.is_none() {
                            bind_group = Some(BindGroup::Default);
                            let cmd = RenderPassCommand::SetBindGroup {
                                bind_group: default_bind_group,
                            };
                            commands.push(cmd);
                        }
                        let cmd = RenderPassCommand::Draw {
                            vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                        };
                        commands.push(cmd);
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    mesh::Draw::Image(image_id, vertex_range) => {
                        let vertex_count = vertex_range.len();
                        if vertex_count == 0 {
                            continue;
                        }
                        // Ensure the bind group matches this image.
                        let expected_bind_group = Some(BindGroup::Image(image_id));
                        if bind_group != expected_bind_group {

                            // Now update the bind group and add the new bind group command.
                            bind_group = expected_bind_group;
                            let cmd = RenderPassCommand::SetBindGroup {
                                bind_group: &bind_groups[&image_id],
                            };
                            commands.push(cmd);
                        }
                        let cmd = RenderPassCommand::Draw {
                            vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                        };
                        commands.push(cmd);
                    }
                },
            }
        }

        Render {
            vertex_buffer,
            commands,
        }
    }
}
