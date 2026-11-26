//! A wgpu renderer for `carbide_3d` scenes.
//!
//! Linking this crate registers a plugin that puts a [`ContextFactory3d`] into the environment, so
//! every `Scene3d` renders through a [`WgpuRenderContext3d`] into its layer.

use std::any::TypeId;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use include_dir::{include_dir, Dir};
use wgpu::{ShaderModuleDescriptor, ShaderSource};

use carbide_3d::render::{ContextFactory3d, RenderContext3dId};
use carbide_core::environment::Environment;
use carbide_core::plugin::Plugin;
use carbide_core::register;
use carbide_wgpu_core::WgpuContextNew;

pub use image_context_3d::ImageContext3d;
pub use render_context_3d::WgpuRenderContext3d;
pub use mesh_cache::{CachedMesh, MeshCache};

mod camera;
mod directional_light;
mod image_context_3d;
mod material;
mod mesh_cache;
mod object;
mod outline;
mod pbr_material;
mod point_light;
mod render_context_3d;
mod render_pass_command;
mod shadow;
mod storage_buffer;
mod uniforms;
mod vertex;

static SHADERS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/shaders");
static INCLUDE_PREFIX: &str = "//!include";

/// Load a shader from the `shaders` folder, replacing each `//!include <path>` line with the
/// contents of that file. Each file is included at most once.
fn preprocess_shader<S: AsRef<Path>>(path: S) -> ShaderModuleDescriptor<'static> {
    let mut files_to_process = vec![path.as_ref().to_path_buf()];
    let mut files_processed = HashSet::new();

    let mut combined = String::new();

    while let Some(path) = files_to_process.pop() {
        if !files_processed.insert(path.clone()) {
            continue;
        }

        let file = SHADERS_DIR.get_file(&path)
            .unwrap_or_else(|| panic!("the shader directory to contain a file with the path: {:?}", path));
        let content = file.contents_utf8().expect("the shader file to be utf8");

        for line in content.lines() {
            if line.starts_with(INCLUDE_PREFIX) {
                files_to_process.push(PathBuf::from(line.replace(INCLUDE_PREFIX, "").trim()));
            } else {
                combined.push_str(line);
                combined.push('\n');
            }
        }
    }

    ShaderModuleDescriptor {
        label: Some("carbide_3d_shader"),
        source: ShaderSource::Wgsl(combined.into()),
    }
}

fn register<'a>(env: &'a mut Environment<'static>) -> Pin<Box<dyn Future<Output=()> + 'a>> {
    Box::pin(async {
        // The meshes' GPU copies, shared by every view the renderer draws.
        env.insert::<MeshCache>(Box::leak(Box::new(MeshCache::new())));

        env.insert::<ContextFactory3d>(Box::leak(Box::new(ContextFactory3d {
            id: RenderContext3dId::of::<WgpuRenderContext3d>(),
            render_context: |_env| Box::new(WgpuRenderContext3d::new()),
        })))
    })
}

register!(Plugin {
    // The render context draws with the device and queue of `WgpuContextNew`.
    dependencies: &[TypeId::of::<WgpuContextNew>()],
    registers: TypeId::of::<ContextFactory3d>(),
    register
});
