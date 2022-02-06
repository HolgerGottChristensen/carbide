use std::collections::HashMap;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use cgmath::{Matrix4, Vector3};
pub use futures::executor::block_on;
use image::DynamicImage;
//use smaa::{SmaaMode, SmaaTarget};
use uuid::Uuid;
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferUsages, PresentMode, Sampler, SurfaceConfiguration, Texture, TextureView};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(target_os = "macos")]
use winit::platform::macos::WindowExtMacOS;
use winit::window::{Icon, WindowBuilder};

use carbide_core::{Scalar, Ui};
use carbide_core::draw::Dimension;
use carbide_core::event::Input;
use carbide_core::image_map::{Id, ImageMap};
use carbide_core::mesh::{DEFAULT_GLYPH_CACHE_DIMS, MODE_IMAGE};
use carbide_core::mesh::mesh::Mesh;
use carbide_core::prelude::{Environment, EnvironmentColor};
use carbide_core::prelude::Rectangle;
use carbide_core::text::{FontFamily, FontId};
use carbide_core::widget::{OverlaidLayer, ZStack};
use carbide_core::widget::Widget;
use carbide_core::event::CustomEvent;
pub use carbide_core::window::TWindow;

use crate::bind_group_layouts::{filter_buffer_bind_group_layout, filter_texture_bind_group_layout, main_texture_group_layout, uniform_bind_group_layout};
use crate::bind_groups::{filter_texture_bind_group, main_bind_group, matrix_to_uniform_bind_group, size_to_uniform_bind_group};
use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
use crate::image::Image;
use crate::pipeline::{create_render_pipeline, MaskType};
use crate::render_pipeline_layouts::{filter_pipeline_layout, main_pipeline_layout};
use crate::renderer::{atlas_cache_tex_desc, main_render_tex_desc, secondary_render_tex_desc};
use crate::samplers::main_sampler;
use crate::textures::create_depth_stencil_texture;
use crate::vertex::Vertex;
use crate::proxy_event_loop::ProxyEventLoop;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowExtWindows;

// Todo: Look into multisampling: https://github.com/gfx-rs/wgpu-rs/blob/v0.6/examples/msaa-line/main.rs
// An alternative is https://github.com/fintelia/smaa-rs (https://github.com/gfx-rs/naga/issues/1275)
// In v0.8 and later I see performance degradation with the filter_shader (https://github.com/gfx-rs/wgpu/issues/1842)

pub struct Window {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    pub(crate) render_pipeline_no_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_add_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_in_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_remove_mask: wgpu::RenderPipeline,
    pub(crate) render_pipeline_in_mask_filter: wgpu::RenderPipeline,
    pub(crate) render_pipeline_no_mask_filter: wgpu::RenderPipeline,
    pub(crate) depth_texture_view: TextureView,
    pub(crate) diffuse_bind_group: wgpu::BindGroup,
    pub(crate) main_bind_group: wgpu::BindGroup,
    pub(crate) texture_size_bind_group: wgpu::BindGroup,
    pub(crate) mesh: Mesh,
    pub(crate) ui: Ui,
    pub(crate) image_map: ImageMap<Image>,
    pub(crate) atlas_cache_tex: Texture,
    pub(crate) main_tex: Texture,
    pub(crate) main_tex_view: TextureView,
    pub(crate) secondary_tex: Texture,
    pub(crate) secondary_tex_view: TextureView,
    pub(crate) bind_groups: HashMap<Id, DiffuseBindGroup>,
    pub(crate) filter_buffer_bind_groups: HashMap<u32, BindGroup>,
    pub(crate) texture_bind_group_layout: BindGroupLayout,
    pub(crate) uniform_bind_group_layout: BindGroupLayout,
    pub(crate) filter_texture_bind_group_layout: BindGroupLayout,
    pub(crate) filter_buffer_bind_group_layout: BindGroupLayout,
    pub(crate) filter_main_texture_bind_group: BindGroup,
    pub(crate) filter_secondary_texture_bind_group: BindGroup,
    pub(crate) uniform_bind_group: wgpu::BindGroup,
    pub(crate) carbide_to_wgpu_matrix: Matrix4<f32>,
    pub(crate) vertex_buffer: (Buffer, usize),
    pub(crate) second_vertex_buffer: Buffer,
    pub(crate) main_sampler: Sampler,
    inner_window: winit::window::Window,
    event_loop: Option<EventLoop<CustomEvent>>,
}

