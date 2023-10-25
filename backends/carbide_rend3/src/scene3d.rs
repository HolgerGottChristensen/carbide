use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use image::buffer;
use rend3::{InstanceAdapterDevice, PotentialAdapter, Renderer, ShaderPreProcessor};
use rend3::types::{DirectionalLightChange, DirectionalLightHandle, glam, MaterialHandle, ObjectHandle, TextureCubeHandle};
use rend3::types::glam::{Mat3, Mat4, UVec2, Vec3};
use rend3_routine::base::BaseRenderGraph;
use rend3_routine::pbr::{PbrMaterial, PbrRoutine};
use rend3_routine::skybox::SkyboxRoutine;
use rend3_routine::tonemapping::TonemappingRoutine;
use uuid::Uuid;
use wgpu::{Texture, TextureFormat, TextureUsages};

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::draw::image::ImageId;
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::layout::Layout;
use carbide_core::mesh::MODE_IMAGE;
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::EnvironmentStateKey;
use carbide_core::widget::*;
use carbide_wgpu::{create_bind_group_from_wgpu_texture, with_adapter, with_bind_groups_mut, with_device_queue, with_instance};

#[derive(Clone, Widget)]
#[carbide_exclude(Render, Layout)]
pub struct Scene3d {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    texture_id: ImageId,
    texture: Option<Rc<Texture>>,
    renderer: Arc<Renderer>,

    directional_light_handle: DirectionalLightHandle,
    object_handle: ObjectHandle,
    material_handle: MaterialHandle,
    rotation: f64,

    inner: Rc<RefCell<InnerScene3d>>
}

struct InnerScene3d {
    skybox: Option<SkyboxRoutine>,
    pbr_routine: PbrRoutine,
    base_rendergraph: BaseRenderGraph,
    spp: ShaderPreProcessor,
    tone_mapping_routine: TonemappingRoutine,
}

impl Scene3d {
    pub fn new() -> Scene3d {
        let iad = Self::get_iad();

        let renderer = Renderer::new(iad, rend3::types::Handedness::Left, None)
            .unwrap();

        let mut spp = ShaderPreProcessor::new();
        rend3_routine::builtin_shaders(&mut spp);

        let base_rendergraph = BaseRenderGraph::new(&renderer, &spp);

        let pbr_routine = PbrRoutine::new(
            &renderer,
            &mut renderer.data_core.lock(),
            &spp,
            &base_rendergraph.interfaces
        );

        let tonemapping_routine = TonemappingRoutine::new(
            &renderer,
            &spp,
            &base_rendergraph.interfaces,
            TextureFormat::Bgra8UnormSrgb,
        );

        // Create mesh and calculate smooth normals based on vertices
        let mesh = create_mesh();

        // Add mesh to renderer's world.
        //
        // All handles are refcounted, so we only need to hang onto the handle until we
        // make an object.
        let mesh_handle = renderer.add_mesh(mesh);

        // Add PBR material with all defaults except a single color.
        let material = PbrMaterial {
            albedo: rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
            unlit: false,
            ..PbrMaterial::default()
        };

        let material_handle = renderer.add_material(material);


        // Combine the mesh and the material with a location to give an object.
        let object = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(mesh_handle),
            material: material_handle.clone(),
            transform: Mat4::IDENTITY,
        };
        // Creating an object will hold onto both the mesh and the material
        // even if they are deleted.
        //
        // We need to keep the object handle alive.
        let object_handle = renderer.add_object(object);

        let view_location = Vec3::new(3.0, 3.0, -5.0);
        let view = Mat4::from_euler(glam::EulerRot::XYZ, -0.55, 0.5, 0.0);
        let view = view * Mat4::from_translation(-view_location);

        // Set camera's location
        renderer.set_camera_data(rend3::types::Camera {
            projection: rend3::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
            view,
        });

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        let directional_handle = renderer.add_directional_light(rend3::types::DirectionalLight {
            color: Vec3::ONE,
            intensity: 3.0,
            // Direction will be normalized
            direction: Vec3::new(-1.0, -4.0, 2.0),
            distance: 5.0,
            resolution: 256,
        });

