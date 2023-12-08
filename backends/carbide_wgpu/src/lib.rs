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
mod image_context;

use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, LoadOp, Operations, Queue};
pub use application::Application;
use carbide_core::draw::image::ImageId;
pub use wgpu_window::WGPUWindow as Window;
use crate::image::BindGroupExtended;
use crate::wgpu_window::{ADAPTER, BIND_GROUPS, DEVICE_QUEUE, INSTANCE};
pub use crate::image_context::create_bind_group_from_wgpu_texture;

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

pub fn with_bind_groups<F: FnOnce(&HashMap<ImageId, BindGroupExtended>)->R, R>(f: F) -> R {
    BIND_GROUPS.with(|bind_groups| {
        let bind_groups = &*bind_groups.borrow();
        f(bind_groups)
    })
}

pub fn with_bind_groups_mut<F: FnOnce(&mut HashMap<ImageId, BindGroupExtended>)->R, R>(f: F) -> R {
    BIND_GROUPS.with(|bind_groups| {
        let bind_groups = &mut *bind_groups.borrow_mut();
        f(bind_groups)
    })
}

pub fn with_adapter<F: FnOnce(Arc<Adapter>)->R, R>(f: F) -> R {
    ADAPTER.with(|adapter| {
        f(adapter.clone())
    })
}

pub fn with_device_queue<F: FnOnce(Arc<Device>, Arc<Queue>)->R, R>(f: F) -> R {
    DEVICE_QUEUE.with(|(device, queue)| {
        f(device.clone(), queue.clone())
    })
}

pub fn with_instance<F: FnOnce(Arc<Instance>)->R, R>(f: F) -> R {
    INSTANCE.with(|instance| {
        f(instance.clone())
    })
}