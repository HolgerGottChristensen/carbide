mod texture;
mod renderer;
mod image;
mod render;
mod render_pass_command;
mod glyph_cache_command;
mod diffuse_bind_group;
mod pipeline;
pub mod window;

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


//let mut state = block_on(Window::new(&window, ui));
/*
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

    "fonts/NotoSans/NotoSans-Regular.ttf"
*/