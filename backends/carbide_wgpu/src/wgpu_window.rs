use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use cgmath::{Matrix4, Vector3};
use futures::executor::block_on;
use raw_window_handle::HasRawWindowHandle;
use wgpu::{Adapter, BindGroup, BindGroupLayout, Buffer, BufferUsages, CommandEncoder, Device, Extent3d, ImageCopyTexture, Instance, PipelineLayout, Queue, RenderPassDepthStencilAttachment, RenderPipeline, Sampler, ShaderModule, Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureFormat, TextureUsages, TextureView};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use carbide_core::{draw, Scene};
use carbide_core::draw::{ColorSpace, Dimension, DrawGradient, ImageId, Position, Rect, Scalar};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::event::{Event, KeyboardEvent, KeyboardEventContext, KeyboardEventHandler, MouseEvent, MouseEventContext, MouseEventHandler, OtherEventContext, OtherEventHandler, WindowEvent, WindowEventContext, WindowEventHandler};
use carbide_core::focus::Focusable;
use carbide_core::layout::{Layout, LayoutContext, Layouter};
use carbide_core::render::{Render, RenderContext};
use carbide_core::state::{IntoReadState, ReadState, StateSync};
use carbide_core::text::InnerTextContext;
use carbide_core::update::{Update, UpdateContext};
use carbide_core::widget::{AnyWidget, CommonWidget, FilterId, GradientRepeat, GradientType, Menu, Overlay, Rectangle, Widget, WidgetExt, WidgetId, ZStack};
use carbide_core::window::WindowId;
use carbide_winit::convert_mouse_cursor;
use carbide_winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use carbide_winit::window::{Window as WinitWindow, WindowBuilder};

use crate::{image_context, render_pass_ops, RenderPassOps};
use crate::application::{EVENT_LOOP, WINDOW_IDS};
use crate::bind_group_layouts::{atlas_bind_group_layout, filter_buffer_bind_group_layout, filter_texture_bind_group_layout, gradient_buffer_bind_group_layout, main_bind_group_layout, uniform_bind_group_layout, uniform_bind_group_layout2};
use crate::bind_groups::{filter_buffer_bind_group, gradient_buffer_bind_group, uniforms_to_bind_group, size_to_uniform_bind_group};
use crate::filter::Filter;
use crate::gradient::Gradient;
use crate::image::BindGroupExtended;
use crate::pipeline::create_pipelines;
use crate::render_context::{Uniform, WGPURenderContext};
use crate::render_pass_command::{RenderPass, RenderPassCommand, WGPUBindGroup};
use crate::render_pipeline_layouts::{filter_pipeline_layout, main_pipeline_layout, RenderPipelines};
use crate::render_target::RenderTarget;
use crate::renderer::atlas_cache_tex_desc;
use crate::samplers::main_sampler;
use crate::texture_atlas_command::TextureAtlasCommand;
use crate::textures::create_depth_stencil_texture;
use crate::vertex::Vertex;

const ZOOM: f32 = 1.0;

// The instance is a handle to our GPU
// BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
//let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
thread_local!(pub static INSTANCE: Arc<Instance> =
    Arc::new(Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    }))
);

thread_local!(pub static ADAPTER: Arc<Adapter> = {
    INSTANCE.with(|instance| {
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })).unwrap();

        //println!("{:#?}", adapter.limits());

        Arc::new(adapter)
    })
});
thread_local!(pub static DEVICE_QUEUE: (Arc<Device>, Arc<Queue>) = {
    ADAPTER.with(|adapter| {
        let mut limits = wgpu::Limits::default();
        limits.max_bind_groups = 5;

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::CLEAR_TEXTURE,
                limits,
            },
            None, // Trace path
        )).unwrap();

        (Arc::new(device), Arc::new(queue))
    })
});

thread_local!(pub static UNIFORM_BIND_GROUP_LAYOUT: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        uniform_bind_group_layout(&device)
    })
});

thread_local!(pub static UNIFORM_BIND_GROUP_LAYOUT2: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        uniform_bind_group_layout2(&device)
    })
});

thread_local!(pub static FILTER_TEXTURE_BIND_GROUP_LAYOUT: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        filter_texture_bind_group_layout(&device)
    })
});

thread_local!(pub static FILTER_BUFFER_BIND_GROUP_LAYOUT: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        filter_buffer_bind_group_layout(&device)
    })
});

thread_local!(pub static MAIN_TEXTURE_BIND_GROUP_LAYOUT: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        main_bind_group_layout(&device)
    })
});

thread_local!(pub static GRADIENT_BIND_GROUP_LAYOUT: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        gradient_buffer_bind_group_layout(&device)
    })
});

thread_local!(pub static ATLAS_BIND_GROUP_LAYOUT: BindGroupLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        atlas_bind_group_layout(&device)
    })
});

thread_local!(pub static MAIN_SHADER: ShaderModule = {
    DEVICE_QUEUE.with(|(device, _queue)| {
        device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"))
    })
});

