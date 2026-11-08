use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use image::DynamicImage;
use crate::text::TextStyle;
use crate::draw::{Dimension, Position, Scalar};
use crate::environment::{Environment, EnvironmentKey, Exclusive};
use crate::text::glyph::Glyph;

/// Lays out, measures and rasterizes text. A backend registers its implementation into the
/// environment through the [TextContextKey], usually from a plugin.
///
/// The `scale_factor` parameters are the scale factor of the window the text is laid out or
/// rendered in. Callers usually read it from the `SceneManager` in the environment.
pub trait TextContext: Debug + 'static {
    /// Calculate the size of a given piece of text based on a requested size.
    /// The returned size will be based on the style, and the text to be rendered,
    /// along with the requested size.
    fn calculate_size_new(
        &mut self,
        text: &str,
        style: &TextStyle,
        requested_size: Option<Dimension>,
        scale_factor: Scalar
    ) -> Dimension;

    /// Render a piece of text at a given position with a requested size and style.
    fn render_new(
        &mut self,
        text: &str,
        style: &TextStyle,
        position: Position,
        requested_size: Option<Dimension>,
        scale_factor: Scalar,
        f: &mut dyn FnMut(&Glyph)
    );

    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, scale_factor: Scalar) -> Dimension;

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, scale_factor: Scalar);

    fn hash(&self, id: TextId) -> Option<u64>;

    fn update(&mut self, id: TextId, text: &str, style: &TextStyle);

    fn render(&mut self, id: TextId, f: &mut dyn FnMut(&Glyph));

    fn prepare_render(&mut self);

    fn update_cache(&mut self, f: &mut dyn FnMut(&DynamicImage));

    fn add_font(&mut self, p: PathBuf);
    fn add_font_from_bytes(&mut self, bytes: Vec<u8>);

    /// Returns (line, index)
    fn hit(&self, id: TextId, position: Position) -> (usize, usize);

    fn position_of(&self, id: TextId, line: usize, index: usize) -> Position;

    fn remove(&mut self, id: TextId);

    /// Free cached data that has not been used recently. The application calls this
    /// periodically. Does nothing by default.
    fn sweep(&mut self) {}
}

/// The key used to look up the [TextContext] from the environment.
#[derive(Copy, Clone, Debug)]
pub struct TextContextKey;

impl EnvironmentKey for TextContextKey {
    // Boxed because environment values must be sized: the environment stores them as
    // `&dyn AnyDebug`, which can't be made from a `&dyn TextContext`. This can become
    // `dyn TextContext` once `ptr_metadata` is stable (https://github.com/rust-lang/rust/issues/81513).
    type Value = Box<dyn TextContext>;
    type Access = Exclusive;
}

impl TextContextKey {
    /// Insert the text context into the environment, making it the text context retrieved by
    /// [TextContextKey::get] and [TextContextKey::get_mut].
    pub fn insert(env: &mut Environment<'static>, text_context: impl TextContext) {
        let value: Box<dyn TextContext> = Box::new(text_context);
        env.insert::<TextContextKey>(Box::leak(Box::new(value)));
    }

    /// Get the text context registered in the environment.
    ///
    /// Panics if no text context is registered. Enable a text backend, like the
    /// `carbide_cosmic_text` plugin, to register one.
    pub fn get<'a>(env: &'a Environment) -> &'a dyn TextContext {
        &**env.get::<TextContextKey>().expect(NO_TEXT_CONTEXT)
    }

    /// Get the text context registered in the environment, mutably.
    ///
    /// Panics if no text context is registered. Enable a text backend, like the
    /// `carbide_cosmic_text` plugin, to register one.
    pub fn get_mut<'a>(env: &'a mut Environment) -> &'a mut dyn TextContext {
        &mut **env.get_mut::<TextContextKey>().expect(NO_TEXT_CONTEXT)
    }
}

const NO_TEXT_CONTEXT: &str = "No text context is registered in the environment. Register one with TextContextKey::insert, for example by enabling the carbide_cosmic_text plugin";

#[derive(Debug)]
pub struct NOOPTextContext;

impl TextContext for NOOPTextContext {
    fn calculate_size_new(&mut self, _text: &str, _style: &TextStyle, _requested_size: Option<Dimension>, _scale_factor: Scalar) -> Dimension {
        unimplemented!()
    }

    fn render_new(&mut self, _text: &str, _style: &TextStyle, _position: Position, _requested_size: Option<Dimension>, _scale_factor: Scalar, _f: &mut dyn FnMut(&Glyph)) {
        unimplemented!()
    }

    fn calculate_size(&mut self, _id: TextId, _requested_size: Dimension, _scale_factor: Scalar) -> Dimension {
        unimplemented!()
    }

    fn calculate_position(&mut self, _id: TextId, _requested_offset: Position, _scale_factor: Scalar) {
        unimplemented!()
    }

    fn hash(&self, _id: TextId) -> Option<u64> {
        unimplemented!()
    }

    fn update(&mut self, _id: TextId, _text: &str, _style: &TextStyle) {
        unimplemented!()
    }

    fn render(&mut self, _id: TextId, _f: &mut dyn FnMut(&Glyph)) {
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

    fn add_font_from_bytes(&mut self, _bytes: Vec<u8>) {
        unimplemented!()
    }

    fn hit(&self, _id: TextId, _position: Position) -> (usize, usize) {
        unimplemented!()
    }

    fn position_of(&self, _id: TextId, _line: usize, _index: usize) -> Position {
        unimplemented!()
    }

    fn remove(&mut self, _id: TextId) {
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