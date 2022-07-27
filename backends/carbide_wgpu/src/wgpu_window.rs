use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use cgmath::{Matrix4, Vector3};
use futures::executor::block_on;
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferUsages, Device, Extent3d, ImageCopyTexture, PresentMode, Queue, RenderPassDepthStencilAttachment, Sampler, Surface, SurfaceConfiguration, Texture, TextureView};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event_loop::ControlFlow;
use winit::window::{Window as WinitWindow, WindowBuilder};
use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::draw::image::{ImageId, ImageMap};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{Event, KeyboardEvent, KeyboardEventHandler, MouseEvent, MouseEventHandler, OtherEventHandler, WidgetEvent, WindowEvent};
use carbide_core::flags::Flags;
use carbide_core::focus::{Focus, Focusable};
use carbide_core::image::DynamicImage;
use carbide_core::layout::Layout;
use carbide_core::mesh::{DEFAULT_GLYPH_CACHE_DIMS, MODE_IMAGE};
use carbide_core::mesh::mesh::Mesh;
use carbide_core::prelude::{CommonWidget, Primitive, Rectangle, Render, StateSync, WidgetId, WidgetIter, WidgetIterMut};
use carbide_core::render::Primitives;
use carbide_core::{Scalar, Scene};
use carbide_core::widget::{FilterId, OverlaidLayer, Widget, ZStack};
use carbide_core::window::WindowId;
use crate::application::{EVENT_LOOP, WINDOW_IDS};
use crate::bind_group_layouts::{filter_buffer_bind_group_layout, filter_texture_bind_group_layout, gradient_buffer_bind_group_layout, main_texture_group_layout, uniform_bind_group_layout};
use crate::bind_groups::{filter_buffer_bind_group, filter_texture_bind_group, main_bind_group, matrix_to_uniform_bind_group, size_to_uniform_bind_group};
use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
use crate::image::Image;
use crate::pipeline::{create_render_pipeline, MaskType};
use crate::render_pass_command::{create_render_pass_commands, RenderPass, RenderPassCommand};
use crate::{render_pass_ops, RenderPassOps};
use crate::filter::Filter;
use crate::render_pipeline_layouts::{filter_pipeline_layout, gradient_pipeline_layout, main_pipeline_layout, RenderPipelines};
use crate::renderer::{atlas_cache_tex_desc, main_render_tex_desc, secondary_render_tex_desc};
use crate::samplers::main_sampler;
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::textures::create_depth_stencil_texture;
use crate::vertex::Vertex;
//use crate::diffuse_bind_group::DiffuseBindGroup;
//use crate::image::Image;
//use crate::render_pipeline_layouts::RenderPipelines;

pub struct WGPUWindow {
    pub(crate) surface: Surface,
    pub(crate) device: Device,
    pub(crate) queue: Queue,

    pub(crate) render_pipelines: RenderPipelines,

    pub(crate) depth_texture_view: TextureView,
    pub(crate) diffuse_bind_group: BindGroup,
    pub(crate) main_bind_group: BindGroup,
    pub(crate) texture_size_bind_group: BindGroup,
    pub(crate) mesh: Mesh,
    pub(crate) image_map: ImageMap<Image>,
    pub(crate) atlas_cache_tex: Texture,
    pub(crate) main_tex: Texture,
    pub(crate) main_tex_view: TextureView,
    pub(crate) secondary_tex: Texture,
    pub(crate) secondary_tex_view: TextureView,
    pub(crate) bind_groups: HashMap<ImageId, DiffuseBindGroup>,
    pub(crate) filter_buffer_bind_groups: HashMap<FilterId, BindGroup>,
    pub(crate) texture_bind_group_layout: BindGroupLayout,
    pub(crate) uniform_bind_group_layout: BindGroupLayout,
    pub(crate) filter_texture_bind_group_layout: BindGroupLayout,
    pub(crate) filter_buffer_bind_group_layout: BindGroupLayout,
    pub(crate) gradient_bind_group_layout: BindGroupLayout,
    pub(crate) filter_main_texture_bind_group: BindGroup,
    pub(crate) filter_secondary_texture_bind_group: BindGroup,
    pub(crate) uniform_bind_group: BindGroup,
    pub(crate) carbide_to_wgpu_matrix: Matrix4<f32>,
    pub(crate) vertex_buffer: (Buffer, usize),
    pub(crate) second_vertex_buffer: Buffer,
    pub(crate) main_sampler: Sampler,
    inner: Rc<WinitWindow>,

