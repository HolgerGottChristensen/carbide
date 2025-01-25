use std::path::PathBuf;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, LayoutRun, Metrics, Shaping, Style, SwashImage, Weight};
use fxhash::FxHashMap;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::scale::image::Content;
use swash::zeno::{Format, Vector};

use carbide_core::draw::{Dimension, MODE_TEXT, MODE_TEXT_COLOR, Position, Rect, Scalar};
use carbide_core::environment::{EnvironmentStack};
use carbide_core::image::{DynamicImage, GrayImage, RgbaImage};
use carbide_core::render::InnerRenderContext;
use carbide_core::text::{FontStyle, InnerTextContext, TextId};
use carbide_core::text::TextStyle;
use carbide_core::widget::Wrap;
use unicode_segmentation::UnicodeSegmentation;
use carbide_core::scene::SceneManager;
use crate::atlas::texture_atlas::{AtlasId, TextureAtlas};
use crate::metadata::Metadata;

pub struct TextContext {
    map: FxHashMap<TextId, (Buffer, Metadata)>,
    font_system: FontSystem,
    atlas: TextureAtlas,

    scale_context: ScaleContext,
}

impl TextContext {
    pub fn new() -> TextContext {
        TextContext {
            map: Default::default(),
            font_system: FontSystem::new(),
            atlas: TextureAtlas::new(1024, 1024),
            scale_context: ScaleContext::new(),
        }
    }

    fn partial_glyph_offset(index: usize, layout_run: &LayoutRun) -> (usize, Scalar) {
        for (glyph_index, glyph) in layout_run.glyphs.iter().enumerate() {
            if index == glyph.start {
                return (glyph_index, 0.0);
            } else if index > glyph.start && index < glyph.end {
                // Guess x offset based on characters
                let mut before = 0;
                let mut total = 0;

                let cluster = &layout_run.text[glyph.start..glyph.end];
                for (i, _) in cluster.grapheme_indices(true) {
                    if glyph.start + i < index {
                        before += 1;
                    }
                    total += 1;
                }

                let offset = glyph.w * (before as f32) / (total as f32);
                return (glyph_index, offset as Scalar);
            }
        }

        match layout_run.glyphs.last() {
            Some(glyph) => {
                if index == glyph.end {
                    (layout_run.glyphs.len(), 0.0)
                } else {
                    unreachable!()
                }
            }
            None => {
                (0, 0.0)
            }
        }
    }
}

impl InnerTextContext for TextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut EnvironmentStack) -> Dimension {
        let (ref mut buffer, _) = self.map.get_mut(&id).unwrap_or_else(|| panic!("Expected the text context to contain an entry with id: {:?}", id));