impl carbide_core::window::TWindow for Window {
    fn add_font_family(&mut self, family: FontFamily) -> String {
        let family_name = family.name.clone();
        self.ui.environment.add_font_family(family);
        family_name
    }

    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let font_path = assets.join(path.as_ref());

        self.ui.environment.insert_font_from_file(font_path).0
    }

    fn add_image_from_path(&mut self, path: &str) -> Option<Id> {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let image = Image::new(assets.join(path), &self.device, &self.queue);

        let information = image.image_information();

        let id = self.image_map.insert(image);

        self.ui.environment.insert_image(id, information);

        Some(id)
    }

    fn add_image(&mut self, image: image::DynamicImage) -> Option<Id> {
        let image = Image::new_from_dynamic(image, &self.device, &self.queue);

        let information = image.image_information();

        let id = self.image_map.insert(image);

        self.ui.environment.insert_image(id, information);

        Some(id)
    }

    fn set_widgets(&mut self, base_widget: Box<dyn Widget>) {
        self.ui.widgets = ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::SystemBackground),
            OverlaidLayer::new("controls_popup_layer", base_widget).steal_events(),
        ]);
    }
}

impl Window {
    pub fn relative_path_to_assets(path: &str) -> PathBuf {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        assets.join(path)
    }

    pub fn environment(&self) -> &Environment {
        &self.ui.environment
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.ui.environment
    }

    fn calculate_carbide_to_wgpu_matrix(dimension: Dimension, scale_factor: Scalar) -> Matrix4<f32> {
        let half_height = dimension.height / 2.0;
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

        let aspect_ratio = (dimension.width / dimension.height) as f32;

        let ortho = cgmath::ortho(-1.0 * aspect_ratio, 1.0 * aspect_ratio, -1.0, 1.0, 1.0, -1.0);
        let res = OPENGL_TO_WGPU_MATRIX * ortho * Matrix4::from_translation(Vector3::new(-aspect_ratio, 1.0, 0.0)) * Matrix4::from(pixel_to_points);
        res
    }

