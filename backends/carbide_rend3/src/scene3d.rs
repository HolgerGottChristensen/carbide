use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use futures::channel;
use futures::channel::oneshot;
use futures::executor::block_on;
use rend3::{InstanceAdapterDevice, Renderer, ShaderPreProcessor};
use rend3::types::glam;
use rend3_routine::base::BaseRenderGraph;
use wgpu::{TextureFormat, TextureUsages};

use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::render::Render;
use carbide_core::widget::*;

#[derive(Clone, Widget)]
//#[carbide_exclude(MouseEvent, Render, OtherEvent)]
pub struct Scene3d {
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    //iad: InstanceAdapterDevice,
    //renderer: Arc<Renderer>,
    //spp: ShaderPreProcessor,
    //base_render_graph: BaseRenderGraph,
}

impl Scene3d {
    pub fn new() -> Scene3d {

        let texture_size = 256u32;

        let depth_or_array_layers = 1;

        let texture_extent = wgpu::Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("carbide_wgpu_main_render_tex_rend3"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let u32_size = std::mem::size_of::<u32>() as u32;

        let output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };


        let iad = block_on(rend3::create_iad(None, None, None, None)).unwrap();

        let texture = iad.device.create_texture(&texture_descriptor);

        let output_buffer = iad.device.create_buffer(&output_buffer_desc);

        let renderer = Renderer::new(
            iad.clone(),
            rend3::types::Handedness::Left,
            Some(texture_size as f32 / texture_size as f32),
        )
            .unwrap();

        let mut spp = ShaderPreProcessor::new();
        rend3_routine::builtin_shaders(&mut spp);

        let base_rendergraph = BaseRenderGraph::new(&renderer, &spp);

        let mut data_core = renderer.data_core.lock();

        let pbr_routine =
            rend3_routine::pbr::PbrRoutine::new(&renderer, &mut data_core, &spp, &base_rendergraph.interfaces);

        drop(data_core);
        let tonemapping_routine = rend3_routine::tonemapping::TonemappingRoutine::new(
            &renderer,
            &spp,
            &base_rendergraph.interfaces,
            TextureFormat::Rgba8UnormSrgb,
        );

        // Create mesh and calculate smooth normals based on vertices
        let mesh = create_mesh();

        // Add mesh to renderer's world.
        //
        // All handles are refcounted, so we only need to hang onto the handle until we
        // make an object.
        let mesh_handle = renderer.add_mesh(mesh);

        // Add PBR material with all defaults except a single color.
        let material = rend3_routine::pbr::PbrMaterial {
            albedo: rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
            ..rend3_routine::pbr::PbrMaterial::default()
        };
        let material_handle = renderer.add_material(material);

        // Combine the mesh and the material with a location to give an object.
        let object = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(mesh_handle),
            material: material_handle,
            transform: glam::Mat4::IDENTITY,
        };
        // Creating an object will hold onto both the mesh and the material
        // even if they are deleted.
        //
        // We need to keep the object handle alive.
        let _object_handle = renderer.add_object(object);

        let view_location = glam::Vec3::new(3.0, 3.0, -5.0);
        let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, -0.55, 0.5, 0.0);
        let view = view * glam::Mat4::from_translation(-view_location);

        // Set camera's location
        renderer.set_camera_data(rend3::types::Camera {
            projection: rend3::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
            view,
        });

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        let _directional_handle = renderer.add_directional_light(rend3::types::DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 10.0,
            // Direction will be normalized
            direction: glam::Vec3::new(-1.0, -4.0, 2.0),
            distance: 400.0,
            resolution: 2048,
        });

        let mut resolution = glam::UVec2::new(texture_size, texture_size);

        // Swap the instruction buffers so that our frame's changes can be processed.
        renderer.swap_instruction_buffers();
        // Evaluate our frame's world-change instructions
        let mut eval_output = renderer.evaluate_instructions();

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        // Import the surface texture into the render graph.
        let frame_handle =
            graph.add_imported_render_target(&texture, 0..1, rend3::graph::ViewportRect::from_size(resolution));
        // Add the default rendergraph without a skybox
        base_rendergraph.add_to_graph(
            &mut graph,
            &eval_output,
            &pbr_routine,
            None,
            &tonemapping_routine,
            frame_handle,
            resolution,
            rend3::types::SampleCount::One,
            glam::Vec4::ZERO,
            glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
        );

        // Dispatch a render using the built up rendergraph!
        graph.execute(&renderer, &mut eval_output);

        let mut encoder = iad.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(u32_size * texture_size),
                    rows_per_image: Some(texture_size),
                },
            },
            texture_descriptor.size,
        );

        iad.queue.submit(Some(encoder.finish()));

        // We need to scope the mapping variables so that we can
        // unmap the buffer
        {
            let buffer_slice = output_buffer.slice(..);

            // NOTE: We have to create the mapping THEN device.poll() before await
            // the future. Otherwise the application will freeze.
            let (tx, rx) = oneshot::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            iad.device.poll(wgpu::Maintain::Wait);
            block_on(async {
                rx.await
            }).unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer =
                ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size, texture_size, data).unwrap();
            buffer.save("image.png").unwrap();

        }
        output_buffer.unmap();


        todo!()
    }
}

/*impl Render for Mandelbrot {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {}
}

impl OtherEventHandler for Mandelbrot {
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