thread_local!(pub static FILTER_SHADER: ShaderModule = {
    DEVICE_QUEUE.with(|(device, _queue)| {
        device.create_shader_module(wgpu::include_wgsl!("../shaders/filter.wgsl"))
    })
});

thread_local!(pub static RENDER_PIPELINE_LAYOUT: PipelineLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        UNIFORM_BIND_GROUP_LAYOUT.with(|uniform_bind_group_layout| {
            MAIN_TEXTURE_BIND_GROUP_LAYOUT.with(|main_bind_group_layout| {
                GRADIENT_BIND_GROUP_LAYOUT.with(|gradient_bind_group_layout| {
                    ATLAS_BIND_GROUP_LAYOUT.with(|atlas_bind_group_layout| {
                        main_pipeline_layout(
                            device,
                            main_bind_group_layout,
                            uniform_bind_group_layout,
                            gradient_bind_group_layout,
                            atlas_bind_group_layout,
                        )
                    })
                })
            })
        })
    })
});

thread_local!(pub static FILTER_RENDER_PIPELINE_LAYOUT: PipelineLayout = {
    DEVICE_QUEUE.with(|(device, _)| {
        FILTER_TEXTURE_BIND_GROUP_LAYOUT.with(|filter_texture_bind_group_layout| {
            FILTER_BUFFER_BIND_GROUP_LAYOUT.with(|filter_buffer_bind_group_layout| {
                UNIFORM_BIND_GROUP_LAYOUT.with(|uniform_bind_group_layout| {
                    filter_pipeline_layout(
                        device,
                        filter_texture_bind_group_layout,
                        filter_buffer_bind_group_layout,
                        uniform_bind_group_layout,
                    )
                })
            })
        })
    })
});

thread_local!(pub static MAIN_SAMPLER: Sampler = {
    DEVICE_QUEUE.with(|(device, _)| {
        main_sampler(device)
    })
});

thread_local!(pub static ATLAS_CACHE_TEXTURE: Texture = {
    DEVICE_QUEUE.with(|(device, _)| {
        let atlas_cache_tex_desc = atlas_cache_tex_desc(1024, 1024);
        device.create_texture(&atlas_cache_tex_desc)
    })
});

thread_local!(pub static ATLAS_CACHE_BIND_GROUP: BindGroup = {
    DEVICE_QUEUE.with(|(device, _)| {
        ATLAS_BIND_GROUP_LAYOUT.with(|layout| {
            ATLAS_CACHE_TEXTURE.with(|texture| {
                let view = texture.create_view(&Default::default());

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                    ],
                    label: None,
                })
            })
        })
    })
});