        buffer.set_size(&mut self.font_system, requested_size.width as f32, f32::MAX);

        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        let scale_factor = env.get_mut::<SceneManager>()
            .map(|a| a.scale_factor())
            .unwrap_or(1.0);

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_top + buffer.metrics().line_height);

            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), scale_factor as f32);

                self.atlas.enqueue(AtlasId::Glyph(physical_glyph.cache_key), || {
                    let image = swash_image(&mut self.font_system, &mut self.scale_context, physical_glyph.cache_key);

                    image.map(|image| {
                        let dynamic = match image.content {
                            Content::Mask => {
                                DynamicImage::ImageLuma8(GrayImage::from_raw(image.placement.width, image.placement.height, image.data).unwrap())
                            }
                            Content::SubpixelMask => todo!(),
                            Content::Color => {
                                DynamicImage::ImageRgba8(RgbaImage::from_raw(image.placement.width, image.placement.height, image.data).unwrap())
                            }
                        };

                        (dynamic, image.placement.top, image.placement.left)
                    })

                })
            }
        }

        Dimension::new(width as f64, height as f64)
    }

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut EnvironmentStack) {
        let scale_factor = env.get_mut::<SceneManager>()
            .map(|a| a.scale_factor())
            .unwrap_or(1.0);

        self.map.get_mut(&id).unwrap().1.position = requested_offset.rounded();
        self.map.get_mut(&id).unwrap().1.scale_factor = scale_factor;
    }

    fn hash(&self, _id: TextId) -> Option<u64> {
        todo!()
    }

    fn update(&mut self, id: TextId, text: &str, style: &TextStyle) {
        if let Some((buffer, metadata)) = self.map.get_mut(&id) {

            if &metadata.text != text || &metadata.style != style {
                metadata.style = style.clone();
                metadata.text = text.to_string();

                let mut buffer = buffer.borrow_with(&mut self.font_system);

                let attributes = Attrs::new()
                    .family(Family::Name(&style.family))
                    .style(convert_style(style))
                    .weight(convert_weight(style));

                buffer.set_text(text, attributes, Shaping::Advanced);
                buffer.set_wrap(convert_wrap(style));
                buffer.set_metrics(Metrics::new(style.font_size as f32, style.font_size as f32 * style.line_height as f32));
            }
        } else {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(style.font_size as f32, style.font_size as f32 * style.line_height as f32));

            {
                let mut buffer = buffer.borrow_with(&mut self.font_system);

                let attributes = Attrs::new()
                    .family(Family::Name(&style.family))
                    .style(convert_style(style))
                    .weight(convert_weight(style));

                buffer.set_text(text, attributes, Shaping::Advanced);
                buffer.set_wrap(convert_wrap(style));
            }


            self.map.insert(id, (buffer, Metadata {
                scale_factor: 1.0,
                position: Default::default(),
                text: text.to_string(),
                style: style.clone(),
            }));
        }
    }

    fn render(&mut self, id: TextId, ctx: &mut dyn InnerRenderContext) {
        let (ref mut buffer, metadata) = self.map.get_mut(&id).unwrap_or_else(|| panic!("Expected the text context to contain an entry with id: {:?}", id));

        // Inspect the output runs
        for run in buffer.layout_runs() {
            /*ctx.style(DrawStyle::Color(RED));
            ctx.rect(Rect::new(Position::new(0.0, run.line_y as f64), Dimension::new(run.line_w as f64, 1.0 / metadata.scale_factor)) + metadata.position);
            ctx.pop_style();

            ctx.style(DrawStyle::Color(YELLOW));
            ctx.rect(Rect::new(Position::new(0.0, run.line_top as f64), Dimension::new(run.line_w as f64, 1.0 / metadata.scale_factor)) + metadata.position);
            ctx.pop_style();

            ctx.style(DrawStyle::Color(LIGHT_GREEN));
            ctx.rect(Rect::new(Position::new(0.0, run.line_top as f64 + buffer.metrics().line_height as f64), Dimension::new(run.line_w as f64, 1.0 / metadata.scale_factor)) + metadata.position);
            ctx.pop_style();*/

            for glyph in run.glyphs.iter() {
                //println!("{:#?}", glyph);
                let physical_glyph = glyph.physical((0., 0.), metadata.scale_factor as f32);

                let book = self.atlas.book(&AtlasId::Glyph(physical_glyph.cache_key));

                if let Some(book) = book {
                    let bb = Rect::new(
                        Position::new(physical_glyph.x as f64 + book.left as f64, run.line_y as f64 * metadata.scale_factor + physical_glyph.y as f64 - book.top as f64),
                        Dimension::new(book.width as f64, book.height as f64)
                    ) / metadata.scale_factor + metadata.position;

                    /*ctx.style(DrawStyle::Color(LIGHT_RED));
                    ctx.rect(bb);
                    ctx.pop_style();*/

                    ctx.image(
                        None,
                        bb,
                        book.tex_coords,
                        if book.has_color { MODE_TEXT_COLOR } else { MODE_TEXT }
                    );

                }
            }
        }
    }

    fn prepare_render(&mut self) {
        self.atlas.process_queued();
    }

    fn update_cache(&mut self, f: &mut dyn FnMut(&DynamicImage)) {
        self.atlas.update_cache(f)
    }

    fn add_font(&mut self, p: PathBuf) {
        self.font_system.db_mut().load_font_file(p).unwrap();
    }

    fn hit(&self, id: TextId, position: Position) -> (usize, usize) {
        let (ref buffer, meta) = self.map.get(&id).unwrap();

        let hit = buffer.hit(position.x as f32, position.y as f32);

        fn grapheme_index_from_byte_offset(index: usize, string: &str) -> usize {
            for (i, (g, _)) in string.grapheme_indices(true).enumerate() {
                if g >= index {
                    return i
                }
            }

            string.grapheme_indices(true).count()
        }

        let grapheme_index = grapheme_index_from_byte_offset(hit.unwrap().index, &meta.text);

        (hit.unwrap().line, grapheme_index)
    }

    fn position_of(&self, id: TextId, line: usize, index: usize) -> Position {
        let (ref buffer, meta) = self.map.get(&id).unwrap();

        let byte_offset = meta.text
            .grapheme_indices(true)
            .skip(index)
            .map(|(i, _)| i)
            .next()
            .unwrap_or(meta.text.len());

        for layout_run in buffer.layout_runs() {
            if layout_run.line_i == line {
                let (glyph_index, internal_offset) = Self::partial_glyph_offset(byte_offset, &layout_run);

                let x = match layout_run.glyphs.get(glyph_index) {
                    Some(glyph) => {
                        // Start of detected glyph
                        if glyph.level.is_rtl() {
                            glyph.x + glyph.w - internal_offset as f32
                        } else {
                            glyph.x + internal_offset as f32
                        }
                    }
                    None => match layout_run.glyphs.last() {
                        Some(glyph) => {
                            // End of last glyph
                            if glyph.level.is_rtl() {
                                glyph.x
                            } else {
                                glyph.x + glyph.w
                            }
                        }
                        None => {
                            // Start of empty line
                            0.0
                        }
                    }
                };

                return Position::new(x as Scalar, 0.0);
            }
        }

        unreachable!()
    }

    fn remove(&mut self, id: TextId) {
        self.map.remove(&id);
    }
}