    id: WidgetId,
    window_id: WindowId,
    title: String,
    position: Position,
    dimension: Dimension,
    child: Box<dyn Widget>,
    close_application_on_window_close: bool,
    visible: bool,
}

impl WGPUWindow {
    pub fn new(title: impl Into<String>, dimension: Dimension, child: Box<dyn Widget>) -> Box<Self> {
        let window_id = WindowId::new();
        let title = title.into();


        let child = ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::SystemBackground),
            OverlaidLayer::new("controls_popup_layer", child).steal_events(),
        ]);

        let builder = WindowBuilder::new()
            .with_inner_size(Size::Logical(LogicalSize {
                width: dimension.width,
                height: dimension.height,
            }))
            .with_title(title.clone())
            //.with_window_icon(loaded_icon)
            ;

        let inner = EVENT_LOOP.with(|a| {
            a.borrow().create_inner_window(builder)
        });

        // Add the window to the list of IDS to make event propagate when received by the window.
        WINDOW_IDS.with(|a| {
            a.borrow_mut().insert(inner.id(), window_id);
        });

        // Position the window in the middle of the screen.
        if let Some(monitor) = inner.current_monitor() {
            let size = monitor.size();

            let outer_window_size = inner.outer_size();

            let position = PhysicalPosition::new(
                size.width / 2 - outer_window_size.width / 2,
                size.height / 2 - outer_window_size.height / 2,
            );

            inner.set_outer_position(position);
        }

        println!("DPI: {}", inner.scale_factor());


        let size = inner.inner_size();

        let pixel_dimensions = Dimension::new(
            inner.inner_size().width as f64,
            inner.inner_size().height as f64,
        );
        let scale_factor = inner.scale_factor();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&inner) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None, // Trace path
        ))
            .unwrap();


        // Configure the surface with format, size and usage
        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: inner.inner_size().width,
                height: inner.inner_size().height,
                present_mode: PresentMode::Mailbox,
            },
        );

        let uniform_bind_group_layout = uniform_bind_group_layout(&device);
        let filter_texture_bind_group_layout = filter_texture_bind_group_layout(&device);
        let filter_buffer_bind_group_layout = filter_buffer_bind_group_layout(&device);
        let main_texture_bind_group_layout = main_texture_group_layout(&device);
        let gradient_bind_group_layout = gradient_buffer_bind_group_layout(&device);

        let matrix = Self::calculate_carbide_to_wgpu_matrix(pixel_dimensions, scale_factor);

        let uniform_bind_group =
            matrix_to_uniform_bind_group(&device, &uniform_bind_group_layout, matrix);
        let texture_size_bind_group = size_to_uniform_bind_group(
            &device,
            &uniform_bind_group_layout,
            pixel_dimensions.width,
            pixel_dimensions.height,
            scale_factor,
        );

        let main_tex = device.create_texture(&main_render_tex_desc([
            pixel_dimensions.width as u32,
            pixel_dimensions.height as u32,
        ]));
        let main_tex_view = main_tex.create_view(&Default::default());
        let secondary_tex =
            device.create_texture(&secondary_render_tex_desc([size.width, size.height]));
        let secondary_tex_view = secondary_tex.create_view(&Default::default());

        let atlas_cache_tex_desc = atlas_cache_tex_desc([512, 512]);
        let atlas_cache_tex = device.create_texture(&atlas_cache_tex_desc);

        let image = Image::new_from_dynamic(DynamicImage::new_rgba8(1, 1), &device, &queue);

        let main_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/shader.wgsl"));
        let gradient_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/gradient.wgsl"));
        let wgsl_filter_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/filter.wgsl"));

        let render_pipeline_layout = main_pipeline_layout(
            &device,
            &main_texture_bind_group_layout,
            &uniform_bind_group_layout,
        );
        let filter_render_pipeline_layout = filter_pipeline_layout(
            &device,
            &filter_texture_bind_group_layout,
            &filter_buffer_bind_group_layout,
            &uniform_bind_group_layout,
        );
        let gradient_render_pipeline_layout = gradient_pipeline_layout(
            &device,
            &gradient_bind_group_layout,
            &uniform_bind_group_layout,
        );

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

        let render_pipeline_in_mask_gradient = create_render_pipeline(
            &device,
            &gradient_render_pipeline_layout,
            &gradient_shader,
            &surface,
            &adapter,
            MaskType::InMask,
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

        let main_bind_group = main_bind_group(
            &device,
            &main_texture_bind_group_layout,
            &main_tex_view,
            &main_sampler,
            &atlas_cache_tex,
        );

        let mesh = Mesh::with_glyph_cache_dimensions(DEFAULT_GLYPH_CACHE_DIMS);

        let image_map = ImageMap::default();

        let depth_texture = create_depth_stencil_texture(&device, size.width, size.height);
        let depth_texture_view = depth_texture.create_view(&Default::default());

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let last_verts: Vec<Vertex> = vec![
            Vertex::new_from_2d(0.0, 0.0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(
                size.width as f32 / scale_factor as f32,
                0.0,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                0.0,
                size.height as f32 / scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                size.width as f32 / scale_factor as f32,
                0.0,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                size.width as f32 / scale_factor as f32,
                size.height as f32 / scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 1.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                0.0,
                size.height as f32 / scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0],
                MODE_IMAGE,
            ),
        ];

        let second_verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&last_verts),
            usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let filter_main_texture_bind_group = filter_texture_bind_group(
            &device,
            &filter_texture_bind_group_layout,
            &main_tex_view,
            &main_sampler,
        );
        let filter_secondary_texture_bind_group = filter_texture_bind_group(
            &device,
            &filter_texture_bind_group_layout,
            &secondary_tex_view,
            &main_sampler,
        );

        Box::new(WGPUWindow {
            surface,
            device,
            queue,
            render_pipelines: RenderPipelines {
                render_pipeline_no_mask,
                render_pipeline_add_mask,
                render_pipeline_in_mask,
                render_pipeline_remove_mask,
                render_pipeline_in_mask_filter,
                render_pipeline_no_mask_filter,
                render_pipeline_in_mask_gradient,
            },
            depth_texture_view,
            diffuse_bind_group,
            main_bind_group,
            texture_size_bind_group,
            mesh,
            image_map,
            atlas_cache_tex,
            main_tex,
            main_tex_view,
            secondary_tex,
            secondary_tex_view,
            bind_groups,
            filter_buffer_bind_groups: filter_bind_groups,
            texture_bind_group_layout: main_texture_bind_group_layout,
            uniform_bind_group_layout,
            filter_texture_bind_group_layout,
            filter_buffer_bind_group_layout,
            gradient_bind_group_layout,
            filter_main_texture_bind_group,
            filter_secondary_texture_bind_group,
            uniform_bind_group,
            carbide_to_wgpu_matrix: matrix,
            vertex_buffer: (vertex_buffer, 0),
            second_vertex_buffer: second_verts_buffer,
            main_sampler,

            inner: Rc::new(inner),
            id: WidgetId::new(),
            window_id,
            title,
            position: Default::default(),
            dimension: Default::default(),
            child,
            close_application_on_window_close: false,
            visible: true,
        })
    }

    pub fn close_application_on_window_close(mut self) -> Box<Self> {
        self.close_application_on_window_close = true;
        Box::new(self)
    }

    fn calculate_carbide_to_wgpu_matrix(
        dimension: Dimension,
        scale_factor: Scalar,
    ) -> Matrix4<f32> {
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

        let ortho = cgmath::ortho(
            -1.0 * aspect_ratio,
            1.0 * aspect_ratio,
            -1.0,
            1.0,
            1.0,
            -1.0,
        );
        let res = OPENGL_TO_WGPU_MATRIX
            * ortho
            * Matrix4::from_translation(Vector3::new(-aspect_ratio, 1.0, 0.0))
            * Matrix4::from(pixel_to_points);
        res
    }
}

