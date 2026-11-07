use std::any::TypeId;
use std::pin::Pin;
use log::info;
use wgpu::{Adapter, Device, Features, Instance, Queue};
use carbide_core::environment::{Environment, EnvironmentKey, Shared};
use carbide_core::plugin::Plugin;
use carbide_core::register;

#[derive(Debug)]
pub struct WgpuContextNew {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue
}

impl WgpuContextNew {

    /// Construct a new core context. We should almost always only have one of these.
    /// If you need any of the fields, the best way to get them is by looking the context
    /// up in the environment
    pub async fn new() -> WgpuContextNew {
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            backend_options: Default::default(),
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }).await.unwrap();

        let mut limits = wgpu::Limits::default();
        limits.max_bind_groups = 5;

        info!("{:#?}", adapter.limits());

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("carbide_device"),
                // Required for multiple layer textures, where we clear the same allocated texture using intermediate steps.
                required_features: Features::CLEAR_TEXTURE,
                required_limits: limits,
                memory_hints: Default::default(),
                trace: Default::default(),
            }
        ).await.unwrap();

        info!("{:#?}", device.limits());
        info!("{:#?}", device.features());

        WgpuContextNew {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

impl EnvironmentKey for WgpuContextNew {
    type Value = WgpuContextNew;
    type Access = Shared;
}

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        let context = WgpuContextNew::new().await;

        env.insert::<WgpuContextNew>(Box::leak(Box::new(context)))
    })
}

register!(Plugin {
    dependencies: &[],
    registers: TypeId::of::<WgpuContextNew>(),
    register
});