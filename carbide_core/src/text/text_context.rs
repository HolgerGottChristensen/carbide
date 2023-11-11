use std::sync::atomic::{AtomicU32, Ordering};
use image::DynamicImage;
use carbide::environment::Environment;
use crate::draw::{Dimension, Position};
use crate::render::InnerRenderContext;

// pub struct TextContext<'a>(&'a mut dyn InnerTextContext);
//
// impl<'a> TextContext<'a> {
//     pub fn new<C: InnerTextContext + 'static>(context: &'a mut C) -> Self {
//         TextContext(context)
//     }
//
//     pub fn calculate_size(&mut self, id: TextId, requested_size: Dimension) -> Dimension {
//         self.0.calculate_size(id, requested_size, )
//     }
//
//     pub fn calculate_position(&mut self, id: TextId, requested_offset: Position) {
//         self.0.calculate_position(id, requested_offset)
//     }
//
//     pub fn hash(&self, id: TextId) -> Option<u64> {
//         self.0.hash(id)
//     }
//
//     pub fn update(&mut self, id: TextId, text: &str) {
//         self.0.update(id, text)
//     }
// }

pub trait InnerTextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut Environment) -> Dimension;

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut Environment);

    fn hash(&self, id: TextId) -> Option<u64>;

    fn update(&mut self, id: TextId, text: &str);

    fn render(&self, id: TextId, ctx: &mut dyn InnerRenderContext);

    fn prepare_render(&mut self);

    fn cache_queued(&mut self, uploader: &mut dyn FnMut(u32, u32, &DynamicImage));
}

pub struct NOOPTextContext;

impl InnerTextContext for NOOPTextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut Environment) -> Dimension {
        unimplemented!()
    }

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut Environment) {
        unimplemented!()
    }

    fn hash(&self, _id: TextId) -> Option<u64> {
        unimplemented!()
    }

    fn update(&mut self, _id: TextId, _text: &str) {
        unimplemented!()
    }

    fn render(&self, id: TextId, ctx: &mut dyn InnerRenderContext) {
        unimplemented!()
    }

    fn prepare_render(&mut self) {
        unimplemented!()
    }

    fn cache_queued(&mut self, uploader: &mut dyn FnMut(u32, u32, &DynamicImage)) {
        todo!()
    }
}

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TextId(u32);

impl TextId {
    /// Generate a new text ID.
    pub fn new() -> Self {
        static TEXT_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
        TextId(TEXT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for TextId {
    fn default() -> Self {
        TextId::new()
    }
}