impl MouseEventHandler for WGPUWindow {
    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        let old_is_current = env.is_event_current();
        let old_dimension = env.pixel_dimensions();
        let old_scale_factor = env.scale_factor();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        env.set_event_is_current_by_id(self.window_id);
        env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        env.set_scale_factor(scale_factor);

        if !*consumed {
            if env.is_event_current() {
                self.capture_state(env);
                self.handle_mouse_event(event, consumed, env);
                self.release_state(env);
            }
        }

        for mut child in self.children_direct() {
            child.process_mouse_event(event, &consumed, env);
            if *consumed {
                return ();
            }
        }

        env.set_event_is_current(old_is_current);
        env.set_pixel_dimensions(old_dimension);
        env.set_scale_factor(old_scale_factor);
    }
}

impl CommonWidget for WGPUWindow {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else if self.child.flag() == Flags::IGNORE {
            WidgetIter::Empty
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else if self.child.flag() == Flags::IGNORE {
            WidgetIterMut::Empty
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension;
    }
}

impl StateSync for WGPUWindow {
    fn capture_state(&mut self, env: &mut Environment) {

    }

    fn release_state(&mut self, env: &mut Environment) {

    }
}

impl Focusable for WGPUWindow {}

impl KeyboardEventHandler for WGPUWindow {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        let old_is_current = env.is_event_current();
        let old_dimension = env.pixel_dimensions();
        let old_scale_factor = env.scale_factor();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        env.set_event_is_current_by_id(self.window_id);
        env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        env.set_scale_factor(scale_factor);