thread_local!(pub static BIND_GROUPS: RefCell<HashMap<ImageId, BindGroupExtended>> = {
    use image_context::create_bind_group;
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

thread_local!(pub static FILTER_BIND_GROUPS: RefCell<HashMap<FilterId, BindGroup>> = RefCell::new(HashMap::new()));

thread_local!(pub static PIPELINES: RefCell<Vec<(TextureFormat, RenderPipelines)>> = RefCell::new(vec![]));

pub struct WGPUWindow<T: ReadState<T=String>> {
    pub(crate) surface: Surface,

    pub(crate) render_pipelines_index: usize,

    pub(crate) depth_texture_view: TextureView,
    pub(crate) texture_size_bind_group: BindGroup,

    pub(crate) targets: Vec<RenderTarget>,

    pub(crate) uniform_bind_group: BindGroup,
    pub(crate) gradient_bind_group: BindGroup,

    pub(crate) carbide_to_wgpu_matrix: Matrix4<f32>,
    pub(crate) vertex_buffer: (Buffer, usize),
    pub(crate) second_vertex_buffer: Buffer,
    pub(crate) render_context: WGPURenderContext,
    inner: Rc<WinitWindow>,

    id: WidgetId,
    title: T,
    position: Position,
    dimension: Dimension,
    child: Box<dyn AnyWidget>,
    close_application_on_window_close: bool,
    visible: bool,
    window_menu: Option<Vec<Menu>>,
}

impl WGPUWindow<String> {
    pub fn new<T: IntoReadState<String>, C: Widget>(title: T, dimension: Dimension, child: C) -> Box<WGPUWindow<T::Output>> {
        let window_id = WindowId::new();
        let title = title.into_read_state();


        let child = ZStack::new((
            Rectangle::new().fill(EnvironmentColor::SystemBackground),
            Overlay::new("controls_popup_layer", child).steal_events(),
        )).boxed();

        let builder = WindowBuilder::new()
            .with_inner_size(Size::Logical(LogicalSize {
                width: dimension.width,
                height: dimension.height,
            }))
            //.with_title(title.clone())
            //.with_window_icon(loaded_icon)
            ;

        let inner = EVENT_LOOP.with(|a| {
            a.borrow().create_inner_window(builder)
        });

        inner.set_ime_allowed(true);

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


        let surface = INSTANCE.with(|instance| {
            unsafe { instance.create_surface(&inner) }
        }).unwrap();

        DEVICE_QUEUE.with(|(device, _queue)| {
            // Configure the surface with format, size and usage
            let surface_caps = ADAPTER.with(|adapter| {
                surface.get_capabilities(adapter)
            });

            surface.configure(
                &device,
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: TextureFormat::Bgra8UnormSrgb,
                    width: inner.inner_size().width,
                    height: inner.inner_size().height,
                    present_mode: surface_caps.present_modes[0],
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                },
            );

            let matrix = Self::calculate_carbide_to_wgpu_matrix(pixel_dimensions, scale_factor);

            let gradient_bind_group = GRADIENT_BIND_GROUP_LAYOUT.with(|gradient_bind_group_layout| {
                let gradient = DrawGradient {
                    colors: vec![],
                    ratios: vec![],
                    gradient_type: GradientType::Linear,
                    gradient_repeat: GradientRepeat::Clamp,
                    start: Default::default(),
                    end: Default::default(),
                    color_space: ColorSpace::Linear,
                };

                let gradient = Gradient::convert(&gradient);
                let gradient_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gradient Buffer"),
                        contents: &*gradient.as_bytes(),
                        usage: wgpu::BufferUsages::STORAGE,
                    });
                gradient_buffer_bind_group(
                    &device,
                    &gradient_bind_group_layout,
                    &gradient_buffer,
                )
            });

            let uniform_bind_group = UNIFORM_BIND_GROUP_LAYOUT.with(|uniform_bind_group_layout| {
                uniforms_to_bind_group(device, uniform_bind_group_layout, matrix, 0.0, 0.0, 0.0, false)
            });

            let texture_size_bind_group = UNIFORM_BIND_GROUP_LAYOUT2.with(|uniform_bind_group_layout| {
                size_to_uniform_bind_group(
                    device,
                    uniform_bind_group_layout,
                    pixel_dimensions.width,
                    pixel_dimensions.height,
                    scale_factor,
                )
            });

            let render_pipelines_index = PIPELINES.with(|pipelines| {
                let pipelines = &mut *pipelines.borrow_mut();

                let preferred_format = ADAPTER.with(|adapter| {
                    let surface_caps = surface.get_capabilities(adapter);
                    surface_caps.formats.iter()
                        .copied()
                        .filter(|f| f.is_srgb())
                        .next()
                        .unwrap_or(surface_caps.formats[0])
                });


                if let Some(index) = pipelines.iter().position(|(format, _)| format == &preferred_format) {
                    index
                } else {
                    let new_pipelines = create_pipelines(device, preferred_format);
                    pipelines.push((preferred_format, new_pipelines));
                    pipelines.len() - 1
                }
            });


            let depth_texture = create_depth_stencil_texture(&device, size.width, size.height);
            let depth_texture_view = depth_texture.create_view(&Default::default());

            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

            let second_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&Vertex::rect(size, scale_factor, ZOOM)),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

            Box::new(WGPUWindow {
                surface,
                render_pipelines_index,
                depth_texture_view,
                texture_size_bind_group,
                targets: vec![
                    RenderTarget::new(size.width, size.height)
                ],
                uniform_bind_group,
                gradient_bind_group,
                carbide_to_wgpu_matrix: matrix,
                vertex_buffer: (vertex_buffer, 0),
                second_vertex_buffer,

                render_context: WGPURenderContext::new(),
                inner: Rc::new(inner),
                id: WidgetId::new(),
                title,
                position: Default::default(),
                dimension: Default::default(),
                child,
                close_application_on_window_close: false,
                visible: true,
                window_menu: None
            })
        })
    }
}

impl<T: ReadState<T=String>> MouseEventHandler for WGPUWindow<T> {
    fn process_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
        let old_dimension = ctx.env.pixel_dimensions();
        let old_scale_factor = ctx.env.scale_factor();
        let old_window_handle = ctx.env.window_handle();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        ctx.env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        ctx.env.set_scale_factor(scale_factor);
        ctx.env.set_window_handle(Some(self.inner.raw_window_handle()));

        let id: u64 = self.inner.id().into();

        let new_ctx = &mut MouseEventContext {
            text: ctx.text,
            image: ctx.image,
            env: ctx.env,
            is_current: &(*ctx.window_id == id),
            window_id: ctx.window_id,
            consumed: ctx.consumed,
        };

        self.foreach_child_direct(&mut |child| {
            child.process_mouse_event(event, new_ctx);
            if *new_ctx.consumed {
                return;
            }
        });

        ctx.env.set_pixel_dimensions(old_dimension);
        ctx.env.set_scale_factor(old_scale_factor);
        ctx.env.set_window_handle(old_window_handle);
    }
}

impl<T: ReadState<T=String>> CommonWidget for WGPUWindow<T> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
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

impl<T: ReadState<T=String>> StateSync for WGPUWindow<T> {
    fn capture_state(&mut self, _env: &mut Environment) {

    }

    fn release_state(&mut self, _env: &mut Environment) {

    }
}

impl<T: ReadState<T=String>> Focusable for WGPUWindow<T> {}

