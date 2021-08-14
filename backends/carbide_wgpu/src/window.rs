use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use cgmath::{Deg, Matrix4, Point3, Rad, SquareMatrix, Vector3};
pub use futures::executor::block_on;
use image::DynamicImage;
use uuid::Uuid;
use wgpu::{BindGroup, BindGroupLayout, Device, PresentMode, RenderPassDepthStencilAttachmentDescriptor, Texture, TextureView};
use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Icon, WindowBuilder};

use carbide_core::{Scalar, Ui};
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::event::Input;
use carbide_core::image_map::{Id, ImageMap};
use carbide_core::mesh::DEFAULT_GLYPH_CACHE_DIMS;
use carbide_core::mesh::mesh::Mesh;
use carbide_core::prelude::{Environment, EnvironmentColor};
use carbide_core::prelude::Rectangle;
use carbide_core::text::{FontFamily, FontId};
use carbide_core::widget::OverlaidLayer;
use carbide_core::widget::Widget;
pub use carbide_core::window::TWindow;

use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
use crate::glyph_cache_command::GlyphCacheCommand;
use crate::image::Image;
use crate::pipeline::{create_render_pipeline, MaskType};
use crate::render_pass_command::{create_render_pass_commands, RenderPassCommand};
use crate::renderer::{atlas_cache_tex_desc, glyph_cache_tex_desc};
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::vertex::Vertex;

// Todo: Look in to multisampling: https://github.com/gfx-rs/wgpu-rs/blob/v0.6/examples/msaa-line/main.rs
pub struct Window {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline_no_mask: wgpu::RenderPipeline,
    render_pipeline_add_mask: wgpu::RenderPipeline,
    render_pipeline_in_mask: wgpu::RenderPipeline,
    render_pipeline_remove_mask: wgpu::RenderPipeline,
    depth_texture_view: TextureView,
    diffuse_bind_group: wgpu::BindGroup,
    mesh: Mesh,
    ui: Ui,
    image_map: ImageMap<Image>,
    glyph_cache_tex: Texture,
    atlas_cache_tex: Texture,
    bind_groups: HashMap<Id, DiffuseBindGroup>,
    texture_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    carbide_to_wgpu_matrix: Matrix4<f32>,
    inner_window: winit::window::Window,
    event_loop: Option<EventLoop<()>>,
}

impl carbide_core::window::TWindow for Window {
    fn add_font_family(&mut self, mut family: FontFamily) -> String {
        let family_name = family.name.clone();
        self.ui.environment.add_font_family(family);
        family_name
    }

    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join(path.as_ref());

        self.ui.environment.insert_font_from_file(font_path)
    }

    fn add_image(&mut self, path: &str) -> Id {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let image = Image::new(assets.join(path), &self.device, &self.queue);

        let information = image.image_information();

        let id = self.image_map.insert(image);

        self.ui.environment.insert_image(id, information);

        id
    }

    fn set_widgets(&mut self, w: Box<dyn Widget>) {
        self.ui.widgets = Rectangle::new(vec![OverlaidLayer::new("controls_popup_layer", w)])
            .fill(EnvironmentColor::SystemBackground);
    }
}

impl Window {
    pub fn path_to_assets(path: &str) -> PathBuf {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        assets.join(path)
    }

    pub fn environment(&self) -> &Environment {
        &self.ui.environment
    }

    pub(crate) fn matrix_to_uniform_bind_group(device: &Device, layout: &BindGroupLayout, matrix: Matrix4<f32>) -> BindGroup {
        let uniforms: [[f32; 4]; 4] = matrix.into();

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        uniform_bind_group
    }

