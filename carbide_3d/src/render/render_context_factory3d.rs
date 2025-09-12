use carbide::environment::{Environment, EnvironmentKey};
use crate::render::InnerRenderContext3d;

#[derive(Debug)]
pub struct ContextFactory3d {
    pub render_context: fn(&mut Environment) -> Box<dyn InnerRenderContext3d>,
}

impl EnvironmentKey for ContextFactory3d {
    type Value = ContextFactory3d;
}