impl<T: ReadState<T=String>> KeyboardEventHandler for WGPUWindow<T> {
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, ctx: &mut KeyboardEventContext) {
        let old_dimension = ctx.env.pixel_dimensions();
        let old_scale_factor = ctx.env.scale_factor();
        let old_window_handle = ctx.env.window_handle();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        ctx.env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        ctx.env.set_scale_factor(scale_factor);
        ctx.env.set_window_handle(Some(self.inner.raw_window_handle()));

        let id: u64 = self.inner.id().into();

        let new_ctx = &mut KeyboardEventContext {
            text: ctx.text,
            image: ctx.image,
            env: ctx.env,
            is_current: &(*ctx.window_id == id),
            window_id: ctx.window_id,
        };

        self.foreach_child_direct(&mut |child| {
            child.process_keyboard_event(event, new_ctx);
        });

        ctx.env.set_pixel_dimensions(old_dimension);
        ctx.env.set_scale_factor(old_scale_factor);
        ctx.env.set_window_handle(old_window_handle);
    }
}

impl<T: ReadState<T=String>> OtherEventHandler for WGPUWindow<T> {
    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        let old_dimension = ctx.env.pixel_dimensions();
        let old_scale_factor = ctx.env.scale_factor();
        let old_window_handle = ctx.env.window_handle();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        ctx.env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        ctx.env.set_scale_factor(scale_factor);
        ctx.env.set_window_handle(Some(self.inner.raw_window_handle()));

        self.foreach_child_direct(&mut |child| {
            child.process_other_event(event, ctx);
        });

        // Set the cursor of the window.
        self.inner.set_cursor_icon(convert_mouse_cursor(ctx.env.cursor()));

        ctx.env.set_pixel_dimensions(old_dimension);
        ctx.env.set_scale_factor(old_scale_factor);
        ctx.env.set_window_handle(old_window_handle);
    }
}

impl<T: ReadState<T=String>> WindowEventHandler for WGPUWindow<T> {
    fn handle_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        match event {
            WindowEvent::Resize(size) => {
                self.resize(LogicalSize::new(size.width, size.height).to_physical(2.0), ctx.env);
                self.request_redraw();
            }
            WindowEvent::Focus => {
                #[cfg(target_os = "macos")]
                {
                    use carbide_macos::NSMenu;
                    use carbide_macos::NSMenuItem;

                    // The outer menu is not visible, but only a container for
                    // other menus.

                    if let Some(menu) = &self.window_menu {
                        let mut outer_menu = NSMenu::new("");

                        for m in menu {
                            let item = NSMenuItem::new(m.name(), None)
                                .set_submenu(NSMenu::from(m, ctx.env));

                            outer_menu = outer_menu.add_item(item);
                        }

                        outer_menu.set_as_main_menu();
                    } else {
                        // Todo: Set default application menu
                    }
                }
            }
            WindowEvent::CloseRequested => {
                self.visible = false;
                if self.close_application_on_window_close {
                    ctx.env.close_application();
                } else {
                    self.inner.set_visible(false);
                }
            }
            _ => ()
        }
    }

    fn process_window_event(&mut self, event: &WindowEvent, ctx: &mut WindowEventContext) {
        let old_dimension = ctx.env.pixel_dimensions();
        let old_scale_factor = ctx.env.scale_factor();
        let old_window_handle = ctx.env.window_handle();
        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();

        ctx.env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        ctx.env.set_scale_factor(scale_factor);
        ctx.env.set_window_handle(Some(self.inner.raw_window_handle()));

        let id: u64 = self.inner.id().into();

        let is_current = *ctx.window_id == id;

        if is_current {
            self.capture_state(ctx.env);
            self.handle_window_event(event, ctx);
            self.release_state(ctx.env);
        }

        let new_ctx = &mut WindowEventContext {
            text: ctx.text,
            image: ctx.image,
            env: ctx.env,
            is_current: &is_current,
            window_id: ctx.window_id,
        };

        self.foreach_child_direct(&mut |child| {
            child.process_window_event(event, new_ctx);
        });

        ctx.env.set_pixel_dimensions(old_dimension);
        ctx.env.set_scale_factor(old_scale_factor);
        ctx.env.set_window_handle(old_window_handle);
    }
}

impl<T: ReadState<T=String>> Layout for WGPUWindow<T> {
    fn calculate_size(&mut self, _requested_size: Dimension, _ctx: &mut LayoutContext) -> Dimension {
        Dimension::new(0.0, 0.0)
    }

    fn position_children(&mut self, _ctx: &mut LayoutContext) {}
}

impl<T: ReadState<T=String>> Update for WGPUWindow<T> {
    fn update(&mut self, _ctx: &mut UpdateContext) {}

    fn process_update(&mut self, _ctx: &mut UpdateContext) {}
}

