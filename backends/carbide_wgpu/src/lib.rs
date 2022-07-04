mod bind_group_layouts;
mod bind_groups;
mod diffuse_bind_group;
mod filter;
mod gradient;
mod image;
mod pipeline;
mod proxy_event_loop;
mod render;
mod render_pass_command;
mod render_pipeline_layouts;
mod renderer;
mod samplers;
mod texture;
mod texture_atlas_command;
mod textures;
mod vertex;
pub mod window;

pub fn init_logger() {
    env_logger::init();
}
