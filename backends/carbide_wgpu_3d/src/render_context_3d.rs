use std::collections::HashMap;

use wgpu::{AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType, ColorTargetState, CompareFunction, DepthBiasState, Extent3d, TextureDimension, TextureUsages, TextureViewDescriptor, TextureViewDimension, DepthStencilState, Device, Face, FilterMode, FragmentState, FrontFace, IndexFormat, Operations, PipelineLayout, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassDepthStencilAttachment, RenderPipeline, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderStages, StencilState, StoreOp, Texture, TextureFormat, TextureView, VertexState};

use carbide_3d::camera::CameraSpec;
use carbide_3d::material::Material;
use carbide_3d::render::{InnerRenderContext3d, LightShadow, MeshSettings, PickSample, Tonemapping, ViewSpec};
use carbide_3d::Mesh;
use carbide_core::color::{Color, ColorExt, WHITE};
use carbide_core::environment::Environment;
use carbide_core::math::{Deg, InnerSpace, Matrix4, Rad, SquareMatrix, Vector3, Vector4};
use carbide_core::render::{InnerLayer, Layer};
use carbide_core::scene::SceneManager;
use carbide_wgpu_core::{WgpuContextNew, WgpuRenderTarget, WgpuTextureStore};

use crate::camera::WgpuCamera;
use crate::directional_light::{WgpuDirectionalLight, WgpuDirectionalLightBuffer};
use crate::material::WgpuMaterial;
use crate::object::WgpuObject;
use crate::outline::OutlineState;
use crate::picking::PickingState;
use crate::pbr_material::{create_pbr_bind_group, create_pbr_bind_group_layout, FallbackTextures, WgpuPbrMaterial, WgpuPbrMaterialTextures, SLOT_COUNT};
use crate::point_light::{WgpuPunctualLight, WgpuPunctualLightBuffer};
use crate::preprocess_shader;
use crate::render_pass_command::{blend_variant, pipeline_variant, RenderPassCommand, BLEND_VARIANTS, PIPELINE_VARIANTS};
use crate::shadow::{directional_view_proj, layer_size, spot_view_proj, ShadowRequest, ShadowTarget, SHADOW_FORMAT};
use crate::storage_buffer::StorageBuffer;
use crate::uniforms::WgpuUniforms;
use crate::mesh_cache::MeshCache;
use crate::vertex::WgpuVertex;

const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

/// Samples per pixel when drawing, for anti-aliased edges. The multisampled picture is resolved
/// into the layer. WebGPU guarantees 4 for the layer's colour format and the depth format.
const SAMPLE_COUNT: u32 = 4;

/// Renders a `Scene3d` into its layer with wgpu.
///
/// The nodes' calls during a frame build a draw list: the materials, objects (a transform, a
/// material and whether shadows fall on it), the meshes to draw, the lights and the shadow maps
/// they ask for. [`render`](InnerRenderContext3d::render) uploads the list, draws a depth-only
/// pass into each shadow map, and then draws the scene in one render pass into the layer. Meshes'
/// GPU copies live in the shared [`MeshCache`]; the GPU state below is kept between frames.
#[derive(Debug)]
pub struct WgpuRenderContext3d {
    material_stack: Vec<(WgpuMaterial, WgpuPbrMaterialTextures)>,
    materials: Vec<(WgpuMaterial, WgpuPbrMaterialTextures)>,
    current_material: Option<(WgpuMaterial, WgpuPbrMaterialTextures, usize)>,

    transform_stack: Vec<Matrix4<f32>>,
    objects: Vec<WgpuObject>,

    directional_lights: Vec<WgpuDirectionalLight>,
    /// Point and spot lights.
    punctual_lights: Vec<WgpuPunctualLight>,
    /// The scene's hemisphere light: sky, ground and intensity, as last given.
    hemisphere: (Color, Color, f32),
    /// The shadow maps to draw, one per light that casts shadows, in layer order.
    shadow_requests: Vec<ShadowRequest>,

    /// Used when the environment has no shared [`MeshCache`], for example when the context was
    /// created without the plugin.
    fallback_mesh_cache: Option<MeshCache>,

    commands: Vec<RenderPassCommand>,


    /// Created on the first render, when the device is available.
    gpu: Option<GpuState>,
}