impl<T: ReadState<T=String>> Render for WGPUWindow<T> {
    fn render(&mut self, context: &mut RenderContext) {
        let old_scale_factor = context.env.scale_factor();
        let old_pixel_dimensions = context.env.pixel_dimensions();

        let scale_factor = self.inner.scale_factor();
        let physical_dimensions = self.inner.inner_size();
        let logical_dimensions = physical_dimensions.to_logical(scale_factor);
        let dimensions = Dimension::new(logical_dimensions.width, logical_dimensions.height);

        context.env.capture_time();

        context.env.set_pixel_dimensions(Dimension::new(physical_dimensions.width as f64, physical_dimensions.height as f64));
        context.env.set_scale_factor(scale_factor);

        // Update children
        self.child.process_update(&mut UpdateContext {
            text: context.text,
            image: context.image,
            env: context.env,
        });

        // Calculate size
        self.child.calculate_size(dimensions, &mut LayoutContext {
            text: context.text,
            image: context.image,
            env: context.env,
        });

        // Position children
        let layout = context.env.root_alignment();
        (layout.positioner())(Position::new(0.0, 0.0), dimensions, &mut self.child);

        self.child.position_children(&mut LayoutContext {
            text: context.text,
            image: context.image,
            env: context.env,
        });

        // Render the children
        self.render_context.start(Rect::new(Position::origin(), dimensions));

        context.text.prepare_render();

        self.child.render(&mut RenderContext {
            render: &mut self.render_context,
            text: context.text,
            image: context.image,
            env: context.env,
        });

        let render_passes = self.render_context.finish();

        let target_count = self.render_context.target_count();
        if self.targets.len() < target_count {
            for _ in self.targets.len()..target_count {
                self.targets.push(RenderTarget::new(self.inner.inner_size().width, self.inner.inner_size().height));
            }
        }

        //println!("\nContext: {:#?}", render_passes);
        //println!("Vertices: {:#?}", &self.render_context.vertices()[0..10]);
        //println!("Targets: {:#?}", &self.targets.len());

        let mut uniform_bind_groups = vec![];
        let mut gradient_bind_groups = vec![];

        DEVICE_QUEUE.with(|(device, queue)| {
            Self::ensure_vertices_in_buffer(device, queue, self.render_context.vertices(), &mut self.vertex_buffer.0, &mut self.vertex_buffer.1);

            UNIFORM_BIND_GROUP_LAYOUT.with(|uniform_bind_group_layout| {
                Self::ensure_uniforms_in_buffer(device, &self.carbide_to_wgpu_matrix, self.render_context.uniforms(), uniform_bind_group_layout, &mut uniform_bind_groups);
                Self::ensure_gradients_in_buffer(device, self.render_context.gradients(), uniform_bind_group_layout, &mut gradient_bind_groups);
            });
        });

        if self.visible {
            {
                self.title.sync(context.env);

                let current = &*self.title.value();
                if &self.inner.title() != current {
                    self.inner.set_title(current);
                }
            }

            self.inner.set_cursor_icon(convert_mouse_cursor(context.env.cursor()));

            match self.render_inner(render_passes, uniform_bind_groups, gradient_bind_groups, context.text, context.env) {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SurfaceError::Lost) => {
                    println!("Swap chain lost");
                    self.resize(self.inner.inner_size(), context.env);
                    self.request_redraw();
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    println!("Swap chain out of memory");
                    context.env.close_application();
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => {
                    // We request a redraw the next frame
                    self.request_redraw();
                    eprintln!("{:?}", e)
                }
            }
        }

        // Reset the environment
        context.env.set_pixel_dimensions(old_pixel_dimensions);
        context.env.set_scale_factor(old_scale_factor);
    }
}

impl<T: ReadState<T=String>> AnyWidget for WGPUWindow<T> {}

impl<T: ReadState<T=String>> Scene for WGPUWindow<T> {
    /// Request the window to redraw next frame
    fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}