        if env.is_event_current() {
            self.capture_state(env);
            self.handle_keyboard_event(event, env);
            self.release_state(env);
        }

        for mut child in self.children_direct() {
            child.process_keyboard_event(event, env);
        }

        env.set_event_is_current(old_is_current);
        env.set_pixel_dimensions(old_dimension);
        env.set_scale_factor(old_scale_factor);
    }
}

impl OtherEventHandler for WGPUWindow {
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        match event {
            /*WindowEvent::Resized(physical_size) => {
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
            }*/
            WidgetEvent::Window(e) => {
                match e {
                    WindowEvent::Resize(size) => {
                        self.resize(LogicalSize::new(size.width, size.height).to_physical(2.0), env);
                        self.request_redraw();
                    }
                    WindowEvent::Focus => {}
                    WindowEvent::UnFocus => {}
                    WindowEvent::Redraw => {}
                    WindowEvent::CloseRequested => {
                        self.visible = false;
                        if self.close_application_on_window_close {
                            env.close_application();
                        } else {
                            self.inner.set_visible(false);
                        }
                    }
                }
            }
            _ => ()
        }
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        let old_is_current = env.is_event_current();
        let old_dimension = env.pixel_dimensions();
        let old_scale_factor = env.scale_factor();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        env.set_event_is_current_by_id(self.window_id);
        env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        env.set_scale_factor(scale_factor);

        if env.is_event_current() {
            self.capture_state(env);
            self.handle_other_event(event, env);
            self.release_state(env);
        }

        for mut child in self.children_direct() {
            child.process_other_event(event, env);
        }

        env.set_event_is_current(old_is_current);
        env.set_pixel_dimensions(old_dimension);
        env.set_scale_factor(old_scale_factor);
    }
}

impl Layout for WGPUWindow {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        Dimension::new(0.0, 0.0)
    }

    fn position_children(&mut self) {}
}

