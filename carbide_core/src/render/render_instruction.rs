use cgmath::Matrix4;
use crate::draw::DrawStyle;
use crate::draw::{DrawOptions, DrawShape};
use crate::render::Style;
use crate::state::AnyReadState;

#[derive(Clone, Debug)]
pub enum RenderInstruction {
    Shape {
        shape: DrawShape,
        options: DrawOptions
    },
    PushStyle {
        style: Box<dyn AnyReadState<T=Style>>,
    },
    PopStyle,
    PushTransform {
        transform: Matrix4<f32>,
    },
    PopTransform,
}