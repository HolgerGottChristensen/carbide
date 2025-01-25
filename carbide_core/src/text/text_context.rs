use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use image::DynamicImage;
use carbide::text::TextStyle;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::render::InnerRenderContext;

pub trait InnerTextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut Environment) -> Dimension;

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut Environment);

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
    fn calculate_size(&mut self, _id: TextId, _requested_size: Dimension, _env: &mut Environment) -> Dimension {
        unimplemented!()
    }

    fn calculate_position(&mut self, _id: TextId, _requested_offset: Position, _env: &mut Environment) {
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