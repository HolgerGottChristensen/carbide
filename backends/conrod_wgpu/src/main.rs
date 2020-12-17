mod texture;
mod renderer;
mod image;
mod render;
mod render_pass_command;
mod glyph_cache_command;
mod diffuse_bind_group;
mod pipeline;

use winit::window::{Window, WindowBuilder};
use winit::event::{WindowEvent, Event, KeyboardInput, ElementState, VirtualKeyCode};
use winit::event_loop::EventLoop;
use futures::executor::block_on;
use winit::event_loop::ControlFlow;
use wgpu::util::DeviceExt;
use crate::diffuse_bind_group::{DiffuseBindGroup, new_diffuse};
use crate::image::Image;
use conrod_core::mesh::vertex::Vertex;
use conrod_core::mesh::mesh::{Mesh, DEFAULT_GLYPH_CACHE_DIMS};
use conrod_core::{Rect, Color, mesh, Ui};
use conrod_core::image::{ImageMap, Id};
use conrod_core::render::cprimitives::CPrimitives;
use conrod_core::render::primitive::Primitive;
use conrod_core::render::primitive_kind::PrimitiveKind;
use conrod_core::widget::{Rectangle, Text};
use conrod_core::color::{GREEN, RED};
use conrod_core::event::input::Input;
use conrod_core::widget::primitive::widget::WidgetExt;
use crate::renderer::glyph_cache_tex_desc;
use wgpu::{Texture, BindGroup, BindGroupLayout};
use crate::glyph_cache_command::GlyphCacheCommand;
use conrod_core::widget::primitive::v_stack::VStack;
use std::collections::HashMap;
use crate::render_pass_command::{create_render_pass_commands, RenderPassCommand};


const GLYPH_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
const GLYPH_TEX_COMPONENT_TY: wgpu::TextureComponentType = wgpu::TextureComponentType::Uint;
const DEFAULT_IMAGE_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;


const VERTICES: &[Vertex] = &[
    // Changed
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], rgba: [1.0,0.0,0.0,1.0], mode: 0 }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], rgba: [1.0,0.0,0.0,1.0], mode: 0 }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397057], rgba: [1.0,0.0,0.0,1.0], mode: 0}, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732911], rgba: [1.0,0.0,0.0,1.0], mode: 0}, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], rgba: [1.0,0.0,0.0,1.0], mode: 0}, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

struct State {
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
    diffuse_texture: Image,
    mesh: Mesh,
    ui: Ui<String>,
    image_map: ImageMap<Image>,
    glyph_cache_tex: Texture,
    bind_groups: HashMap<Id, DiffuseBindGroup>,
    texture_bind_group_layout: BindGroupLayout,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window, ui: Ui<String>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
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

        let mut image_map = ImageMap::new();
        image_map.insert(image);

        let image = Image::new(assets.join("images/rust.png"), &device, &queue);

        image_map.insert(image);

        let image = Image::new(assets.join("images/happy-tree.png"), &device, &queue);
        let diffuse_bind_group = new_diffuse(&device, &image, &glyph_cache_tex, &texture_bind_group_layout);

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

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;
        let num_vertices = VERTICES.len() as u32;


        let mut bind_groups = HashMap::new();
        bind_groups.insert(Id(0), diffuse_bind_group);

        let diffuse_bind_group = new_diffuse(&device, &image, &glyph_cache_tex, &texture_bind_group_layout);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture: image,
            mesh: Mesh::with_glyph_cache_dimensions(mesh::mesh::DEFAULT_GLYPH_CACHE_DIMS),
            ui,
            image_map,
            glyph_cache_tex,
            bind_groups,
            texture_bind_group_layout
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.ui.win_w = self.size.width as f64;
        self.ui.win_h = self.size.height as f64;
        let mut str = String::from("Hejsa");
        self.ui.handle_event(Input::Redraw, &mut str);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {

    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self
            .swap_chain
            .get_current_frame()?
            .output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        if let (primitives, cprims) = self.ui.draw() {
            let fill = self.mesh.fill(Rect::new([0.0,0.0], [self.size.width as f64, self.size.height as f64]), 1.0, &self.image_map, cprims).unwrap();

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
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
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
                        //render_pass.set_scissor_rect(x, y, w, h);
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
}



fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut ui: Ui<String> = conrod_core::UiBuilder::new([window.inner_size().width as f64, window.inner_size().height as f64]).build();

    ui.widgets = Rectangle::initialize(vec![
        Rectangle::initialize(vec![
            VStack::initialize(
                vec![
                    conrod_core::widget::Text::initialize("Majs med jokejjkjjjj".into(), vec![]),
                    conrod_core::widget::Image::new(Id(0), [50.0,50.0], vec![]),
                    conrod_core::widget::Image::new(Id(1), [150.0,150.0], vec![])
                ]
            )

        ]).fill(RED).frame(200.0, 600.0)
    ]).fill(GREEN);

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path);

    let mut state = block_on(State::new(&window, ui));
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) { // UPDATED!
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}