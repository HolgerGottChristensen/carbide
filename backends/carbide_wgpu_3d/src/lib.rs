use std::collections::{HashSet, LinkedList};
use std::path::{Path, PathBuf};
use include_dir::{Dir, include_dir};
use once_cell::sync::Lazy;
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};
use carbide_3d::register_render_context3d_initializer;
use carbide_wgpu::DEVICE;
use crate::render_context_3d::render_context_3d_initializer;

mod render_context_3d;
mod pbr_material;
mod material;
mod render_pass_command;
mod vertex;
mod storage_buffer;
mod object;
mod camera;
mod directional_light;
mod point_light;
mod uniforms;

pub fn init() {
    register_render_context3d_initializer("carbide_wgpu_3d", render_context_3d_initializer);
}

static SHADERS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/shaders");
static INCLUDE_PREFIX: &'static str = "//!include";

pub(crate) static SHADER: Lazy<ShaderModule> = Lazy::new(|| {
    DEVICE.create_shader_module(preprocess_shader("pbr/pbr.wgsl"))
});

fn preprocess_shader<S: AsRef<Path>>(path: S) -> ShaderModuleDescriptor<'static> {
    let mut files_to_process = Vec::new();
    let mut files_processed = HashSet::new();
    files_to_process.push(path.as_ref().to_path_buf());

    let mut combined = String::new();

    while let Some(path) = files_to_process.pop() {
        if files_processed.contains(&path) { continue; }
        files_processed.insert(path.clone());
        let file = SHADERS_DIR.get_file(path.clone()).expect(&format!("the included shader directory to contain a filw with the path: {:?}", path));
        let content = file.contents_utf8().expect("the file to be readable in utf8");
        let mut lines = content.lines();

        for line in lines {
            if line.starts_with(INCLUDE_PREFIX) {
                let new_path = PathBuf::from(line.replace(INCLUDE_PREFIX, "").trim());
                files_to_process.push(new_path);
            } else {
                combined.push_str(line);
                combined.push('\n');
            }
        }
    }

    ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(combined.into()),
    }
}





