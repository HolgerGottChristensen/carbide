use std::collections::HashMap;
use std::rc::Rc;
use crate::draw::{Dimension, ImageId};
use crate::environment::EnvironmentKey;
use crate::render::RenderInstruction;

#[derive(Copy, Clone, Debug)]
pub struct RenderInstructionCache;

impl EnvironmentKey for RenderInstructionCache {
    type Value = HashMap<ImageId, Rc<(Dimension, Vec<RenderInstruction>)>>;
}