impl Render for WGPUWindow {
    fn process_get_primitives(&mut self, _: &mut Vec<Primitive>, env: &mut Environment) {
        let old_scale_factor = env.scale_factor();
        let old_pixel_dimensions = env.pixel_dimensions();

        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();
        let logical_dimensions = physical_dimensions.to_logical(scale_factor);
        let dimensions = Dimension::new(logical_dimensions.width, logical_dimensions.height);

        env.capture_time();

        env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        env.set_scale_factor(scale_factor);

        let primitives = Primitives::new(
            dimensions,
            &mut self.child,
            env,
        );

        if self.visible {
            match self.render(primitives, env) {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SurfaceError::Lost) => {
                    println!("Swap chain lost");
                    self.resize(self.inner.inner_size(), env);
                    self.request_redraw();
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    println!("Swap chain out of memory");
                    env.close_application();
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => {
                    // We request a redraw the next frame
                    self.request_redraw();
                    eprintln!("{:?}", e)
                }
            }
        }

        env.set_pixel_dimensions(old_pixel_dimensions);
        env.set_scale_factor(old_scale_factor);
    }
}

impl Widget for WGPUWindow {}

impl Scene for WGPUWindow {
    /// Request the window to redraw next frame
    fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}

impl WGPUWindow {
    fn render(&mut self, primitives: Vec<Primitive>, env: &mut Environment) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let size = self.inner.inner_size();

        self.image_map.retain(|a, _| env.image_map.contains_key(a));

        for (id, img) in env.image_map.iter() {
            if self.image_map.contains_key(id) {
                continue;
            }

            let image = Image::new_from_dynamic(img.clone(), &self.device, &self.queue);

            self.image_map.insert(id.clone(), image);
        }


        let fill = self
            .mesh
            .fill(
                Rect::new(
                    Position::new(0.0, 0.0),
                    Dimension::new(size.width as f64, size.height as f64),
                ),
                env,
                &self.image_map,
                primitives,
            )
            .unwrap();

