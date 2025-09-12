use std::collections::HashSet;
use std::path::{Path, PathBuf};
use include_dir::{Dir, include_dir};
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};
pub use render_context_3d::WGPURenderContext3d;
pub use image_context_3d::ImageContext3d;

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
mod image_context_3d;

static SHADERS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/shaders");
static INCLUDE_PREFIX: &'static str = "//!include";

fn preprocess_shader<S: AsRef<Path>>(path: S) -> ShaderModuleDescriptor<'static> {
    let mut files_to_process = Vec::new();
    let mut files_processed = HashSet::new();
    files_to_process.push(path.as_ref().to_path_buf());

    let mut combined = String::new();

    while let Some(path) = files_to_process.pop() {
        if files_processed.contains(&path) { continue; }
        files_processed.insert(path.clone());
        let file = SHADERS_DIR.get_file(path.clone()).expect(&format!("the included shader directory to contain a file with the path: {:?}", path));
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