/// The GPU objects a render context keeps between frames.
#[derive(Debug)]
struct GpuState {
    shader: ShaderModule,
    /// Materials, objects and camera.
    scene_layout: BindGroupLayout,
    /// Lights, sampler and uniforms.
    lighting_layout: BindGroupLayout,
    /// A material's textures.
    pbr_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    /// The colour format the pipelines were created for, and the pipelines in `pipeline_variant`
    /// order: culling back faces or not (double-sided), with counter-clockwise or clockwise
    /// (mirrored) front faces.
    pipeline: Option<(TextureFormat, [RenderPipeline; 4])>,
    /// The blending pipelines, for blended materials, in `blend_variant` order: they don't write
    /// depth.
    blend_pipelines: Option<(TextureFormat, [RenderPipeline; 4])>,
    /// Repeating, linear and anisotropic: for textures, and the default for materials.
    sampler: Sampler,
    /// Repeating and nearest: for materials with `SampleType::Nearest`.
    nearest_sampler: Sampler,
    depth_texture: Option<Texture>,
    depth_view: Option<TextureView>,
    /// The multisampled colour target, resolved into the layer, with its size and format.
    multisampled: Option<((u32, u32, TextureFormat), TextureView)>,
    /// A bind group per set of material images, with the stored versions of the images it was made
    /// from: it is made again when an image loads or is replaced.
    texture_bind_groups: HashMap<WgpuPbrMaterialTextures, ([u64; SLOT_COUNT], BindGroup)>,
    /// Created on the first frame, with the queue.
    fallback_textures: Option<FallbackTextures>,

    /// The objects and a light's view-projection, for the shadow pass.
    shadow_layout: BindGroupLayout,
    /// Draws casting meshes' depth into a shadow map.
    shadow_pipeline: RenderPipeline,
    /// Compares depths in the shadow maps, `GreaterEqual` for reverse-Z.
    shadow_sampler: Sampler,
    /// The shadow maps, one layer per light, with their size and layer count: a single texel when
    /// no light casts shadows, since the main pass always binds them.
    shadow_maps: Option<((u32, u32), Texture, TextureView)>,

    /// The outline passes, created on the first frame with something outlined.
    outline: Option<OutlineState>,
    /// The picking pass and the last frame's picking buffer, created on the first frame that
    /// asks for one.
    picking: Option<PickingState>,
}

impl WgpuRenderContext3d {
    pub fn new() -> Self {
        WgpuRenderContext3d {
            material_stack: vec![],
            materials: vec![],
            current_material: None,
            transform_stack: vec![Matrix4::identity()],
            objects: vec![],
            directional_lights: vec![],
            punctual_lights: vec![],
            hemisphere: (WHITE, WHITE, 0.1),
            shadow_requests: vec![],
            fallback_mesh_cache: None,
            commands: vec![],
                    gpu: None,
        }
    }

    fn clear(&mut self) {
        self.material_stack.clear();
        self.current_material = None;
        self.materials.clear();

        self.transform_stack.clear();
        self.transform_stack.push(Matrix4::identity());
        self.objects.clear();
        self.directional_lights.clear();
        self.punctual_lights.clear();
        self.shadow_requests.clear();

        self.commands.clear();
    }

    fn ensure_current_object(&mut self, receives_shadows: bool, outline: Vector4<f32>, pick_id: u32) {
        let object = WgpuObject {
            transform: *self.transform_stack.last().unwrap(),
            material_index: self.current_material.as_ref().unwrap().2 as u32,
            receives_shadows: receives_shadows as u32,
            pick_id,
            outline,
        };

        if self.objects.last() != Some(&object) {
            self.objects.push(object);
        }
    }

    fn ensure_current_material(&mut self, material: &WgpuMaterial, tex: &WgpuPbrMaterialTextures) {
        let needs_update = if let Some((current, _, _)) = &self.current_material {
            current != material
        } else {
            true
        };

        if needs_update {
            self.materials.push((material.clone(), tex.clone()));
            self.current_material = Some((material.clone(), tex.clone(), self.materials.len() - 1));
            self.commands.push(RenderPassCommand::SetPbrMaterial(tex.clone()))
        }
    }
}

