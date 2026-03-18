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
        style: RenderInstructionValue<Style>,
    },
    PopStyle,
    PushTransform {
        transform: Matrix4<f32>,
    },
    PopTransform,
}

#[derive(Clone, Debug)]
pub enum RenderInstructionValue<T> where Box<dyn AnyReadState<T=T>>: Clone {
    Constant(T),
    Variable(Box<dyn AnyReadState<T=T>>)
}