fn swash_image(
    font_system: &mut FontSystem,
    context: &mut ScaleContext,
    cache_key: cosmic_text::CacheKey,
) -> Option<SwashImage> {
    let font = match font_system.get_font(cache_key.font_id) {
        Some(some) => some,
        None => {
            return None;
        }
    };

    // Build the scaler
    let mut scaler = context
        .builder(font.as_swash())
        .size(f32::from_bits(cache_key.font_size_bits))
        .hint(true)
        .build();

    // Compute the fractional offset-- you'll likely want to quantize this
    // in a real renderer
    let offset = Vector::new(cache_key.x_bin.as_float(), cache_key.y_bin.as_float());

    // Select our source order
    Render::new(&[
        // Color outline with the first palette
        Source::ColorOutline(0),
        // Color bitmap with best fit selection mode
        Source::ColorBitmap(StrikeWith::BestFit),
        // Standard scalable outline
        Source::Outline,
    ])
        // Select a subpixel format
        .format(Format::Alpha)
        // Apply the fractional offset
        .offset(offset)
        // Render the image
        .render(&mut scaler, cache_key.glyph_id)
}

fn convert_style(style: &TextStyle) -> Style {
    match style.font_style {
        FontStyle::Normal => Style::Normal,
        FontStyle::Italic => Style::Italic,
    }
}

fn convert_weight(weight: &TextStyle) -> Weight {
    Weight(weight.font_weight.weight())
}

fn convert_wrap(wrap: &TextStyle) -> cosmic_text::Wrap {
    match wrap.wrap {
        Wrap::Character => cosmic_text::Wrap::Glyph,
        Wrap::Whitespace => cosmic_text::Wrap::Word,
        Wrap::None => cosmic_text::Wrap::None,
    }
}