        // Check if an upload to texture atlas is needed.
        let texture_atlas_cmd = match fill.atlas_requires_upload {
            true => {
                let width = self.mesh.texture_atlas().width();
                let height = self.mesh.texture_atlas().height();
                Some(TextureAtlasCommand {
                    texture_atlas_buffer: self.mesh.texture_atlas_image_as_bytes(),
                    texture_atlas_texture: &self.atlas_cache_tex,
                    width,
                    height,
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

        let mut uniform_bind_groups = vec![];

        let keys = env
            .filters()
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        self.filter_buffer_bind_groups
            .retain(|id, _| keys.contains(id));

        for (filter_id, filter) in env.filters() {
            if !self.filter_buffer_bind_groups.contains_key(filter_id) {
                let filter: Filter = filter.clone().into();
                let filter_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Filter Buffer"),
                            contents: &*filter.as_bytes(),
                            usage: wgpu::BufferUsages::STORAGE,
                        });
                let filter_buffer_bind_group = filter_buffer_bind_group(
                    &self.device,
                    &self.filter_buffer_bind_group_layout,
                    &filter_buffer,
                );
                self.filter_buffer_bind_groups
                    .insert(*filter_id, filter_buffer_bind_group);
            }
        }


        let commands = create_render_pass_commands(
            &self.diffuse_bind_group,
            &mut self.bind_groups,
            &mut uniform_bind_groups,
            &self.image_map,
            &self.mesh,
            &self.device,
            &self.atlas_cache_tex,
            &self.texture_bind_group_layout,
            &self.uniform_bind_group_layout,
            &self.gradient_bind_group_layout,
            self.carbide_to_wgpu_matrix,
        );

        let vertices: Vec<Vertex> = self
            .mesh
            .vertices()
            .iter()
            .map(|v| Vertex::from(*v))
            .collect::<Vec<_>>();

        if vertices.len() <= self.vertex_buffer.1 {
            // There is space in the current vertex buffer
            self.queue
                .write_buffer(&self.vertex_buffer.0, 0, bytemuck::cast_slice(&vertices));
        } else {
            // We need to create a new and larger vertex buffer
            let new_vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            self.vertex_buffer = (new_vertex_buffer, vertices.len());
        }

        let instance_range = 0..1;
        let mut stencil_level = 0;
        let mut first_pass = true;

        let mut current_main_render_pipeline = &self.render_pipelines.render_pipeline_no_mask;
        let current_vertex_buffer_slice = self.vertex_buffer.0.slice(..);
        let mut current_uniform_bind_group = &self.uniform_bind_group;

        for command in commands {
            match command {
                RenderPass::Normal(inner) => {
                    if inner.len() == 0 {
                        continue;
                    }
                    let (color_op, stencil_op) = if first_pass {
                        first_pass = false;
                        render_pass_ops(RenderPassOps::Start)
                    } else {
                        render_pass_ops(RenderPassOps::Middle)
                    };
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_pipeline(current_main_render_pipeline);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);

                    for inner_command in inner {
                        match inner_command {
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
                                render_pass.set_pipeline(&self.render_pipelines.render_pipeline_add_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                current_main_render_pipeline = &self.render_pipelines.render_pipeline_in_mask;
                                render_pass.set_pipeline(current_main_render_pipeline);
                                render_pass.set_stencil_reference(stencil_level);
                            }
                            RenderPassCommand::DeStencil { vertex_range } => {
                                stencil_level -= 1;
                                render_pass.set_pipeline(&self.render_pipelines.render_pipeline_remove_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                render_pass.set_stencil_reference(stencil_level);
                                if stencil_level == 0 {
                                    current_main_render_pipeline = &self.render_pipelines.render_pipeline_no_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                } else {
                                    current_main_render_pipeline = &self.render_pipelines.render_pipeline_in_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                }
                            }
                            RenderPassCommand::Transform {
                                uniform_bind_group_index,
                            } => {
                                current_uniform_bind_group =
                                    &uniform_bind_groups[uniform_bind_group_index];
                                render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                            }
                        }
                    }
                }
                RenderPass::Gradient(vertex_range, bind_group_index) => {
                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_in_mask_gradient);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &uniform_bind_groups[bind_group_index], &[]);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::Filter(vertex_range, bind_group_index) => {
                    encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: &self.main_tex,
                            mip_level: 0,
                            origin: Default::default(),
                            aspect: Default::default(),
                        },
                        ImageCopyTexture {
                            texture: &self.secondary_tex,
                            mip_level: 0,
                            origin: Default::default(),
                            aspect: Default::default(),
                        },
                        Extent3d {
                            width: size.width,
                            height: size.height,
                            depth_or_array_layers: 1,
                        },
                    );

                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.filter_secondary_texture_bind_group, &[]);
                    render_pass.set_bind_group(
                        1,
                        &self
                            .filter_buffer_bind_groups
                            .get(&bind_group_index)
                            .unwrap(),
                        &[],
                    );
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.texture_size_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::FilterSplitPt1(vertex_range, filter_id) => {
                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.secondary_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_no_mask_filter);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.filter_main_texture_bind_group, &[]);
                    render_pass.set_bind_group(
                        1,
                        &self.filter_buffer_bind_groups.get(&filter_id).unwrap(),
                        &[],
                    );
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.texture_size_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::FilterSplitPt2(vertex_range, filter_id) => {
                    let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &self.main_tex_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: None,
                            stencil_ops: Some(stencil_op),
                        }),
                    });
                    render_pass.set_pipeline(&self.render_pipelines.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.filter_secondary_texture_bind_group, &[]);
                    render_pass.set_bind_group(
                        1,
                        &self.filter_buffer_bind_groups.get(&filter_id).unwrap(),
                        &[],
                    );
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.texture_size_bind_group, &[]);
                    render_pass.draw(vertex_range, instance_range.clone());
                }
            };
        }

        // Render from the texture to the swap chain
        let (color_op, stencil_op) = render_pass_ops(RenderPassOps::Middle);

        // This blocks until a new frame is available.
        let output = self.surface.get_current_texture()?;
        let frame_view = output.texture.create_view(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame_view, // Here is the render target
                resolve_target: None,
                ops: color_op,
            }],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: None,
                stencil_ops: Some(stencil_op),
            }),
        });

        render_pass.set_pipeline(&self.render_pipelines.render_pipeline_no_mask);
        render_pass.set_vertex_buffer(0, self.second_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.main_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.draw(0..6, instance_range);

        drop(render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, _: &mut Environment) {
        let size = new_size;
        //env.set_pixel_dimensions(size.width as f64);
        //env.set_pixel_height(size.height as f64);
        //self.ui.compound_and_add_event(Input::Redraw);

        let depth_texture =
            create_depth_stencil_texture(&self.device, new_size.width, new_size.height);
        let depth_texture_view = depth_texture.create_view(&Default::default());
        self.depth_texture_view = depth_texture_view;

        let main_tex = self
            .device
            .create_texture(&main_render_tex_desc([new_size.width, new_size.height]));
        let main_tex_view = main_tex.create_view(&Default::default());
        let secondary_tex = self.device.create_texture(&secondary_render_tex_desc([
            new_size.width,
            new_size.height,
        ]));
        let secondary_tex_view = secondary_tex.create_view(&Default::default());

        let main_texture_bind_group_layout = main_texture_group_layout(&self.device);

        let scale_factor = self.inner.scale_factor();

        self.main_bind_group = main_bind_group(
            &self.device,
            &main_texture_bind_group_layout,
            &main_tex_view,
            &self.main_sampler,
            &self.atlas_cache_tex,
        );
        let texture_size_bind_group = size_to_uniform_bind_group(
            &self.device,
            &self.uniform_bind_group_layout,
            size.width as f64,
            size.height as f64,
            scale_factor,
        );
        self.texture_size_bind_group = texture_size_bind_group;

        self.main_tex = main_tex;
        self.main_tex_view = main_tex_view;
        self.secondary_tex = secondary_tex;
        self.secondary_tex_view = secondary_tex_view;

        self.filter_main_texture_bind_group = filter_texture_bind_group(
            &self.device,
            &self.filter_texture_bind_group_layout,
            &self.main_tex_view,
            &self.main_sampler,
        );
        self.filter_secondary_texture_bind_group = filter_texture_bind_group(
            &self.device,
            &self.filter_texture_bind_group_layout,
            &self.secondary_tex_view,
            &self.main_sampler,
        );

        let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);

        self.carbide_to_wgpu_matrix =
            Self::calculate_carbide_to_wgpu_matrix(dimension, scale_factor);

        let uniform_bind_group = matrix_to_uniform_bind_group(
            &self.device,
            &self.uniform_bind_group_layout,
            self.carbide_to_wgpu_matrix,
        );

        self.uniform_bind_group = uniform_bind_group;

        let last_verts: Vec<Vertex> = vec![
            Vertex::new_from_2d(0.0, 0.0, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0], MODE_IMAGE),
            Vertex::new_from_2d(
                size.width as f32 / scale_factor as f32,
                0.0,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                0.0,
                size.height as f32 / scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                size.width as f32 / scale_factor as f32,
                0.0,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                size.width as f32 / scale_factor as f32,
                size.height as f32 / scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 1.0],
                MODE_IMAGE,
            ),
            Vertex::new_from_2d(
                0.0,
                size.height as f32 / scale_factor as f32,
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0],
                MODE_IMAGE,
            ),
        ];

        self.queue.write_buffer(
            &self.second_vertex_buffer,
            0,
            bytemuck::cast_slice(&last_verts),
        );

        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: new_size.width,
                height: new_size.height,
                present_mode: PresentMode::Mailbox,
            },
        );

        println!("Resized window: {:?} to: {:?}", self.window_id, new_size);
    }
}

impl Clone for WGPUWindow {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Debug for WGPUWindow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}