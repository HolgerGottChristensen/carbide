use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use image::DynamicImage;
use carbide::environment::Environment;
use carbide::text::TextStyle;
use crate::draw::{Dimension, Position};
use crate::environment::EnvironmentStack;
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
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut EnvironmentStack) -> Dimension;

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut EnvironmentStack);

    fn hash(&self, id: TextId) -> Option<u64>;

    fn update(&mut self, id: TextId, text: &str, style: &TextStyle);

    fn render(&mut self, id: TextId, ctx: &mut dyn InnerRenderContext);

    fn prepare_render(&mut self);

    fn update_cache(&mut self, f: &mut dyn FnMut(&DynamicImage));

    fn add_font(&mut self, p: PathBuf);

    /// Returns (line, index)
    fn hit(&self, id: TextId, position: Position) -> (usize, usize);

    fn position_of(&self, id: TextId, line: usize, index: usize) -> Position;

    fn remove(&mut self, id: TextId);
}

pub struct NOOPTextContext;

impl InnerTextContext for NOOPTextContext {
    fn calculate_size(&mut self, _id: TextId, _requested_size: Dimension, _env: &mut EnvironmentStack) -> Dimension {
        unimplemented!()
    }

    fn calculate_position(&mut self, _id: TextId, _requested_offset: Position, _env: &mut EnvironmentStack) {
        unimplemented!()
    }

    fn hash(&self, _id: TextId) -> Option<u64> {
        unimplemented!()
    }

    fn update(&mut self, _id: TextId, _text: &str, _style: &TextStyle) {
        unimplemented!()
    }

    fn render(&mut self, _id: TextId, _ctx: &mut dyn InnerRenderContext) {
        unimplemented!()
    }

    fn prepare_render(&mut self) {
        unimplemented!()
    }

    fn update_cache(&mut self, _f: &mut dyn FnMut(&DynamicImage)) {
        unimplemented!()
    }

    fn add_font(&mut self, _p: PathBuf) {
        unimplemented!()
    }

    fn hit(&self, _id: TextId, _position: Position) -> (usize, usize) {
        unimplemented!()
    }

    fn position_of(&self, _id: TextId, _line: usize, _index: usize) -> Position {
        unimplemented!()
    }

    fn remove(&mut self, id: TextId) {
        unimplemented!()
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