        Scene3d {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            texture_id: ImageId::new(PathBuf::from(Uuid::new_v4().to_string())),
            texture: None,
            renderer,
            directional_light_handle: directional_handle,
            object_handle,
            material_handle,
            rotation: 0.0,
            inner: Rc::new(RefCell::new(InnerScene3d {
                skybox: None,
                base_rendergraph,
                pbr_routine,
                tone_mapping_routine: tonemapping_routine,
                spp
            })),
        }
    }

    fn get_iad() -> InstanceAdapterDevice {
        with_instance(|instance| {
            with_adapter(|adapter| {
                with_device_queue(|device, queue| {
                    let info = adapter.get_info();
                    let limits = adapter.limits();
                    let features = adapter.features();

                    let potential_adapter = PotentialAdapter::new(
                        adapter,
                        info,
                        limits,
                        features,
                        None
                    ).unwrap();

                    InstanceAdapterDevice {
                        instance,
                        adapter: potential_adapter.inner,
                        device,
                        queue,
                        profile: potential_adapter.profile,
                        info: potential_adapter.info,
                    }
                })
            })
        })
    }

    pub fn skybox<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path = path.as_ref();
        let mut buffer = Vec::new();

        buffer.extend_from_slice(image::open(path.join("right.jpg")).unwrap().into_rgba8().as_raw());
        buffer.extend_from_slice(image::open(path.join("left.jpg")).unwrap().into_rgba8().as_raw());
        buffer.extend_from_slice(image::open(path.join("top.jpg")).unwrap().into_rgba8().as_raw());
        buffer.extend_from_slice(image::open(path.join("bottom.jpg")).unwrap().into_rgba8().as_raw());
        buffer.extend_from_slice(image::open(path.join("front.jpg")).unwrap().into_rgba8().as_raw());
        buffer.extend_from_slice(image::open(path.join("back.jpg")).unwrap().into_rgba8().as_raw());

        let handle = self.renderer.add_texture_cube(rend3::types::Texture {
            format: TextureFormat::Rgba8UnormSrgb,
            size: UVec2::new(2048, 2048),
            data: buffer,
            label: Some("background".into()),
            mip_count: rend3::types::MipmapCount::ONE,
            mip_source: rend3::types::MipmapSource::Uploaded,
        });

        let mut routine = SkyboxRoutine::new(&self.renderer, &self.inner.borrow().spp, &self.inner.borrow().base_rendergraph.interfaces);

        routine.set_background_texture(Some(handle));

        self.inner.borrow_mut().skybox = Some(routine);
        self
    }
}