impl WgpuRenderContext3d {
    /// A point or spot light at the origin of the current transform, shining along its -Z. A spot
    /// light that casts shadows gives its outer angle with its shadow settings.
    fn punctual(&mut self, color: Color, intensity: f32, range: Option<f32>, cone_scale: f32, cone_offset: f32, shadow: Option<(LightShadow, Deg<f32>)>) {
        let transform = *self.transform_stack.last().unwrap();
        let position = transform * Vector4::new(0.0, 0.0, 0.0, 1.0);
        let direction = (transform * Vector4::new(0.0, 0.0, -1.0, 0.0)).truncate();
        let direction = if direction.magnitude2() > f32::EPSILON { direction.normalize() } else { -Vector3::unit_z() };

        let (view_proj, shadow_layer, shadow_bias) = match shadow {
            Some((settings, outer)) => {
                let layer = self.request_shadow(ShadowTarget::Punctual { index: self.punctual_lights.len() }, settings);
                (spot_view_proj(position.truncate(), direction, outer), layer, settings.bias)
            }
            None => (Matrix4::identity(), -1, 0.0),
        };

        self.punctual_lights.push(WgpuPunctualLight {
            view_proj,
            position,
            direction: direction.extend(0.0),
            color: linear(color) * intensity,
            range: range.unwrap_or(0.0).max(0.0),
            cone_scale,
            cone_offset,
            shadow_layer,
            shadow_bias,
            // Set when rendering, once the layer size is known.
            shadow_scale: 1.0,
        });
    }

    /// Ask for a shadow map for a light, and return its layer.
    fn request_shadow(&mut self, target: ShadowTarget, settings: LightShadow) -> i32 {
        self.shadow_requests.push(ShadowRequest { target, settings });
        self.shadow_requests.len() as i32 - 1
    }
}

/// `color` converted from sRGB to linear, as the 2D renderer converts colours, without alpha.
fn linear(color: Color) -> Vector3<f32> {
    let [r, g, b, _] = color.gamma_srgb_to_linear().to_fsa();
    Vector3::new(r, g, b)
}

impl Default for WgpuRenderContext3d {
    fn default() -> Self {
        Self::new()
    }
}

/// A clone is a new render context: it starts with an empty draw list, and creates its own GPU
/// state on its first render.
impl Clone for WgpuRenderContext3d {
    fn clone(&self) -> Self {
        WgpuRenderContext3d::new()
    }
}

