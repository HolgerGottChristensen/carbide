use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, LoadOp, Operations, Queue};

pub use application::Application;
use carbide_core::draw::ImageId;
pub use window::Window;
pub use render_target::RenderTarget;

pub use crate::image_context::create_bind_group_from_wgpu_texture;
use crate::application::{ADAPTER, INSTANCE};
pub use crate::application::{DEVICE, QUEUE};
use crate::globals::BIND_GROUPS;
use crate::image_context::BindGroupExtended;

mod bind_group_layouts;
mod bind_groups;
mod filter;
mod gradient;
mod pipeline;
mod proxy_event_loop;
//mod render;
mod render_pass_command;
mod samplers;
mod texture_atlas_command;
mod textures;
mod vertex;
//pub mod window;
mod application;
mod render_context;
mod image_context;
mod render_target;
mod window;
mod globals;
mod msaa;

pub fn init_logger() {
    }

enum RenderPassOps {
    Start,
    Middle,
}

fn render_pass_ops(ops_type: RenderPassOps) -> (Operations<wgpu::Color>, Operations<u32>, Operations<f32>) {
    let color_op = match ops_type {
        RenderPassOps::Start => wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 1.0,
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

pub fn with_bind_groups<F: FnOnce(&HashMap<ImageId, BindGroupExtended>)->R, R>(f: F) -> R {
    BIND_GROUPS.with(|bind_groups| {
        let bind_groups = &*bind_groups.borrow();
        f(bind_groups)
    })
}

pub fn with_adapter<F: FnOnce(Arc<Adapter>)->R, R>(f: F) -> R {
    f(ADAPTER.clone())
}

pub fn with_device<F: FnOnce(Arc<Device>)->R, R>(f: F) -> R {
    f(DEVICE.clone())
}

pub fn with_queue<F: FnOnce(Arc<Queue>)->R, R>(f: F) -> R {
    f(QUEUE.clone())
}

pub fn with_instance<F: FnOnce(Arc<Instance>)->R, R>(f: F) -> R {
    f(INSTANCE.clone())
}