impl Scene3d {
    fn recreate_texture(&mut self, env: &mut Environment) {
        let texture_size = self.dimension * env.scale_factor();

        if texture_size.width as u32 == 0 || texture_size.height as u32 == 0 {
            return;
        }

        let depth_or_array_layers = 1;

        let texture_extent = wgpu::Extent3d {
            width: texture_size.width as u32,
            height: texture_size.height as u32,
            depth_or_array_layers,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("carbide_wgpu_main_render_tex_rend3"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = with_device_queue(|device, queue| {
            device.create_texture(&texture_descriptor)
        });

        with_bind_groups_mut(|bind_groups| {
            let bind_group = create_bind_group_from_wgpu_texture(&texture);
            bind_groups.insert(self.texture_id.clone(), bind_group);
        });

        self.renderer.set_aspect_ratio((texture_size.width / texture_size.height) as f32);
        self.texture = Some(Rc::new(texture))
    }
}

impl Layout for Scene3d {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {

        // If the requested size is not the same as the dimension, recreate the texture
        if requested_size != self.dimension || self.texture.is_none() {
            self.dimension = requested_size;
            self.recreate_texture(env);
        }
        requested_size
    }
}

impl Render for Scene3d {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {

        self.rotation = self.rotation + 0.01;

        self.renderer.set_object_transform(&self.object_handle, Mat4::from_rotation_y(self.rotation as f32));

        let color = env.get_color(&EnvironmentStateKey::Color(EnvironmentColor::Accent)).unwrap();

        let color = glam::Vec4::new(color.red(), color.green(), color.blue(), color.opacity());

        self.renderer.update_material(&self.material_handle, PbrMaterial {
            albedo: rend3_routine::pbr::AlbedoComponent::Value(color),
            unlit: false,
            ..PbrMaterial::default()
        });

        env.request_animation_frame();

        // Swap the instruction buffers so that our frame's changes can be processed.
        self.renderer.swap_instruction_buffers();
        // Evaluate our frame's world-change instructions
        let mut eval_output = self.renderer.evaluate_instructions();

        let mut inner = self.inner.borrow_mut();
        inner.skybox.as_mut().map(|a| a.evaluate(&self.renderer));

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        let texture = &**self.texture.as_ref().unwrap();
        let resolution = UVec2::new(texture.width(), texture.height());

        // Import the surface texture into the render graph.
        let frame_handle =
            graph.add_imported_render_target(texture, 0..1, rend3::graph::ViewportRect::from_size(resolution));

        // Add the default rendergraph without a skybox

        inner.base_rendergraph.add_to_graph(
            &mut graph,
            &eval_output,
            &inner.pbr_routine,
            inner.skybox.as_ref(),
            &inner.tone_mapping_routine,
            frame_handle,
            resolution,
            rend3::types::SampleCount::One,
            Vec3::splat(0.10).extend(1.0),
            glam::Vec4::new(0.0, 0.0, 0.0, 0.0),
        );

        // Dispatch a render using the built up rendergraph!
        graph.execute(&self.renderer, &mut eval_output);

        context.image(
            self.texture_id.clone(),
            self.bounding_box(),
            Rect::from_corners(Position::new(0.0, 1.0), Position::new(1.0, 0.0)),
            MODE_IMAGE,
        );
    }
}

/*impl OtherEventHandler for Mandelbrot {
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {}
}

impl MouseEventHandler for Mandelbrot {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {}
}*/

impl CommonWidget for Scene3d {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl WidgetExt for Scene3d {}

impl Debug for Scene3d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

fn vertex(pos: [f32; 3]) -> glam::Vec3 {
    glam::Vec3::from(pos)
}

fn create_mesh() -> rend3::types::Mesh {
    let vertex_positions = [
        // far side (0.0, 0.0, 1.0)
        vertex([-1.0, -1.0, 1.0]),
        vertex([1.0, -1.0, 1.0]),
        vertex([1.0, 1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0]),
        // near side (0.0, 0.0, -1.0)
        vertex([-1.0, 1.0, -1.0]),
        vertex([1.0, 1.0, -1.0]),
        vertex([1.0, -1.0, -1.0]),
        vertex([-1.0, -1.0, -1.0]),
        // right side (1.0, 0.0, 0.0)
        vertex([1.0, -1.0, -1.0]),
        vertex([1.0, 1.0, -1.0]),
        vertex([1.0, 1.0, 1.0]),
        vertex([1.0, -1.0, 1.0]),
        // left side (-1.0, 0.0, 0.0)
        vertex([-1.0, -1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0]),
        vertex([-1.0, 1.0, -1.0]),
        vertex([-1.0, -1.0, -1.0]),
        // top (0.0, 1.0, 0.0)
        vertex([1.0, 1.0, -1.0]),
        vertex([-1.0, 1.0, -1.0]),
        vertex([-1.0, 1.0, 1.0]),
        vertex([1.0, 1.0, 1.0]),
        // bottom (0.0, -1.0, 0.0)
        vertex([1.0, -1.0, 1.0]),
        vertex([-1.0, -1.0, 1.0]),
        vertex([-1.0, -1.0, -1.0]),
        vertex([1.0, -1.0, -1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // far
        4, 5, 6, 6, 7, 4, // near
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ];

    rend3::types::MeshBuilder::new(vertex_positions.to_vec(), rend3::types::Handedness::Left)
        .with_indices(index_data.to_vec())
        .build()
        .unwrap()
}