impl GpuState {
    fn new(device: &Device) -> GpuState {
        let shader = device.create_shader_module(preprocess_shader("pbr/pbr.wgsl"));

        let storage = |binding: u32| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let scene_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("carbide_3d_scene_layout"),
            entries: &[storage(0), storage(1), storage(2)],
        });

        let lighting_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("carbide_3d_lighting_layout"),
            entries: &[
                storage(0),
                storage(1),
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                storage(3),
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Comparison),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pbr_layout = create_pbr_bind_group_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("carbide_3d_pipeline_layout"),
            bind_group_layouts: &[&scene_layout, &lighting_layout, &pbr_layout],
            push_constant_ranges: &[],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("carbide_3d_linear_sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 16,
            border_color: None,
        });

        let nearest_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("carbide_3d_nearest_sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let shadow_shader = device.create_shader_module(preprocess_shader("pbr/shadow.wgsl"));

        let vertex_storage = |binding: u32| BindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX,
            ..storage(binding)
        };

        let fragment_sampler = |binding: u32| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };

        // Objects and the light, for placing meshes; materials and samplers, for alpha-testing
        // cutout and blended materials.
        let shadow_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("carbide_3d_shadow_layout"),
            entries: &[
                vertex_storage(0),
                vertex_storage(1),
                BindGroupLayoutEntry { visibility: ShaderStages::FRAGMENT, ..storage(2) },
                fragment_sampler(3),
                fragment_sampler(4),
            ],
        });

        let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("carbide_3d_shadow_pipeline_layout"),
            // The material textures as the main pass binds them, for the albedo's alpha.
            bind_group_layouts: &[&shadow_layout, &pbr_layout],
            push_constant_ranges: &[],
        });

        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("carbide_3d_shadow_pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: VertexState {
                module: &shadow_shader,
                entry_point: Some("shadow_vs"),
                compilation_options: Default::default(),
                buffers: &[WgpuVertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                // Both sides, so open meshes such as planes cast shadows from either side.
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: SHADOW_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState::default(),
                // Pushed away from the light (smaller, in reverse-Z) on slopes, against acne on
                // surfaces at grazing angles. The normal offset in the lookup does the rest.
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: -1.5,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            // Depth only; the fragment stage discards what alpha-testing leaves out.
            fragment: Some(FragmentState {
                module: &shadow_shader,
                entry_point: Some("shadow_fs"),
                compilation_options: Default::default(),
                targets: &[],
            }),
            multiview: None,
            cache: None,
        });

        let shadow_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("carbide_3d_shadow_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(CompareFunction::GreaterEqual),
            anisotropy_clamp: 1,
            border_color: None,
        });

        GpuState {
            shadow_layout,
            shadow_pipeline,
            shadow_sampler,
            shadow_maps: None,
            outline: None,
            picking: None,
            shader,
            scene_layout,
            lighting_layout,
            pbr_layout,
            pipeline_layout,
            pipeline: None,
            blend_pipelines: None,
            sampler,
            depth_texture: None,
            depth_view: None,
            multisampled: None,
            texture_bind_groups: HashMap::new(),
            fallback_textures: None,
            nearest_sampler,
        }
    }

    /// The depth attachment, recreated when the layer's size changes.
    fn depth_view(&mut self, device: &Device, width: u32, height: u32) -> &TextureView {
        let matches = self.depth_texture.as_ref()
            .is_some_and(|texture| texture.width() == width && texture.height() == height);

        if !matches {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("carbide_3d_depth_texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.depth_view = Some(texture.create_view(&Default::default()));
            self.depth_texture = Some(texture);
        }

        self.depth_view.as_ref().unwrap()
    }

    /// The multisampled colour target the frame is drawn into, recreated when the layer's size or
    /// format changes.
    fn multisampled_view(&mut self, device: &Device, width: u32, height: u32, format: TextureFormat) -> &TextureView {
        let key = (width, height, format);

        if self.multisampled.as_ref().map(|(current, _)| *current) != Some(key) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("carbide_3d_multisampled_texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.multisampled = Some((key, texture.create_view(&Default::default())));
        }

        &self.multisampled.as_ref().unwrap().1
    }

    /// The shadow maps, recreated when the layer size or count changes: `layers` layers of `size`
    /// by `size` texels.
    fn shadow_maps(&mut self, device: &Device, size: u32, layers: u32) -> &Texture {
        if self.shadow_maps.as_ref().map(|(key, _, _)| *key) != Some((size, layers)) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("carbide_3d_shadow_maps"),
                size: Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: SHADOW_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            // An array view even with one layer, as the shader reads an array.
            let view = texture.create_view(&TextureViewDescriptor {
                label: Some("carbide_3d_shadow_maps_view"),
                dimension: Some(TextureViewDimension::D2Array),
                ..Default::default()
            });

            self.shadow_maps = Some(((size, layers), texture, view));
        }

        &self.shadow_maps.as_ref().unwrap().1
    }

    /// Create the pipeline for `format` if it doesn't exist yet.
    fn ensure_pipeline(&mut self, device: &Device, format: TextureFormat) {
        if self.pipeline.as_ref().is_some_and(|(current, _)| *current == format) {
            return;
        }

        let pipelines = PIPELINE_VARIANTS.map(|(cull_mode, front_face)| self.create_pipeline(device, format, cull_mode, front_face, true));
        self.pipeline = Some((format, pipelines));
    }

    /// Create the blending pipelines for `format` if they don't exist yet: only once a blended
    /// mesh is drawn.
    fn ensure_blend_pipelines(&mut self, device: &Device, format: TextureFormat) {
        if self.blend_pipelines.as_ref().is_some_and(|(current, _)| *current == format) {
            return;
        }

        let pipelines = BLEND_VARIANTS.map(|(cull_mode, front_face)| self.create_pipeline(device, format, cull_mode, front_face, false));
        self.blend_pipelines = Some((format, pipelines));
    }

    /// The PBR pipeline for `format`, culling `cull_mode`, with `front_face` facing the viewer,
    /// and writing depth or not.
    fn create_pipeline(&self, device: &Device, format: TextureFormat, cull_mode: Option<Face>, front_face: FrontFace, depth_write: bool) -> RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("carbide_3d_pbr_pipeline"),
            layout: Some(&self.pipeline_layout),
            vertex: VertexState {
                module: &self.shader,
                entry_point: Some("main_vs"),
                compilation_options: Default::default(),
                buffers: &[WgpuVertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // Right-handed, as OBJ and glTF: counter-clockwise triangles face the viewer,
                // unless a mirroring transform turned them clockwise.
                front_face,
                cull_mode,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            // Reverse-Z: depth is cleared to 0, and nearer fragments have greater depth.
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: depth_write,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &self.shader,
                entry_point: Some("main_fs"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        })
    }
}

impl InnerRenderContext3d for WgpuRenderContext3d {
    fn start(&mut self) {
        self.clear();
    }