    pub fn new(title: impl Into<String>, width: u32, height: u32, icon: Option<PathBuf>) -> Self {
        let event_loop = EventLoop::<CustomEvent>::with_user_event();

        let loaded_icon = if let Some(path) = icon {
            let rgba_logo_image = image::open(path).expect("Couldn't load logo").to_rgba8();

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

        #[cfg(target_os = "macos")]
            let ui = Ui::new(
            pixel_dimensions,
            scale_factor,
            Some(inner_window.ns_window()),
            Box::new(ProxyEventLoop(event_loop.create_proxy()))
        );

        #[cfg(target_os = "windows")]
            let ui = Ui::new(pixel_dimensions, scale_factor, Some(inner_window.hwnd()), Box::new(event_sink));


        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
            let ui = Ui::new(pixel_dimensions, scale_factor, None, Box::new(event_sink));

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&inner_window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }))
            .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        ))
            .unwrap();

        surface.configure(&device, &SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        });

        let uniform_bind_group_layout = uniform_bind_group_layout(&device);
        let filter_texture_bind_group_layout = filter_texture_bind_group_layout(&device);
        let filter_buffer_bind_group_layout = filter_buffer_bind_group_layout(&device);
        let main_texture_bind_group_layout = main_texture_group_layout(&device);

        let matrix = Window::calculate_carbide_to_wgpu_matrix(pixel_dimensions, scale_factor);
        let uniform_bind_group = matrix_to_uniform_bind_group(&device, &uniform_bind_group_layout, matrix);
        let texture_size_bind_group = size_to_uniform_bind_group(&device, &uniform_bind_group_layout, pixel_dimensions.width, pixel_dimensions.height, scale_factor);

        let main_tex = device.create_texture(&main_render_tex_desc([pixel_dimensions.width as u32, pixel_dimensions.height as u32]));
        let main_tex_view = main_tex.create_view(&Default::default());
        let secondary_tex = device.create_texture(&secondary_render_tex_desc([size.width, size.height]));
        let secondary_tex_view = secondary_tex.create_view(&Default::default());

        let atlas_cache_tex_desc = atlas_cache_tex_desc([512, 512]);
        let atlas_cache_tex = device.create_texture(&atlas_cache_tex_desc);

        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();

        let image = Image::new(assets.join("images/happy-tree.png"), &device, &queue);

        let main_shader = device.create_shader_module(&wgpu::include_wgsl!("../shaders/shader.wgsl"));
        let wgsl_filter_shader = device.create_shader_module(&wgpu::include_wgsl!("../shaders/filter.wgsl"));

        let render_pipeline_layout = main_pipeline_layout(&device, &main_texture_bind_group_layout, &uniform_bind_group_layout);
        let filter_render_pipeline_layout = filter_pipeline_layout(&device, &filter_texture_bind_group_layout, &filter_buffer_bind_group_layout, &uniform_bind_group_layout);

        let render_pipeline_no_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &main_shader,
            &surface,
            &adapter,
            MaskType::NoMask,
        );
        let render_pipeline_add_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &main_shader,
            &surface,
            &adapter,
            MaskType::AddMask,
        );
        let render_pipeline_in_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &main_shader,
            &surface,
            &adapter,
            MaskType::InMask,
        );
        let render_pipeline_remove_mask = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            &main_shader,
            &surface,
            &adapter,
            MaskType::RemoveMask,
        );
        let render_pipeline_in_mask_filter = create_render_pipeline(
            &device,
            &filter_render_pipeline_layout,
            &wgsl_filter_shader,
            &surface,
            &adapter,
            MaskType::InMask,
        );
        let render_pipeline_no_mask_filter = create_render_pipeline(
            &device,
            &filter_render_pipeline_layout,
            &wgsl_filter_shader,
            &surface,
            &adapter,
            MaskType::NoMask,
        );

        let bind_groups = HashMap::new();
        let filter_bind_groups = HashMap::new();

        let diffuse_bind_group = new_diffuse(
            &device,
            &image,
            &atlas_cache_tex,
            &main_texture_bind_group_layout,
        );

        let main_sampler = main_sampler(&device);

        let main_bind_group = main_bind_group(&device, &main_texture_bind_group_layout, &main_tex_view, &main_sampler, &atlas_cache_tex);

        let mesh = Mesh::with_glyph_cache_dimensions(DEFAULT_GLYPH_CACHE_DIMS);

        let image_map = ImageMap::new();

        let depth_texture = create_depth_stencil_texture(&device, width, height);
        let depth_texture_view = depth_texture.create_view(&Default::default());

        let vertex_buffer = device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

        let last_verts: Vec<Vertex> = vec![
            Vertex::new_from_2d(0.0, 0.0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(size.width as f32 / scale_factor as f32, 0.0, [0.0, 0.0, 0.0, 0.0], [1.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(0.0, size.height as f32 / scale_factor as f32, [0.0, 0.0, 0.0, 0.0], [0.0, 1.0], MODE_IMAGE),
            Vertex::new_from_2d(size.width as f32 / scale_factor as f32, 0.0, [0.0, 0.0, 0.0, 0.0], [1.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(size.width as f32 / scale_factor as f32, size.height as f32 / scale_factor as f32, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0], MODE_IMAGE),
            Vertex::new_from_2d(0.0, size.height as f32 / scale_factor as f32, [0.0, 0.0, 0.0, 0.0], [0.0, 1.0], MODE_IMAGE),
        ];

        let second_verts_buffer = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&last_verts),
                usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

        let filter_main_texture_bind_group = filter_texture_bind_group(&device, &filter_texture_bind_group_layout, &main_tex_view, &main_sampler);
        let filter_secondary_texture_bind_group = filter_texture_bind_group(&device, &filter_texture_bind_group_layout, &secondary_tex_view, &main_sampler);

        /*let smaa = SmaaTarget::new(
            &device,
            &queue,
            window.inner_size().width,
            window.inner_size().height,
            swapchain_format,
            SmaaMode::Smaa1X,
        );*/

        Self {
            surface,
            device,
            queue,
            size,
            render_pipeline_no_mask,
            render_pipeline_add_mask,
            render_pipeline_in_mask,
            render_pipeline_remove_mask,
            render_pipeline_in_mask_filter,
            render_pipeline_no_mask_filter,
            depth_texture_view,
            diffuse_bind_group,
            main_bind_group,
            texture_size_bind_group,
            mesh,
            ui,
            image_map,
            atlas_cache_tex,
            main_tex,
            main_tex_view,
            secondary_tex,
            secondary_tex_view,
            bind_groups,
            filter_buffer_bind_groups: filter_bind_groups,
            texture_bind_group_layout: main_texture_bind_group_layout,
            filter_texture_bind_group_layout,
            filter_buffer_bind_group_layout,
            filter_main_texture_bind_group,
            filter_secondary_texture_bind_group,
            uniform_bind_group_layout,
            uniform_bind_group,
            inner_window,
            vertex_buffer: (vertex_buffer, 0),
            carbide_to_wgpu_matrix: matrix,
            event_loop: Some(event_loop),
            second_vertex_buffer: second_verts_buffer,
            main_sampler,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.ui.set_window_width(self.size.width as f64);
        self.ui.set_window_height(self.size.height as f64);
        self.ui.compound_and_add_event(Input::Redraw);
        let depth_texture = create_depth_stencil_texture(&self.device, new_size.width, new_size.height);
        let depth_texture_view = depth_texture.create_view(&Default::default());
        self.depth_texture_view = depth_texture_view;

        let main_tex = self.device.create_texture(&main_render_tex_desc([new_size.width, new_size.height]));
        let main_tex_view = main_tex.create_view(&Default::default());
        let secondary_tex = self.device.create_texture(&secondary_render_tex_desc([new_size.width, new_size.height]));
        let secondary_tex_view = secondary_tex.create_view(&Default::default());

        let main_texture_bind_group_layout = main_texture_group_layout(&self.device);

        let scale_factor = self.inner_window.scale_factor();

        self.main_bind_group = main_bind_group(&self.device, &main_texture_bind_group_layout, &main_tex_view, &self.main_sampler, &self.atlas_cache_tex);
        let texture_size_bind_group = size_to_uniform_bind_group(&self.device, &self.uniform_bind_group_layout, self.size.width as f64, self.size.height as f64, scale_factor);
        self.texture_size_bind_group = texture_size_bind_group;


        self.main_tex = main_tex;
        self.main_tex_view = main_tex_view;
        self.secondary_tex = secondary_tex;
        self.secondary_tex_view = secondary_tex_view;

        self.filter_main_texture_bind_group = filter_texture_bind_group(&self.device, &self.filter_texture_bind_group_layout, &self.main_tex_view, &self.main_sampler);
        self.filter_secondary_texture_bind_group = filter_texture_bind_group(&self.device, &self.filter_texture_bind_group_layout, &self.secondary_tex_view, &self.main_sampler);

        let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);

        self.carbide_to_wgpu_matrix = Window::calculate_carbide_to_wgpu_matrix(dimension, scale_factor);

        let uniform_bind_group = matrix_to_uniform_bind_group(&self.device, &self.uniform_bind_group_layout, self.carbide_to_wgpu_matrix);

        self.uniform_bind_group = uniform_bind_group;

        let last_verts: Vec<Vertex> = vec![
            Vertex::new_from_2d(0.0, 0.0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(self.size.width as f32 / scale_factor as f32, 0.0, [0.0, 0.0, 0.0, 0.0], [1.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(0.0, self.size.height as f32 / scale_factor as f32, [0.0, 0.0, 0.0, 0.0], [0.0, 1.0], MODE_IMAGE),
            Vertex::new_from_2d(self.size.width as f32 / scale_factor as f32, 0.0, [0.0, 0.0, 0.0, 0.0], [1.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(self.size.width as f32 / scale_factor as f32, self.size.height as f32 / scale_factor as f32, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0], MODE_IMAGE),
            Vertex::new_from_2d(0.0, self.size.height as f32 / scale_factor as f32, [0.0, 0.0, 0.0, 0.0], [0.0, 1.0], MODE_IMAGE),
        ];

        self.queue.write_buffer(&self.second_vertex_buffer, 0, bytemuck::cast_slice(&last_verts));

        self.surface.configure(&self.device, &SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: new_size.width,
            height: new_size.height,
            present_mode: PresentMode::Mailbox,
        });
    }

    /// Request the window to redraw next frame
    fn request_redraw(&self) {
        self.inner_window.request_redraw();
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match convert_window_event(event, &self.inner_window) {
            None => false,
            Some(input) => {
                self.ui.compound_and_add_event(input);
                false
            }
        }
    }

    /// Update the state of the UI by delegating all captured events to the widgets
    /// Returns *true* if the update should result in a redraw.
    fn update(&mut self) -> bool {
        // Capture the current time and update the animations in the environment.
        self.environment_mut().capture_time();
        self.environment_mut().update_animation();
        self.environment_mut().clear_animation_frame();

        let next_index = self.image_map.next_index();
        self.environment_mut().set_last_image_index(next_index);
        self.environment_mut().check_tasks();
        self.add_queued_images();

        self.ui.delegate_events()
    }

    fn add_queued_images(&mut self) {
        if let Some(queued_images) = self.environment_mut().queued_images() {
            for queued_image in queued_images {
                let _ = self.add_image(queued_image);
            }
        }
    }

    /// This method launches the application. It is a call that only completed after the window
    /// is closed.
    pub fn launch(mut self) {
        // Make the state sync on event loop run
        self.input(&WindowEvent::Focused(true));
        let mut event_loop = None;
        std::mem::swap(&mut event_loop, &mut self.event_loop);

        let mut last_render_start_time = Instant::now();

        event_loop.expect("The event loop should be retrieved").run(
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
                                            create_dir_all(&image_folder).unwrap();
                                            self.mesh
                                                .texture_atlas_image()
                                                .save(image_folder.clone() + "/glyph_atlas0.png").unwrap();
                                            let atlas1 = DynamicImage::ImageLuma8(
                                                GrayImage::from_raw(
                                                    DEFAULT_GLYPH_CACHE_DIMS[0],
                                                    DEFAULT_GLYPH_CACHE_DIMS[1],
                                                    self.mesh.glyph_cache_pixel_buffer().to_vec(),
                                                )
                                                    .unwrap(),
                                            );
                                            atlas1.save(image_folder.clone() + "/glyph_atlas1.png").unwrap();
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
                                    self.ui.set_scale_factor(*scale_factor);
                                    self.resize(**new_inner_size);
                                    self.inner_window.request_redraw();
                                }
                                _ => {}
                            }
                        }
                    }

                    // Gets called whenever we receive carbide sent events
                    Event::UserEvent(event) => {
                        println!("{:?}", event);
                        self.ui.compound_and_add_event(Input::Custom(event));
                        self.request_redraw();
                    }

                    // Gets called when all window and user events are delivered
                    Event::MainEventsCleared => {
                        // If we have any events queued up and update the UI
                        if self.ui.has_queued_events() || self.ui.has_animations() {
                            // If the ui should redraw because of the update
                            if self.update() || self.ui.has_animations() {
                                self.request_redraw();
                            }

                            self.inner_window.set_cursor_icon(convert_mouse_cursor(self.ui.mouse_cursor()));
                        }
                    }

                    // Gets called if redrawing is requested.
                    Event::RedrawRequested(_) => {
                        last_render_start_time = Instant::now();

                        match self.render() {
                            Ok(_) => {}
                            // Recreate the swap_chain if lost
                            Err(wgpu::SurfaceError::Lost) => {
                                println!("Swap chain lost");
                                self.resize(self.size);
                                self.request_redraw();
                            },
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                println!("Swap chain out of memory");
                                *control_flow = ControlFlow::Exit
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => {
                                // We request a redraw the next frame
                                self.request_redraw();
                                eprintln!("{:?}", e)
                            },
                        }

                        // Wait for the next event to be received
                        *control_flow = ControlFlow::Wait;
                    }

                    // This is called after the rendering
                    Event::RedrawEventsCleared => {
                        // If we have any animations running we should draw as soon as possible next frame
                        if self.ui.has_animations() {
                            self.request_redraw();
                        }
                        //self.request_redraw();
                    }
                    _ => {}
                }
            },
        );
    }
}

carbide_winit::v023_conversion_fns!();
