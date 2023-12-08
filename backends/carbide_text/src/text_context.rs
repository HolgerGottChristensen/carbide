use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache, SwashImage};
use fxhash::FxHashMap;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::scale::image::Content;
use swash::zeno::{Format, Vector};

use carbide_core::color::{RED, YELLOW};
use carbide_core::draw::{Dimension, Position, Rect, Scalar};
use carbide_core::draw::draw_style::DrawStyle;
use carbide_core::draw::image::ImageId;
use carbide_core::environment::Environment;
use carbide_core::image::{DynamicImage, GrayImage, RgbaImage};
use carbide_core::mesh::{MODE_TEXT, MODE_TEXT_COLOR};
use carbide_core::render::InnerRenderContext;
use carbide_core::text::{InnerTextContext, TextId};
use carbide_core::text::TextStyle;

use crate::atlas::texture_atlas::{AtlasId, TextureAtlas};

pub struct TextContext {
    map: FxHashMap<TextId, Buffer>,
    metadata: FxHashMap<TextId, (Position, Scalar)>,
    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextureAtlas,

    scale_context: ScaleContext,
}

impl TextContext {
    pub fn new() -> TextContext {
        TextContext {
            map: Default::default(),
            metadata: Default::default(),
            font_system: FontSystem::new(),
            cache: SwashCache::new(),
            atlas: TextureAtlas::new(512, 512),
            scale_context: ScaleContext::new(),
        }
    }
}

impl InnerTextContext for TextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let mut buffer = self.map.get_mut(&id).unwrap();

        buffer.set_wrap(&mut self.font_system, cosmic_text::Wrap::Word);

        buffer.set_size(&mut self.font_system, requested_size.width as f32, f32::MAX);

        buffer.shape_until_scroll(&mut self.font_system);

        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            height = height.max(run.line_top + buffer.metrics().line_height);

            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), env.scale_factor() as f32);

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

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut Environment) {
        self.metadata.insert(id, (requested_offset.rounded(), env.scale_factor()));
    }

    fn hash(&self, id: TextId) -> Option<u64> {
        todo!()
    }

    fn update(&mut self, id: TextId, text: &str, style: &TextStyle) {
        if let Some(buffer) = self.map.get_mut(&id) {
            let mut buffer = buffer.borrow_with(&mut self.font_system);

            let attributes = Attrs::new();

            buffer.set_text(text, attributes, Shaping::Advanced);
            buffer.set_metrics(Metrics::new(style.font_size as f32, style.font_size as f32 * style.line_height as f32))
        } else {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(style.font_size as f32, style.font_size as f32 * style.line_height as f32));

            {
                let mut buffer = buffer.borrow_with(&mut self.font_system);

                let attributes = Attrs::new();

                buffer.set_text(text, attributes, Shaping::Advanced);
            }


            self.map.insert(id, buffer);
        }
    }

    fn render(&mut self, id: TextId, ctx: &mut dyn InnerRenderContext) {
        let mut buffer = self.map.get_mut(&id).unwrap();
        let metadata = self.metadata.get_mut(&id).unwrap();
        //let mut buffer = buffer.borrow_with(&mut self.font_system);

        buffer.shape_until_scroll(&mut self.font_system);

        // Inspect the output runs
        for run in buffer.layout_runs() {
            ctx.style(DrawStyle::Color(RED));
            ctx.rect(Rect::new(Position::new(0.0, run.line_y as f64), Dimension::new(run.line_w as f64, 1.0 / metadata.1)) + metadata.0);
            ctx.pop_style();

            ctx.style(DrawStyle::Color(YELLOW));
            ctx.rect(Rect::new(Position::new(0.0, run.line_top as f64), Dimension::new(run.line_w as f64, 1.0 / metadata.1)) + metadata.0);
            ctx.pop_style();

            for glyph in run.glyphs.iter() {
                //println!("{:#?}", glyph);
                let physical_glyph = glyph.physical((0., 0.), metadata.1 as f32);

                let book = self.atlas.book(&AtlasId::Glyph(physical_glyph.cache_key));

                if let Some(book) = book {
                    ctx.image(
                        ImageId::default(),
                        Rect::new(
                            Position::new(physical_glyph.x as f64 + book.left as f64, run.line_y as f64 * metadata.1 + physical_glyph.y as f64 - book.top as f64),
                            Dimension::new(book.width as f64, book.height as f64)
                        ) / metadata.1 + metadata.0,
                        book.tex_coords,
                        if book.has_color { MODE_TEXT_COLOR } else { MODE_TEXT }
                    );
                }
            }
        }

        /*let text_color = Color::rgb(0xFF, 0xFF, 0xFF);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);

                let glyph_color = match glyph.color_opt {
                    Some(some) => some,
                    None => text_color,
                };

                self.cache.with_pixels(
                    &mut self.font_system,
                    physical_glyph.cache_key,
                    glyph_color,
                    |x, y, color| {
                        let x = physical_glyph.x + x;
                        let y = run.line_y as i32 + physical_glyph.y + y;
                        let w = 1;
                        let h = 1;
                        let color = color;

                        //ctx.style(DrawStyle::Color(carbide_core::draw::Color::random()));
                        ctx.style(DrawStyle::Color(carbide_core::draw::Color::new_rgba(color.r(), color.g(), color.b(), color.a())));
                        ctx.rect(Rect::new(Position::new(x as f64, y as f64), Dimension::new(w as f64, h as f64)) + *position);
                        ctx.pop_style();
                    },
                );
            }
        }*/
    }

    fn prepare_render(&mut self) {
        self.atlas.process_queued();
    }

    fn update_cache(&mut self, f: &mut dyn FnMut(&DynamicImage)) {
        self.atlas.update_cache(f)
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