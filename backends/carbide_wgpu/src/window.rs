use crate::image::Image;
use carbide_core::mesh::mesh::Mesh;
use carbide_core::{Ui, Rect};
use carbide_core::image_map::{ImageMap, Id};
use wgpu::{Texture, BindGroupLayout};
use std::collections::HashMap;
use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
use carbide_core::mesh::vertex::Vertex;
use crate::renderer::glyph_cache_tex_desc;
use carbide_core::event::input::Input;
use crate::render_pass_command::{RenderPassCommand, create_render_pass_commands};
use wgpu::util::DeviceExt;
use crate::glyph_cache_command::GlyphCacheCommand;
use winit::event::{WindowEvent, Event, KeyboardInput, ElementState, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use carbide_core::widget::primitive::Widget;
use carbide_core::text::font::Error;
use winit::window::{WindowBuilder, Icon};
use carbide_core::text::font;
use winit::dpi::{Size, PhysicalSize, PhysicalPosition};
use std::path::PathBuf;
use carbide_core::state::global_state::GlobalState;
use carbide_core::prelude::Rectangle;
use carbide_core::state::environment_color::EnvironmentColor;
use carbide_core::mesh::DEFAULT_GLYPH_CACHE_DIMS;

// Todo: Look in to multisampling: https://github.com/gfx-rs/wgpu-rs/blob/v0.6/examples/msaa-line/main.rs
pub struct Window<T: GlobalState> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    mesh: Mesh,
    ui: Ui<T>,
    image_map: ImageMap<Image>,
    glyph_cache_tex: Texture,
    bind_groups: HashMap<Id, DiffuseBindGroup>,
    texture_bind_group_layout: BindGroupLayout,
    state: T,
    inner_window: winit::window::Window,
    event_loop: Option<EventLoop<()>>
}

impl<T: GlobalState> carbide_core::window::TWindow<T> for Window<T> {

    fn add_font(&mut self, path: &str) -> Result<font::Id, Error> {
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join(path);

        self.ui.environment.insert_font_from_file(font_path)
    }

    fn add_image(&mut self, path: &str) -> Result<Id, Error> {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        let image = Image::new(assets.join(path), &self.device, &self.queue);

        let information = image.image_information();

        let id = self.image_map.insert(image);

        self.ui.environment.insert_image(id, information);

        Ok(id)
    }

    fn set_widgets(&mut self, w: Box<dyn Widget<T>>) {
        self.ui.widgets = Rectangle::initialize(vec![w])
            .fill(EnvironmentColor::SystemBackground.into());
    }
}

impl<T: GlobalState> Window<T> {

    pub fn path_to_assets(path: &str) -> PathBuf {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();
        assets.join(path)
    }

    pub async fn new(title: String, width: u32, height: u32, icon: Option<PathBuf>, state: T) -> Self {

        let event_loop = EventLoop::new();

        let loaded_icon = if let Some(path) = icon {
            let rgba_logo_image = image::open(path)
                .expect("Couldn't load logo")
                .to_rgba();

            let width = rgba_logo_image.width();
            let height = rgba_logo_image.height();

            Some(Icon::from_rgba(rgba_logo_image.into_raw(), width, height).unwrap())
        } else {
            None
        };

        let inner_window = WindowBuilder::new()
            .with_inner_size(Size::Physical(PhysicalSize{ width, height }))
            .with_title(title)
            .with_window_icon(loaded_icon)
            .build(&event_loop)
            .unwrap();

        if let Some(monitor) = inner_window.current_monitor() {
            let size = monitor.size();

            let outer_window_size = inner_window.outer_size();

            let position = PhysicalPosition::new(
                size.width / 2 - outer_window_size.width / 2,
                size.height / 2 - outer_window_size.height / 2
            );

            inner_window.set_outer_position(position);
        }



        let size = inner_window.inner_size();

        let ui: Ui<T> = carbide_core::UiBuilder::new([inner_window.inner_size().width as f64, inner_window.inner_size().height as f64]).build();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&inner_window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None, // Trace path
        ).await.unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // CHANGED!

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                        },
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
                ],
                label: Some("texture_bind_group_layout"),
            }
        );

        let text_cache_tex_desc = glyph_cache_tex_desc(DEFAULT_GLYPH_CACHE_DIMS);
        let glyph_cache_tex = device.create_texture(&text_cache_tex_desc);

        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();

        let image = Image::new(assets.join("images/happy-tree.png"), &device, &queue);

        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"), // NEW!
                bind_group_layouts: &[&texture_bind_group_layout], // NEW!
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main", // 1.
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor { // 2.
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(
                wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None, // Todo fix mesh to always be CCW, then we can cull backfaces
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                }
            ),
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
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
                },
            ],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            depth_stencil_state: None, // 2.
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16, // 3.
                vertex_buffers: &[
                    Vertex::desc(),
                ], // 4.
            },
            sample_count: 1, // 5.
            sample_mask: !0, // 6.
            alpha_to_coverage_enabled: false, // 7.
        });

        let bind_groups = HashMap::new();

        let diffuse_bind_group = new_diffuse(&device, &image, &glyph_cache_tex, &texture_bind_group_layout);

        let mesh = Mesh::with_glyph_cache_dimensions(DEFAULT_GLYPH_CACHE_DIMS);

        let image_map = ImageMap::new();

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            diffuse_bind_group,
            mesh,
            ui,
            image_map,
            glyph_cache_tex,
            bind_groups,
            texture_bind_group_layout,
            state,
            inner_window,
            event_loop: Some(event_loop)
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.ui.set_window_width(self.size.width as f64);
        self.ui.set_window_height(self.size.height as f64);
        self.ui.handle_event(Input::Redraw, &mut self.state);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match convert_window_event(event, &self.inner_window) {
            None => false,
            Some(input) => {
                self.ui.handle_event(input, &mut self.state);
                false
            },
        }
    }

    fn update(&mut self) {
        self.ui.delegate_events(&mut self.state);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let primitives = self.ui.draw();

        let fill = self.mesh.fill(Rect::new([0.0,0.0], [self.size.width as f64, self.size.height as f64]), 1.0, &self.image_map, primitives).unwrap();

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

        let commands = create_render_pass_commands(&self.diffuse_bind_group, &mut self.bind_groups, &self.image_map, &self.mesh, &self.device, &self.glyph_cache_tex, &self.texture_bind_group_layout);

        //println!("{:#?}", self.mesh.vertices());

        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(self.mesh.vertices()),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
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
                        }
                    }
                ],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            let instance_range = 0..1;
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
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())

    }

    pub fn run_event_loop(mut self) {

        // Make the state sync on event loop run
        self.input(&WindowEvent::Focused(true));

        let mut event_loop = None;

        std::mem::swap(&mut event_loop, &mut self.event_loop);

        event_loop
            .expect("The eventloop should be retrieved")
            .run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.inner_window.id() => if !self.input(event) { // UPDATED!
                    match event {

                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input,
                            ..
                        } => {
                            match input {
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
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(_) => {
                    self.update();
                    match self.render() {
                        Ok(_) => {}
                        // Recreate the swap_chain if lost
                        Err(wgpu::SwapChainError::Lost) => self.resize(self.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
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
        });
    }
}

carbide_winit::v023_conversion_fns!();