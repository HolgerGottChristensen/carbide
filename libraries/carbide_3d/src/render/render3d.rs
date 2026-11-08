use crate::node3d::CommonNode3d;
use crate::render::RenderContext3d;

/// The render trait is used to get the primitives from a widget. It contains two basic functions.
pub trait Render3d: CommonNode3d {
    fn render(&mut self, context: &mut RenderContext3d);
}