    fn calculate_carbide_to_wgpu_matrix(dimension: Dimension, fov: Scalar, scale_factor: Scalar) -> Matrix4<f32> {
        let fov = fov as f32;
        let half_height = (dimension.height / 2.0);
        let scale = (scale_factor / half_height) as f32;

        #[rustfmt::skip]
        pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        let pixel_to_points: [[f32; 4]; 4] = [
            [scale, 0.0, 0.0, 0.0],
            [0.0, -scale, 0.0, 0.0],
            [0.0, 0.0, scale, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let angle_to_screen_center = 90.0;

        let outer_angle = 180.0 - (fov / 2.0) - angle_to_screen_center;

        let z = outer_angle.to_radians().tan() as f32;
        let aspect_ratio = (dimension.width / dimension.height) as f32;

        let perspective = cgmath::perspective(cgmath::Deg(fov), aspect_ratio, 0.1, 100.0);
        //let view = cgmath::Matrix4::look_at_lh(self.eye, self.target, self.up);
        let up: Vector3<f32> = cgmath::Vector3::unit_y();
        let target: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
        let eye: Point3<f32> = Point3::new(0.0, 0.0, z);

        let view = Matrix4::look_at_rh(eye, target, up);
        //perspective * view *
        let ortho = cgmath::ortho(-1.0 * aspect_ratio, 1.0 * aspect_ratio, -1.0, 1.0, 1.0, -1.0);
        let res = OPENGL_TO_WGPU_MATRIX * ortho * Matrix4::from_translation(Vector3::new(-aspect_ratio, 1.0, 0.0)) * Matrix4::from(pixel_to_points);
        res
    }

    pub fn new(title: String, width: u32, height: u32, icon: Option<PathBuf>) -> Self {
        let event_loop = EventLoop::new();

        let loaded_icon = if let Some(path) = icon {
            let rgba_logo_image = image::open(path).expect("Couldn't load logo").to_rgba();

            let width = rgba_logo_image.width();
            let height = rgba_logo_image.height();

            Some(Icon::from_rgba(rgba_logo_image.into_raw(), width, height).unwrap())
        } else {
            None
        };

        let inner_window = WindowBuilder::new()
            .with_inner_size(Size::Physical(PhysicalSize { width, height }))
            .with_title(title)
            .with_window_icon(loaded_icon)
            .build(&event_loop)
            .unwrap();

        if let Some(monitor) = inner_window.current_monitor() {
            let size = monitor.size();

            let outer_window_size = inner_window.outer_size();

            let position = PhysicalPosition::new(
                size.width / 2 - outer_window_size.width / 2,
                size.height / 2 - outer_window_size.height / 2,
            );

            inner_window.set_outer_position(position);
        }

        println!("DPI: {}", inner_window.scale_factor());

        let size = inner_window.inner_size();

        let pixel_dimensions = Dimension::new(
            inner_window.inner_size().width as f64,
            inner_window.inner_size().height as f64,
        );
        let scale_factor = inner_window.scale_factor();

        let ui = Ui::new(pixel_dimensions, scale_factor);

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&inner_window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        }))
            .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None, // Trace path
        ))
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let fov = 45.0;
        let matrix = Window::calculate_carbide_to_wgpu_matrix(pixel_dimensions, fov, scale_factor);
        let uniforms: [[f32; 4]; 4] = matrix.clone().into();

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let text_cache_tex_desc = glyph_cache_tex_desc(DEFAULT_GLYPH_CACHE_DIMS);
        let glyph_cache_tex = device.create_texture(&text_cache_tex_desc);
        let atlas_cache_tex_desc = atlas_cache_tex_desc([512, 512]);
        let atlas_cache_tex = device.create_texture(&atlas_cache_tex_desc);

        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();

        let image = Image::new(assets.join("images/happy-tree.png"), &device, &queue);

        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_no_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &vs_module,
            &fs_module,
            &sc_desc,
            MaskType::NoMask,
        );
        let render_pipeline_add_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &vs_module,
            &fs_module,
            &sc_desc,
            MaskType::AddMask,
        );
        let render_pipeline_in_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &vs_module,
            &fs_module,
            &sc_desc,
            MaskType::InMask,
        );
        let render_pipeline_remove_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &vs_module,
            &fs_module,
            &sc_desc,
            MaskType::RemoveMask,
        );

        let bind_groups = HashMap::new();

        let diffuse_bind_group = new_diffuse(
            &device,
            &image,
            &glyph_cache_tex,
            &atlas_cache_tex,
            &texture_bind_group_layout,
        );

        let mesh = Mesh::with_glyph_cache_dimensions(DEFAULT_GLYPH_CACHE_DIMS);

        let image_map = ImageMap::new();

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture descriptor"),
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let depth_texture_view = depth_texture.create_view(&Default::default());

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline_no_mask,
            render_pipeline_add_mask,
            render_pipeline_in_mask,
            render_pipeline_remove_mask,
            depth_texture_view,
            diffuse_bind_group,
            mesh,
            ui,
            image_map,
            glyph_cache_tex,
            atlas_cache_tex,
            bind_groups,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            uniform_bind_group,
            inner_window,
            carbide_to_wgpu_matrix: matrix,
            event_loop: Some(event_loop),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.ui.set_window_width(self.size.width as f64);
        self.ui.set_window_height(self.size.height as f64);
        self.ui.handle_event(Input::Redraw);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture descriptor"),
            size: wgpu::Extent3d {
                width: new_size.width,
                height: new_size.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let depth_texture_view = depth_texture.create_view(&Default::default());
        self.depth_texture_view = depth_texture_view;

        let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);
        let fov = 45.0;
        let scale_factor = self.inner_window.scale_factor();

        self.carbide_to_wgpu_matrix = Window::calculate_carbide_to_wgpu_matrix(dimension, fov, scale_factor);

        let uniform_bind_group = Window::matrix_to_uniform_bind_group(&self.device, &self.uniform_bind_group_layout, self.carbide_to_wgpu_matrix);

        self.uniform_bind_group = uniform_bind_group;

        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match convert_window_event(event, &self.inner_window) {
            None => false,
            Some(input) => {
                self.ui.handle_event(input);
                false
            }
        }
    }

    fn update(&mut self) {
        let update_start = Instant::now();
        self.ui.delegate_events();
        println!(
            "Time for update: {:?}us",
            update_start.elapsed().as_micros()
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // This blocks until a new frame is available.
        let frame = self.swap_chain.get_current_frame()?.output;

        let render_start = Instant::now();

        let now = Instant::now();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        println!("encodere: {:?}us", now.elapsed().as_micros());

        let now = Instant::now();
        let primitives = self.ui.draw();
        println!("Time for draw: {:?}us", now.elapsed().as_micros());
        let now = Instant::now();
        let fill = self
            .mesh
            .fill(
                Rect::new(
                    Position::new(0.0, 0.0),
                    Dimension::new(self.size.width as f64, self.size.height as f64),
                ),
                &mut self.ui.environment,
                &self.image_map,
                primitives,
            )
            .unwrap();
        println!("Time for fill: {:?}us", now.elapsed().as_micros());

        let now = Instant::now();
        // Check if an upload to the glyph cache is needed
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

        match glyph_cache_cmd {
            None => (),
            Some(cmd) => {
                cmd.load_buffer_and_encode(&self.device, &mut encoder);
            }
        }
        println!("glyph_cache_cmd: {:?}us", now.elapsed().as_micros());
        let now = Instant::now();

        // Check if an upload to texture atlas is needed.
        let texture_atlas_cmd = match fill.atlas_requires_upload {
            true => {
                let width = self.mesh.texture_atlas().width();
                let height = self.mesh.texture_atlas().height();
                Some(TextureAtlasCommand {
                    texture_atlas_buffer: self.mesh.texture_atlas_image_as_bytes(),
                    texture_atlas_texture: &self.atlas_cache_tex,
                    width: 512,
                    height: 512,
                })
            }
            false => None,
        };

        match texture_atlas_cmd {
            None => (),
            Some(cmd) => {
                cmd.load_buffer_and_encode(&self.device, &mut encoder);
            }
        }
        println!("atlas: {:?}us", now.elapsed().as_micros());
        let now = Instant::now();

        let mut uniform_bind_groups = vec![];

        let commands = create_render_pass_commands(
            &self.diffuse_bind_group,
            &mut self.bind_groups,
            &mut uniform_bind_groups,
            &self.image_map,
            &self.mesh,
            &self.device,
            &self.glyph_cache_tex,
            &self.atlas_cache_tex,
            &self.texture_bind_group_layout,
            &self.uniform_bind_group_layout,
            self.carbide_to_wgpu_matrix,
        );
        println!("commands: {:?}us", now.elapsed().as_micros());
        //println!("{:#?}", self.mesh.vertices());

        let vertices: Vec<Vertex> = self
            .mesh
            .vertices()
            .iter()
            .map(|v| Vertex::from(*v))
            .collect::<Vec<_>>();

        let now = Instant::now();
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });
        println!("vertex_buffer: {:?}us", now.elapsed().as_micros());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline_no_mask); // 2.
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            let instance_range = 0..1;
            let mut stencil_level = 0;
            for cmd in commands {
                match cmd {
                    RenderPassCommand::SetBindGroup { bind_group } => {
                        render_pass.set_bind_group(0, bind_group, &[]);
                    }
                    RenderPassCommand::SetScissor {
                        top_left,
                        dimensions,
                    } => {
                        let [x, y] = top_left;
                        let [w, h] = dimensions;
                        render_pass.set_scissor_rect(x, y, w, h);
                    }
                    RenderPassCommand::Draw { vertex_range } => {
                        render_pass.draw(vertex_range, instance_range.clone());
                    }
                    RenderPassCommand::Stencil { vertex_range } => {
                        stencil_level += 1;
                        render_pass.set_pipeline(&self.render_pipeline_add_mask);
                        render_pass.draw(vertex_range, instance_range.clone());
                        render_pass.set_pipeline(&self.render_pipeline_in_mask);
                        render_pass.set_stencil_reference(stencil_level);
                    }
                    RenderPassCommand::DeStencil { vertex_range } => {
                        stencil_level -= 1;
                        render_pass.set_pipeline(&self.render_pipeline_remove_mask);
                        render_pass.draw(vertex_range, instance_range.clone());
                        render_pass.set_stencil_reference(stencil_level);
                        if stencil_level == 0 {
                            render_pass.set_pipeline(&self.render_pipeline_no_mask);
                        } else {
                            render_pass.set_pipeline(&self.render_pipeline_in_mask);
                        }
                    }
                    RenderPassCommand::Transform { uniform_bind_group_index } => {
                        render_pass.set_bind_group(1, &uniform_bind_groups[uniform_bind_group_index], &[]);
                    }
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        println!(
            "Time for render: {:?}us",
            render_start.elapsed().as_micros()
        );
        Ok(())
    }

    pub fn run_event_loop(mut self) {
        // Make the state sync on event loop run
        self.input(&WindowEvent::Focused(true));

        let mut event_loop = None;

        std::mem::swap(&mut event_loop, &mut self.event_loop);

        event_loop.expect("The eventloop should be retrieved").run(
            move |event, _, control_flow| {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == self.inner_window.id() => {
                        if !self.input(event) {
                            match event {
                                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                                WindowEvent::KeyboardInput { input, .. } => {
                                    match input {
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::F1),
                                            ..
                                        } => {
                                            // This is only for debugging purposes.
                                            use image::GrayImage;
                                            use std::fs::*;
                                            let image_folder =
                                                String::from("/tmp/carbide_img_dump_")
                                                    + &Uuid::new_v4().to_string();
                                            create_dir_all(&image_folder);
                                            self.mesh
                                                .texture_atlas_image()
                                                .save(image_folder.clone() + "/glyph_atlas0.png");
                                            let atlas1 = DynamicImage::ImageLuma8(
                                                GrayImage::from_raw(
                                                    DEFAULT_GLYPH_CACHE_DIMS[0],
                                                    DEFAULT_GLYPH_CACHE_DIMS[1],
                                                    self.mesh.glyph_cache_pixel_buffer().to_vec(),
                                                )
                                                    .unwrap(),
                                            );
                                            atlas1.save(image_folder.clone() + "/glyph_atlas1.png");
                                            println!("Images dumped to: {}", image_folder);
                                        }
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::Escape),
                                            ..
                                        } => *control_flow = ControlFlow::Exit,
                                        _ => {}
                                    }
                                }
                                WindowEvent::Resized(physical_size) => {
                                    self.resize(*physical_size);
                                    self.inner_window.request_redraw();
                                }
                                WindowEvent::ScaleFactorChanged {
                                    new_inner_size,
                                    scale_factor,
                                } => {
                                    self.resize(**new_inner_size);
                                    self.ui.set_scale_factor(*scale_factor);
                                    self.inner_window.request_redraw();
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::RedrawRequested(_) => {
                        self.update();
                        match self.render() {
                            Ok(_) => {}
                            // Recreate the swap_chain if lost
                            Err(wgpu::SwapChainError::Lost) => self.resize(self.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SwapChainError::OutOfMemory) => {
                                *control_flow = ControlFlow::Exit
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    Event::MainEventsCleared => {
                        // RedrawRequested will only trigger once, unless we manually
                        // request it.
                        self.inner_window.request_redraw();
                    }
                    _ => {}
                }
            },
        );
    }
}

carbide_winit::v023_conversion_fns!();
