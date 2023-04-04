mod bind_group_layouts;
mod bind_groups;
mod diffuse_bind_group;
mod filter;
mod gradient;
mod image;
mod pipeline;
mod proxy_event_loop;
//mod render;
mod render_pass_command;
mod render_pipeline_layouts;
mod renderer;
mod samplers;
mod texture;
mod texture_atlas_command;
mod textures;
mod vertex;
//pub mod window;
mod application;
mod wgpu_window;
mod render_context;

use wgpu::{LoadOp, Operations};
pub use application::Application;
pub use wgpu_window::WGPUWindow as Window;

pub fn init_logger() {
    env_logger::init();
}

enum RenderPassOps {
    Start,
    Middle,
}

fn render_pass_ops(ops_type: RenderPassOps) -> (Operations<wgpu::Color>, Operations<u32>, Operations<f32>) {
    let color_op = match ops_type {
        RenderPassOps::Start => wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            store: true,
        },
        RenderPassOps::Middle => wgpu::Operations {
            load: LoadOp::Load,
            store: true,
        },
    };

    let stencil_op = match ops_type {
        RenderPassOps::Start => wgpu::Operations {
            load: wgpu::LoadOp::Clear(0),
            store: true,
        },
        RenderPassOps::Middle => wgpu::Operations {
            load: LoadOp::Load,
            store: true,
        },
    };

    let depth_op = match ops_type {
        RenderPassOps::Start => wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: true,
        },
        RenderPassOps::Middle => wgpu::Operations {
            load: LoadOp::Load,
            store: true,
        },
    };

    (color_op, stencil_op, depth_op)
}

