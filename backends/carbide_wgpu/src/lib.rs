mod diffuse_bind_group;
mod image;
mod pipeline;
mod render_pass_command;
mod renderer;
mod texture;
mod texture_atlas_command;
mod vertex;
pub mod window;
mod render;
mod filter;
mod bind_group_layouts;
mod render_pipeline_layouts;
mod samplers;
mod bind_groups;
mod textures;
mod proxy_event_loop;
mod gradient;


pub fn init_logger() {
    env_logger::init();
}