impl<T: ReadState<T=String>> WGPUWindow<T> {
    pub fn menu(mut self, menu: Vec<Menu>) -> Box<Self> {
        self.window_menu = Some(menu);
        Box::new(self)
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

    fn update_atlas_cache(device: &Device, encoder: &mut CommandEncoder, ctx: &mut dyn InnerTextContext) {
        ATLAS_CACHE_TEXTURE.with(|atlas_cache_tex| {
            ctx.update_cache(&mut |image| {
                TextureAtlasCommand {
                    texture_atlas_buffer: image.as_bytes(),
                    texture_atlas_texture: atlas_cache_tex,
                    width: image.width(),
                    height: image.height(),
                }.load_buffer_and_encode(device, encoder);
            });
        });
    }

    fn update_filter_bind_groups(device: &Device, filter_bind_groups: &mut HashMap<FilterId, BindGroup>, env: &mut Environment, size: PhysicalSize<u32>) {
        filter_bind_groups.retain(|id, _| env.filters().contains_key(id));

        for (filter_id, filter) in env.filters() {
            if !filter_bind_groups.contains_key(filter_id) {
                let mut filter: Filter = filter.clone().into();
                filter.texture_size = [size.width as f32, size.height as f32];

                let filter_buffer =
                    device
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("Filter Buffer"),
                            contents: &*filter.as_bytes(),
                            usage: wgpu::BufferUsages::STORAGE,
                        });
                let filter_buffer_bind_group =
                    FILTER_BUFFER_BIND_GROUP_LAYOUT.with(|filter_buffer_bind_group_layout| {
                        filter_buffer_bind_group(
                            device,
                            filter_buffer_bind_group_layout,
                            &filter_buffer,
                        )
                    });
                filter_bind_groups
                    .insert(*filter_id, filter_buffer_bind_group);
            }
        }
    }

    fn ensure_vertices_in_buffer(device: &Device, queue: &Queue, vertices: &Vec<Vertex>, vertex_buffer: &mut Buffer, buffer_size: &mut usize) {
        if vertices.len() <= *buffer_size {
            // There is space in the current vertex buffer
            queue.write_buffer(vertex_buffer, 0, bytemuck::cast_slice(vertices));
        } else {
            // We need to create a new and larger vertex buffer
            let new_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            *vertex_buffer = new_vertex_buffer;
            *buffer_size = vertices.len();
        }
    }

    fn ensure_uniforms_in_buffer(device: &Device, carbide_to_wgpu_matrix: &Matrix4<f32>, uniforms: &Vec<Uniform>, uniform_bind_group_layout: &BindGroupLayout, uniform_bind_groups: &mut Vec<BindGroup>) {
        for uniform in uniforms {
            let transformed_matrix = carbide_to_wgpu_matrix * uniform.transform;

            let new_bind_group = uniforms_to_bind_group(
                device,
                uniform_bind_group_layout,
                transformed_matrix,
                uniform.hue_rotation,
                uniform.saturation_shift,
                uniform.luminance_shift,
                uniform.color_invert,
            );

            uniform_bind_groups.push(new_bind_group);
        }
    }

    fn ensure_gradients_in_buffer(device: &Device, gradients: &Vec<Gradient>, _uniform_bind_group_layout: &BindGroupLayout, gradient_bind_groups: &mut Vec<BindGroup>) {
        for gradient in gradients {
            GRADIENT_BIND_GROUP_LAYOUT.with(|gradient_bind_group_layout| {
                let gradient_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Gradient Buffer"),
                        contents: &*gradient.as_bytes(),
                        usage: wgpu::BufferUsages::STORAGE,
                    });
                gradient_bind_groups.push(
                    gradient_buffer_bind_group(
                        &device,
                        &gradient_bind_group_layout,
                        &gradient_buffer,
                    )
                );

            })
        }
    }

    fn render_texture_to_swapchain(&self, encoder: &mut CommandEncoder, render_pipeline_no_mask: &RenderPipeline, output: &SurfaceTexture, atlas_cache_bind_group: &BindGroup, bind_groups: &HashMap<ImageId, BindGroupExtended>) {
        let instance_range = 0..1;

        let (color_op, stencil_op, depth_op) = render_pass_ops(RenderPassOps::Start);

        let frame_view = output.texture.create_view(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame_view, // Here is the render target
                resolve_target: None,
                ops: color_op,
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(depth_op),
                stencil_ops: Some(stencil_op),
            }),
        });

        render_pass.set_pipeline(render_pipeline_no_mask);
        render_pass.set_vertex_buffer(0, self.second_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.targets[0].bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.gradient_bind_group, &[]);
        render_pass.set_bind_group(3, atlas_cache_bind_group, &[]);
        render_pass.set_bind_group(4, &bind_groups[&ImageId::default()].bind_group, &[]);

        render_pass.draw(0..6, instance_range);
    }

    fn process_render_passes(
        &self,
        render_passes: Vec<RenderPass>,
        encoder: &mut CommandEncoder,
        render_pipelines: &RenderPipelines,
        bind_groups: &HashMap<ImageId, BindGroupExtended>,
        uniform_bind_groups: &Vec<BindGroup>,
        gradient_bind_groups: &Vec<BindGroup>,
        filter_bind_groups: &HashMap<FilterId, BindGroup>,
        size: PhysicalSize<u32>,
        scale_factor: Scalar,
        atlas_cache_bind_group: &BindGroup,
    ) {
        let instance_range = 0..1;
        let mut stencil_level = 0;
        let mut first_pass = true;

        let mut current_target_view;
        let mut current_main_render_pipeline = &render_pipelines.render_pipeline_no_mask;
        let current_vertex_buffer_slice = self.vertex_buffer.0.slice(..);
        let mut current_uniform_bind_group = &self.uniform_bind_group;
        let mut current_gradient_bind_group = &self.gradient_bind_group;
        let mut invalid_scissor = false;

        // println!("{:#?}", render_passes);

        for command in render_passes {
            match command {
                RenderPass::Normal { commands: inner, target_index: index } => {
                    current_target_view = &self.targets[index].view;

                    if inner.len() == 0 {
                        continue;
                    }
                    let (color_op, stencil_op, depth_op) = if first_pass {
                        first_pass = false;
                        render_pass_ops(RenderPassOps::Start)
                    } else {
                        render_pass_ops(RenderPassOps::Middle)
                    };

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: current_target_view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        })],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: Some(depth_op),
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_pipeline(current_main_render_pipeline);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &bind_groups[&ImageId::default()].bind_group, &[]);
                    render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(2, current_gradient_bind_group, &[]);
                    render_pass.set_bind_group(3, atlas_cache_bind_group, &[]);
                    render_pass.set_bind_group(4, &bind_groups[&ImageId::default()].bind_group, &[]);


                    for inner_command in inner {
                        match inner_command {
                            RenderPassCommand::SetBindGroup { bind_group } => {
                                match bind_group {
                                    WGPUBindGroup::Default => {
                                        render_pass.set_bind_group(0, &bind_groups[&ImageId::default()].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Image(id) => {
                                        render_pass.set_bind_group(0, &bind_groups[&id].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Target(index) => {
                                        render_pass.set_bind_group(0, &self.targets[index].bind_group, &[]);
                                    }
                                }
                            }
                            RenderPassCommand::SetScissor { rect } => {
                                let x = (rect.left() * scale_factor).max(0.0) as u32;
                                let y = (rect.bottom() * scale_factor).max(0.0) as u32;
                                let width = (rect.width() * scale_factor) as u32;
                                let height = (rect.height() * scale_factor) as u32;

                                invalid_scissor = width <= 0 || height <= 0;

                                if !invalid_scissor {
                                    render_pass.set_scissor_rect(x, y, width, height);
                                }
                            }
                            RenderPassCommand::Draw { vertex_range } => {
                                if invalid_scissor {
                                    continue;
                                }
                                render_pass.draw(vertex_range, instance_range.clone());
                            }
                            RenderPassCommand::Stencil { vertex_range } => {
                                stencil_level += 1;
                                render_pass.set_pipeline(&render_pipelines.render_pipeline_add_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                current_main_render_pipeline = &render_pipelines.render_pipeline_in_mask;
                                render_pass.set_pipeline(current_main_render_pipeline);
                                render_pass.set_stencil_reference(stencil_level);
                            }
                            RenderPassCommand::DeStencil { vertex_range } => {
                                stencil_level -= 1;
                                render_pass.set_pipeline(&render_pipelines.render_pipeline_remove_mask);
                                render_pass.draw(vertex_range, instance_range.clone());
                                render_pass.set_stencil_reference(stencil_level);
                                if stencil_level == 0 {
                                    current_main_render_pipeline = &render_pipelines.render_pipeline_no_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                } else {
                                    current_main_render_pipeline = &render_pipelines.render_pipeline_in_mask;
                                    render_pass.set_pipeline(current_main_render_pipeline);
                                }
                            }
                            RenderPassCommand::Uniform {
                                uniform_bind_group_index,
                            } => {
                                current_uniform_bind_group = &uniform_bind_groups[uniform_bind_group_index];
                                render_pass.set_bind_group(1, current_uniform_bind_group, &[]);
                            }
                            RenderPassCommand::Gradient { index } => {
                                current_gradient_bind_group = &gradient_bind_groups[index];
                                render_pass.set_bind_group(2, current_gradient_bind_group, &[]);
                            }
                            RenderPassCommand::SetMaskBindGroup { bind_group } => {
                                match bind_group {
                                    WGPUBindGroup::Default => {
                                        render_pass.set_bind_group(4, &bind_groups[&ImageId::default()].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Image(id) => {
                                        render_pass.set_bind_group(4, &bind_groups[&id].bind_group, &[]);
                                    }
                                    WGPUBindGroup::Target(index) => {
                                        render_pass.set_bind_group(4, &self.targets[index].bind_group, &[]);
                                    }
                                }
                            }
                        }
                    }
                }
                RenderPass::Filter {
                    vertex_range,
                    filter_id,
                    source_id,
                    target_id,
                    initial_copy
                } => {
                    if invalid_scissor {
                        continue;
                    }

                    if initial_copy {
                        encoder.copy_texture_to_texture(
                            ImageCopyTexture {
                                texture: &self.targets[target_id].texture,
                                mip_level: 0,
                                origin: Default::default(),
                                aspect: Default::default(),
                            },
                            ImageCopyTexture {
                                texture: &self.targets[source_id].texture,
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
                    }

                    let (color_op, stencil_op, depth_op) = render_pass_ops(RenderPassOps::Middle);

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &self.targets[target_id].view, // Here is the render target
                            resolve_target: None,
                            ops: color_op,
                        })],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: Some(depth_op),
                            stencil_ops: Some(stencil_op),
                        }),
                    });

                    render_pass.set_pipeline(&render_pipelines.render_pipeline_in_mask_filter);
                    render_pass.set_stencil_reference(stencil_level);
                    render_pass.set_vertex_buffer(0, current_vertex_buffer_slice);
                    render_pass.set_bind_group(0, &self.targets[source_id].bind_group, &[]);
                    render_pass.set_bind_group(1, &filter_bind_groups[&filter_id], &[]);
                    render_pass.set_bind_group(2, current_uniform_bind_group, &[]);
                    render_pass.set_bind_group(3, &self.targets[source_id].bind_group, &[]);

                    render_pass.draw(vertex_range, instance_range.clone());
                }
                RenderPass::Clear { target_index: index } => {
                    encoder.clear_texture(&self.targets[index].texture, &Default::default())
                }
            };
        }
    }

    fn render_inner(&self, render_passes: Vec<RenderPass>, uniform_bind_groups: Vec<BindGroup>, gradient_bind_groups: Vec<BindGroup>, ctx: &mut dyn InnerTextContext, env: &mut Environment) -> Result<(), wgpu::SurfaceError> {
        DEVICE_QUEUE.with(|(device, queue)| {
            BIND_GROUPS.with(|bind_groups| {
                ATLAS_CACHE_BIND_GROUP.with(|atlas_cache_bind_group| {
                    FILTER_BIND_GROUPS.with(|filter_bind_groups| {
                        PIPELINES.with(|render_pipelines| {
                            let render_pipelines = &render_pipelines.borrow()[self.render_pipelines_index].1;
                            let filter_bind_groups = &mut *filter_bind_groups.borrow_mut();
                            let bind_groups = &mut *bind_groups.borrow_mut();

                            let mut encoder = device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("Render Encoder"),
                                });

                            let size = self.inner.inner_size();

                            // Handle update of atlas cache
                            Self::update_atlas_cache(device, &mut encoder, ctx);

                            // Update filter bind groups
                            Self::update_filter_bind_groups(device, filter_bind_groups, env, size);

                            // Ensure the images are added as bind groups
                            //Self::ensure_images_exist_as_bind_groups(device, queue, bind_groups, env);

                            self.process_render_passes(render_passes, &mut encoder, render_pipelines, bind_groups, &uniform_bind_groups, &gradient_bind_groups, filter_bind_groups, size, env.scale_factor(), atlas_cache_bind_group);

                            // This blocks until a new frame is available.
                            let output = self.surface.get_current_texture()?;

                            // Render from the texture to the swap chain
                            self.render_texture_to_swapchain(&mut encoder, &render_pipelines.render_pipeline_no_mask, &output, atlas_cache_bind_group, bind_groups);

                            // submit will accept anything that implements IntoIter
                            queue.submit(std::iter::once(encoder.finish()));
                            output.present();
                            Ok(())
                        })
                    })
                })
            })
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, _: &mut Environment) {
        DEVICE_QUEUE.with(|(device, queue)| {
            let size = new_size;
            //env.set_pixel_dimensions(size.width as f64);
            //env.set_pixel_height(size.height as f64);
            //self.ui.compound_and_add_event(Input::Redraw);

            let depth_texture =
                create_depth_stencil_texture(device, new_size.width, new_size.height);
            let depth_texture_view = depth_texture.create_view(&Default::default());

            self.depth_texture_view = depth_texture_view;

            self.targets = vec![
                RenderTarget::new(new_size.width, new_size.height)
            ];

            let scale_factor = self.inner.scale_factor();

            self.texture_size_bind_group =
                UNIFORM_BIND_GROUP_LAYOUT2.with(|uniform_bind_group_layout| {
                    size_to_uniform_bind_group(
                        device,
                        uniform_bind_group_layout,
                        size.width as f64,
                        size.height as f64,
                        scale_factor,
                    )
                });

            let dimension = Dimension::new(new_size.width as Scalar, new_size.height as Scalar);

            self.carbide_to_wgpu_matrix =
                Self::calculate_carbide_to_wgpu_matrix(dimension, scale_factor);

            let uniform_bind_group =
                UNIFORM_BIND_GROUP_LAYOUT.with(|uniform_bind_group_layout| {
                    uniforms_to_bind_group(
                        device,
                        uniform_bind_group_layout,
                        self.carbide_to_wgpu_matrix,
                        0.0,
                        0.0,
                        0.0,
                        false
                    )
                });

            self.uniform_bind_group = uniform_bind_group;

            FILTER_BIND_GROUPS.with(|a| {
                a.borrow_mut().clear();
            });

            queue.write_buffer(
                &self.second_vertex_buffer,
                0,
                bytemuck::cast_slice(&Vertex::rect(size, scale_factor, ZOOM)),
            );

            let surface_caps = ADAPTER.with(|adapter| {
                self.surface.get_capabilities(adapter)
            });

            self.surface.configure(
                device,
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    format: TextureFormat::Bgra8UnormSrgb,
                    width: new_size.width,
                    height: new_size.height,
                    present_mode: surface_caps.present_modes[0],
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                },
            );
        })
    }
}

impl<T: ReadState<T=String>> Clone for WGPUWindow<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T: ReadState<T=String>> Debug for WGPUWindow<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .field("child", &self.child)
            .finish()
    }
}

impl<T: ReadState<T=String>> Drop for WGPUWindow<T> {
    fn drop(&mut self) {
        WINDOW_IDS.with(|a| {
            a.borrow_mut().remove(&self.inner.id());
        });
    }
}