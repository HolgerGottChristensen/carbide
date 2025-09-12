use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, LoadOp, Operations, Queue, StoreOp};

pub use application::Application;
use carbide_core::draw::ImageId;
pub use wgpu_render_target::WgpuRenderTarget;
pub use window::Window;

pub use crate::image_context::create_bind_group_from_wgpu_texture;
use crate::image_context::BindGroupExtended;
pub use wgpu_context::*;

mod bind_group_layouts;
mod bind_groups;
mod wgpu_filter;
mod wgpu_gradient;
mod pipeline;
mod proxy_event_loop;
//mod render;
mod render_pass_command;
mod samplers;
mod texture_atlas_command;
mod textures;
mod wgpu_vertex;
//pub mod window;
mod application;
mod render_context;
mod image_context;
mod wgpu_render_target;
mod window;
mod wgpu_msaa;
mod wgpu_context;
mod wgpu_dashes;

pub fn init_logger() {
    }

enum RenderPassOps {
    Start,
    Middle,
}

fn render_pass_ops(ops_type: RenderPassOps) -> (Operations<wgpu::Color>, Operations<u32>, Operations<f32>) {
    let color_op = match ops_type {
        RenderPassOps::Start => Operations {
            load: LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            store: StoreOp::Store,
        },
        RenderPassOps::Middle => Operations {
            load: LoadOp::Load,
            store: StoreOp::Store,
        },
    };

    let stencil_op = match ops_type {
        RenderPassOps::Start => Operations {
            load: LoadOp::Clear(0),
            store: StoreOp::Store,
        },
        RenderPassOps::Middle => Operations {
            load: LoadOp::Load,
            store: StoreOp::Store,
        },
    };

    let depth_op = match ops_type {
        RenderPassOps::Start => Operations {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Store,
        },
        RenderPassOps::Middle => Operations {
            load: LoadOp::Load,
            store: StoreOp::Store,
        },
    };

    (color_op, stencil_op, depth_op)
}

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;
/// Draw colored icons from main images and not the glyph atlas.
pub const MODE_ICON: u32 = 3;
/// Draw colored bitmap glyphs.
pub const MODE_TEXT_COLOR: u32 = 4;

pub const MODE_GRADIENT_GEOMETRY: u32 = 5;

pub const MODE_GRADIENT_ICON: u32 = 6;

pub const MODE_GRADIENT_TEXT: u32 = 7;

pub const MODE_GEOMETRY_DASH_FAST: u32 = 8;
pub const MODE_GRADIENT_GEOMETRY_DASH_FAST: u32 = 9;

pub const MODE_GEOMETRY_DASH: u32 = 10;
pub const MODE_GRADIENT_GEOMETRY_DASH: u32 = 11;