    fn render(&mut self, layer: Layer, camera: CameraSpec, view: ViewSpec, env: &mut Environment) {
        // A layer from another 2D renderer, or no device: nothing this context can draw into.
        let Some(render_target) = layer.inner.downcast_ref::<WgpuRenderTarget>() else {
            return;
        };
        let Some(wgpu_context) = env.get::<WgpuContextNew>() else {
            return;
        };
        // Cheap handles, so the environment is free for the mesh cache below.
        let device = &wgpu_context.device.clone();
        let queue = &wgpu_context.queue.clone();

        // Outline thicknesses are in logical pixels; the layer is in physical ones.
        let scale_factor = env.get_mut::<SceneManager>().map(|manager| manager.scale_factor()).unwrap_or(1.0) as f32;

        // The materials' textures, from the store shared with 2D. A bind group is made for each set
        // of images, and made again when one of them loads or is replaced. Slots without a loaded
        // image get fallback textures, and are not sampled.
        let texture_enable = {
            let store = env.get::<WgpuTextureStore>();
            let gpu = self.gpu.get_or_insert_with(|| GpuState::new(device));
            let fallback = gpu.fallback_textures.get_or_insert_with(|| FallbackTextures::new(device, queue));

            for (_, textures) in &self.materials {
                let versions = textures.versions(store);
                let current = gpu.texture_bind_groups.get(textures).is_some_and(|(made_from, _)| *made_from == versions);
                if !current {
                    let bind_group = create_pbr_bind_group(textures, store, fallback, &gpu.pbr_layout, device);
                    gpu.texture_bind_groups.insert(textures.clone(), (versions, bind_group));
                }
            }

            self.materials.iter().map(|(_, textures)| textures.enabled(store)).collect::<Vec<_>>()
        };

        // The meshes' GPU copies, shared by every view; uploaded once, freed once dropped.
        let meshes = match env.get_mut::<MeshCache>() {
            Some(cache) => cache,
            None => self.fallback_mesh_cache.get_or_insert_with(MeshCache::new),
        };
        meshes.free_dropped();
        for command in &self.commands {
            if let RenderPassCommand::DrawMesh { mesh, .. } = command {
                meshes.upload(mesh, device);
            }
        }


        let (width, height) = render_target.dimensions();

        let gpu = self.gpu.get_or_insert_with(|| GpuState::new(device));
        gpu.depth_view(device, width, height);
        gpu.multisampled_view(device, width, height, render_target.texture_format());
        gpu.ensure_pipeline(device, render_target.texture_format());
        if self.commands.iter().any(|command| matches!(command, RenderPassCommand::DrawMesh { blended: true, .. })) {
            gpu.ensure_blend_pipelines(device, render_target.texture_format());
        }
        gpu.shadow_maps(device, layer_size(&self.shadow_requests), self.shadow_requests.len().max(1) as u32);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("carbide_3d_encoder"),
        });

        // The background, converted as the 2D renderer converts colours: to linear, premultiplied.
        let [r, g, b, a] = view.background.gamma_srgb_to_linear().pre_multiply().to_fsa();
        let background = wgpu::Color { r: r as f64, g: g as f64, b: b as f64, a: a as f64 };

        let clear = Operations {
            load: wgpu::LoadOp::Clear(background),
            store: StoreOp::Store,
        };

        // Nothing to draw: only clear the layer, so an empty scene is transparent.
        if self.commands.is_empty() {
            if let Some(picking) = &mut gpu.picking {
                picking.clear();
            }

            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("carbide_3d_clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target.view(),
                    resolve_target: None,
                    ops: clear,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            queue.submit(std::iter::once(encoder.finish()));
            return;
        }

        let pbr_materials = self.materials.iter().zip(&texture_enable)
            .map(|((material, _), enabled)| match material { WgpuMaterial::PBR(material) => material.with_texture_enable(*enabled) })
            .collect::<Vec<_>>();

        let mut materials_buffer = StorageBuffer::from(pbr_materials);
        // Outline thicknesses in physical pixels, and which objects are outlined.
        let objects = self.objects.iter()
            .map(|object| WgpuObject { outline: object.outline.truncate().extend(object.outline.w * scale_factor), ..*object })
            .collect::<Vec<_>>();
        let outlined = objects.iter().map(|object| object.outline.w > 0.0).collect::<Vec<_>>();
        let max_outline = objects.iter().map(|object| object.outline.w).fold(0.0f32, f32::max);
        let mut objects_buffer = StorageBuffer::from(objects);

        let aspect_ratio = width as f32 / height as f32;
        let inv_view = camera.view.invert().unwrap_or(Matrix4::identity());
        let mut camera_buffer = StorageBuffer::from(WgpuCamera {
            view: camera.view,
            view_proj: camera.view_projection,
            orig_view: Matrix4::from_cols(
                camera.view.x,
                camera.view.y,
                camera.view.z,
                Vector4::unit_w(),
            ),
            inv_view,
            aspect_ratio,
        });
        let (sky, ground, intensity) = self.hemisphere;
        let linear = |color: Color| {
            let [r, g, b, _] = color.gamma_srgb_to_linear().to_fsa();
            Vector4::new(r * intensity, g * intensity, b * intensity, 1.0)
        };
        let mut uniforms = StorageBuffer::from(WgpuUniforms {
            sky: linear(sky),
            ground: linear(ground),
            exposure: view.exposure,
            tonemapping: match view.tonemapping {
                Tonemapping::PbrNeutral => 0,
                Tonemapping::Aces => 1,
                Tonemapping::None => 2,
            },
        });

        // Each shadow map's view-projection, and the part of its layer it fills. Directional maps
        // follow the camera, so they are placed now.
        let shadow_size = layer_size(&self.shadow_requests);
        let mut directional = self.directional_lights.clone();
        let mut punctual = self.punctual_lights.clone();
        let mut shadow_view_projs = Vec::with_capacity(self.shadow_requests.len());

        for request in &self.shadow_requests {
            let resolution = request.settings.resolution.clamp(1, shadow_size);
            let scale = resolution as f32 / shadow_size as f32;

            let view_proj = match request.target {
                ShadowTarget::Directional { index, direction } => {
                    let view_proj = directional_view_proj(direction, inv_view, &request.settings);
                    directional[index].view_proj = view_proj;
                    directional[index].shadow_scale = scale;
                    view_proj
                }
                ShadowTarget::Punctual { index } => {
                    punctual[index].shadow_scale = scale;
                    punctual[index].view_proj
                }
            };

            shadow_view_projs.push((view_proj, resolution));
        }

        let mut directional_lights = StorageBuffer::from(WgpuDirectionalLightBuffer::new(directional));
        let mut point_lights = StorageBuffer::from(WgpuPunctualLightBuffer::new(punctual));

        materials_buffer.write_buffer(device, queue);
        objects_buffer.write_buffer(device, queue);
        camera_buffer.write_buffer(device, queue);
        directional_lights.write_buffer(device, queue);
        point_lights.write_buffer(device, queue);
        uniforms.write_buffer(device, queue);

        let scene_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("carbide_3d_scene_bind_group"),
            layout: &gpu.scene_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: materials_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: objects_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: camera_buffer.binding().unwrap(),
                }
            ],
        });

        let lighting_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("carbide_3d_lighting_bind_group"),
            layout: &gpu.lighting_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: directional_lights.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: point_lights.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&gpu.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: uniforms.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&gpu.shadow_maps.as_ref().unwrap().2),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Sampler(&gpu.shadow_sampler),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(&gpu.nearest_sampler),
                },
            ],
        });

        // A depth-only pass per shadow map, drawing the meshes that cast shadows from the light.
        let shadow_texture = gpu.shadow_maps.as_ref().unwrap().1.clone();
        for (layer, (view_proj, resolution)) in shadow_view_projs.iter().enumerate() {
            let mut view_proj_buffer = StorageBuffer::from(*view_proj);
            view_proj_buffer.write_buffer(device, queue);

            let shadow_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("carbide_3d_shadow_bind_group"),
                layout: &gpu.shadow_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: objects_buffer.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: view_proj_buffer.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: materials_buffer.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::Sampler(&gpu.sampler),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: BindingResource::Sampler(&gpu.nearest_sampler),
                    },
                ],
            });

            let layer_view = shadow_texture.create_view(&TextureViewDescriptor {
                label: Some("carbide_3d_shadow_layer"),
                dimension: Some(TextureViewDimension::D2),
                base_array_layer: layer as u32,
                array_layer_count: Some(1),
                ..Default::default()
            });

            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("carbide_3d_shadow_pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &layer_view,
                    // Cleared to the far plane: where nothing is drawn, nothing is shadowed.
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            shadow_pass.set_pipeline(&gpu.shadow_pipeline);
            shadow_pass.set_bind_group(0, &shadow_bind_group, &[]);
            // A smaller map fills the layer's top-left corner.
            shadow_pass.set_viewport(0.0, 0.0, *resolution as f32, *resolution as f32, 0.0, 1.0);

            for command in &self.commands {
                match command {
                    RenderPassCommand::DrawMesh { mesh, object, casts_shadows: true, .. } => {
                        let Some(cached) = meshes.get(mesh.id()) else {
                            continue;
                        };

                        shadow_pass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                        shadow_pass.set_index_buffer(cached.index_buffer.slice(..), IndexFormat::Uint32);
                        shadow_pass.draw_indexed(0..cached.index_count, 0, *object..*object + 1);
                    }
                    RenderPassCommand::SetPbrMaterial(pbr) => {
                        shadow_pass.set_bind_group(1, &gpu.texture_bind_groups[pbr].1, &[]);
                    }
                    RenderPassCommand::DrawMesh { .. } => {}
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("carbide_3d_render_pass"),
                // Drawn multisampled, and resolved into the layer.
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &gpu.multisampled.as_ref().unwrap().1,
                    resolve_target: Some(render_target.view()),
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(background),
                        store: StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: gpu.depth_view.as_ref().unwrap(),
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: StoreOp::Discard,
                    }),
                    // The depth format has no stencil aspect.
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let (_, pipelines) = gpu.pipeline.as_ref().unwrap();
            // The blended draws, with their view depth and the material textures set before them,
            // drawn after the others.
            let mut blended_draws = vec![];
            let mut current_textures: Option<&WgpuPbrMaterialTextures> = None;
            // Which pipeline is set: switched only when a mesh needs another.
            let mut variant_set = 0;
            render_pass.set_pipeline(&pipelines[0]);

            render_pass.set_bind_group(0, &scene_bind_group, &[]);
            render_pass.set_bind_group(1, &lighting_bind_group, &[]);

            for command in &self.commands {
                match command {
                    RenderPassCommand::DrawMesh { mesh, object, double_sided, mirrored, blended, centre, .. } => {
                        // A mesh with no triangles has no GPU copy, and nothing to draw.
                        let Some(cached) = meshes.get(mesh.id()) else {
                            continue;
                        };

                        // Blended meshes are drawn after the others, sorted.
                        if *blended {
                            let depth = (camera.view * centre.extend(1.0)).z;
                            blended_draws.push((depth, current_textures.clone(), cached, *object, *double_sided, *mirrored));
                            continue;
                        }

                        let variant = pipeline_variant(*double_sided, *mirrored);
                        if variant != variant_set {
                            render_pass.set_pipeline(&pipelines[variant]);
                            variant_set = variant;
                        }

                        render_pass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(cached.index_buffer.slice(..), IndexFormat::Uint32);
                        // The object is the instance index, which the shader reads its transform
                        // and material with: objects can share a mesh's buffers.
                        render_pass.draw_indexed(0..cached.index_count, 0, *object..*object + 1);
                    }
                    RenderPassCommand::SetPbrMaterial(pbr) => {
                        render_pass.set_bind_group(2, &gpu.texture_bind_groups[pbr].1, &[]);
                        current_textures = Some(pbr);
                    }
                }
            }

            // Then the blended meshes, farthest first: view-space depth is negative in front of
            // the camera, so the most negative is the farthest. They test against the depth of
            // the others but don't write it. A double-sided one draws its far side, its back
            // faces, first, then its front faces.
            blended_draws.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (_, textures, cached, object, double_sided, mirrored) in blended_draws {
                if let Some(textures) = textures {
                    render_pass.set_bind_group(2, &gpu.texture_bind_groups[textures].1, &[]);
                }
                render_pass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                render_pass.set_index_buffer(cached.index_buffer.slice(..), IndexFormat::Uint32);

                let (_, blend_pipelines) = gpu.blend_pipelines.as_ref().unwrap();
                let sides: &[bool] = if double_sided { &[true, false] } else { &[false] };
                for back in sides {
                    render_pass.set_pipeline(&blend_pipelines[blend_variant(*back, mirrored)]);
                    render_pass.draw_indexed(0..cached.index_count, 0, object..object + 1);
                }
            }
        }

        // The picking buffer, kept until the next frame for `pick`.
        if view.picking {
            let scene_layout = &gpu.scene_layout;
            let picking = gpu.picking.get_or_insert_with(|| PickingState::new(device, scene_layout));
            picking.draw(device, &mut encoder, (width, height), &scene_bind_group, &self.commands, meshes, camera.view_projection, camera.view);
        } else if let Some(picking) = &mut gpu.picking {
            picking.clear();
        }

        // Outlines, on top of the scene, from the outlined objects' silhouettes.
        if max_outline > 0.0 {
            let format = render_target.texture_format();
            let scene_layout = &gpu.scene_layout;
            let outline = gpu.outline.get_or_insert_with(|| OutlineState::new(device, scene_layout));
            outline.draw(
                device, queue, &mut encoder, render_target.view(), format, (width, height),
                &scene_bind_group, &self.commands, &outlined, meshes, max_outline,
            );
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn pick(&mut self, position: (u32, u32), radius: u32, env: &mut Environment) -> Option<PickSample> {
        let wgpu_context = env.get::<WgpuContextNew>()?;
        let (device, queue) = (wgpu_context.device.clone(), wgpu_context.queue.clone());
        self.gpu.as_mut()?.picking.as_mut()?.pick(&device, &queue, position, radius)
    }

    fn mesh(&mut self, mesh: &Mesh, settings: MeshSettings) {
        let Some((material, material_tex)) = self.material_stack.last().cloned() else {
            // A mesh outside any material has nothing to be shaded with.
            return;
        };

        self.ensure_current_material(&material, &material_tex);
        // Linear, with the thickness still in logical pixels: it is scaled when rendering.
        let outline = match settings.outline {
            Some(outline) if outline.thickness > 0.0 => linear(outline.color).extend(outline.thickness),
            _ => Vector4::new(0.0, 0.0, 0.0, 0.0),
        };
        self.ensure_current_object(settings.receives_shadows, outline, settings.pick_id);

        // A clone of the handle: it keeps the mesh alive until this frame is drawn.
        let object = self.objects.len() as u32 - 1;
        let (double_sided, blended) = match &material {
            WgpuMaterial::PBR(material) => (material.double_sided(), material.blended()),
        };
        let transform = *self.transform_stack.last().unwrap();
        let mirrored = transform.determinant() < 0.0;
        let centre = (transform * mesh.centre().extend(1.0)).truncate();
        self.commands.push(RenderPassCommand::DrawMesh { mesh: mesh.clone(), object, casts_shadows: settings.casts_shadows, double_sided, mirrored, blended, centre });
    }

    fn material(&mut self, material: &Material, env: &mut Environment) {
        self.material_stack.push(match material {
            Material::PBR(material) => {
                (
                    WgpuMaterial::PBR(WgpuPbrMaterial::from_material(material, env)),
                    WgpuPbrMaterialTextures::from_material(material)
                )
            }
        });
    }

    fn pop_material(&mut self) {
        self.material_stack.pop();
    }

    fn transform(&mut self, transform: &Matrix4<f32>) {
        let latest = *self.transform_stack.last().unwrap();
        self.transform_stack.push(latest * transform);
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    fn directional(&mut self, color: Color, intensity: f32, direction: Vector3<f32>, shadow: Option<LightShadow>) {
        // The direction is in the space of the current transform: turn it into world space.
        let world = (*self.transform_stack.last().unwrap() * direction.extend(0.0)).truncate();
        if world.magnitude2() <= f32::EPSILON {
            return;
        }

        let direction = world.normalize();

        // The map follows the view's camera, so its view-projection is set when rendering.
        let (shadow_layer, shadow_bias) = match shadow {
            Some(settings) => {
                let target = ShadowTarget::Directional { index: self.directional_lights.len(), direction };
                (self.request_shadow(target, settings), settings.bias)
            }
            None => (-1, 0.0),
        };

        self.directional_lights.push(WgpuDirectionalLight {
            view_proj: Matrix4::identity(),
            color: linear(color) * intensity,
            shadow_layer,
            direction,
            shadow_bias,
            shadow_scale: 1.0,
        })
    }

    fn point(&mut self, color: Color, intensity: f32, range: Option<f32>) {
        // Point lights don't cast shadows yet: they need cube maps.
        self.punctual(color, intensity, range, 0.0, 1.0, None);
    }

    fn spot(&mut self, color: Color, intensity: f32, range: Option<f32>, inner: Deg<f32>, outer: Deg<f32>, shadow: Option<LightShadow>) {
        let (cos_inner, cos_outer) = (Rad::from(inner).0.cos(), Rad::from(outer).0.cos());
        let scale = 1.0 / (cos_inner - cos_outer).max(0.001);
        self.punctual(color, intensity, range, scale, -cos_outer * scale, shadow.map(|shadow| (shadow, outer)));
    }

    fn hemisphere(&mut self, sky: Color, ground: Color, intensity: f32) {
        self.hemisphere = (sky, ground, intensity);
    }
}
