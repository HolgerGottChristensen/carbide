use std::collections::HashMap;
use carbide_core::draw::{Dimension, Position};
use carbide_core::text::{InnerTextContext, TextId};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
use carbide_core::environment::Environment;
use carbide_core::image::DynamicImage;
use carbide_core::render::InnerRenderContext;

pub struct WGPUTextContext {
    pub(crate) layouts: HashMap<TextId, Buffer>,
    pub(crate) positions: HashMap<TextId, Position>,
    pub(crate) font_system: FontSystem,
}

impl InnerTextContext for WGPUTextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut Environment) -> Dimension {
        if let Some(layout) = self.layouts.get_mut(&id) {
            layout.set_size(&mut self.font_system, requested_size.width as f32, requested_size.height as f32);
        }

        requested_size
    }

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut Environment) {
        self.positions.insert(id, requested_offset);
    }

    fn hash(&self, id: TextId) -> Option<u64> {
        None
    }

    fn update(&mut self, id: TextId, text: &str) {
        if let Some(layout) = self.layouts.get_mut(&id) {
            layout.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        } else {
            let mut layout = Buffer::new(&mut self.font_system, Metrics::new(32.0, 32.0));
            layout.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);

            self.layouts.insert(id, layout);
        }
    }

    fn render(&self, id: TextId, ctx: &mut dyn InnerRenderContext) {
        todo!()
    }

    fn prepare_render(&mut self) {
        todo!()
    }

    fn cache_queued(&mut self, uploader: &mut dyn FnMut(u32, u32, &DynamicImage)) {
        todo!()
    }
}