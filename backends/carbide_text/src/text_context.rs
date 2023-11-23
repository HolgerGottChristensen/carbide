use std::collections::HashMap;
use std::path::Path;
use fxhash::{FxBuildHasher, FxHashMap};
use carbide_core::draw::{Dimension, Position, Rect, Scalar};
use carbide_core::draw::image::ImageId;
use carbide_core::environment::Environment;
use carbide_core::image::DynamicImage;
use carbide_core::locate_folder;
use carbide_core::mesh::{MODE_TEXT, MODE_TEXT_COLOR};
use carbide_core::render::InnerRenderContext;
use carbide_core::text::{FontId, FontSize, FontStyle, FontWeight, InnerTextContext, TextDecoration, TextId};
use carbide_core::widget::Wrap;
use crate::atlas::texture_atlas::TextureAtlas;
use crate::font::Font;
use crate::font_family::FontFamily;
use crate::glyph::Glyph;
use crate::internal_text::Text;
use carbide_core::text::TextStyle;

pub struct TextContext {
    atlas: TextureAtlas,
    map: FxHashMap<TextId, Text>,

    /// Keep the loaded fonts in a map from font id to font. This is used when
    /// calculating the size of rendered strings.
    fonts: Vec<Font>,

    /// Font families. The fonts are still kept as seperate fonts in the above vec, but references
    /// are kept in families for better lookup.
    font_families: FxHashMap<String, FontFamily>,

    /// System font family name. This is used to get the font family for default text rendering.
    /// This should always be expected to exist.
    default_font_family_name: String,
}

impl TextContext {
    pub fn new() -> TextContext {
        let default_font_family_name = "NotoSans";

        let mut res = TextContext {
            atlas: TextureAtlas::new(512, 512),
            map: Default::default(),
            fonts: vec![],
            font_families: HashMap::with_hasher(FxBuildHasher::default()),
            default_font_family_name: default_font_family_name.to_string(),
        };

        #[cfg(target_os = "macos")]
        {
            let mut family = FontFamily::new("Apple Color Emoji");
            family.add_font_with_hints(
                "/System/Library/Fonts/Apple Color Emoji.ttc",
                FontWeight::Normal,
                FontStyle::Normal,
            );
            res.add_font_family(family);
        }

        res
    }

    pub fn insert_font_from_file(&mut self, path: impl AsRef<Path>) -> (FontId, FontWeight, FontStyle) {
        let mut font = Font::from_file(path);
        let weight = font.weight();
        let style = font.style();
        let font_id = self.fonts.len();
        font.set_font_id(font_id);
        self.fonts.push(font);
        (font_id, weight, style)
    }

    pub fn add_glyphs_to_atlas(&mut self, glyphs: Vec<&mut Glyph>, scale_factor: f64) {
        for glyph in glyphs {
            let font = &self.fonts[glyph.font_id()];
            if let Some(entry) = self
                .atlas
                .queue_glyph(glyph, font, scale_factor)
            {
                glyph.set_atlas_entry(entry);
            }
        }
    }

    pub fn remove_glyphs_from_atlas(&mut self, _glyphs: &Vec<Glyph>) {}

    pub fn get_glyph_from_fallback(
        &mut self,
        c: char,
        font_size: FontSize,
        scale_factor: Scalar,
    ) -> (Scalar, Glyph) {
        // Try all the loaded fonts. We only check the first font in each family.
        // Todo: Consider using weight hints and style hints.
        // Todo: Consider going through a separate list if we have a lot of families loaded.
        for (_, font_family) in &self.font_families {
            println!("Looking up in font family: {:?}", font_family);
            let font_id = font_family.get_best_fit(FontWeight::Normal, FontStyle::Normal);
            if let Some(res) = self.get_font(font_id).glyph_for(c, font_size, scale_factor) {
                return res;
            }
        }

        // Try load fallback fonts until we run out
        // Todo: Implement fallback font list.

        // Get the glyph for the unknown character: �
        if c != '�' {
            self.get_glyph_from_fallback('�', font_size, scale_factor)
        } else {
            panic!("Could not lookup the char in any of the loaded fonts or in any fallback fonts. \
            Further more we could not look up the missing char replacement �. Something is not right.")
        }
    }

    pub fn get_font_ref(&self, id: FontId) -> &Font {
        &self.fonts[id]
    }

    pub fn get_font(&self, id: FontId) -> Font {
        self.fonts[id].clone()
    }

    pub fn get_font_mut(&mut self, id: FontId) -> &mut Font {
        &mut self.fonts[id]
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert_font(&mut self, font: Font) -> FontId {
        let font_id = self.fonts.len();
        self.fonts.push(font);
        font_id
    }

    // TODO: Add fonts automatically and warn if none could be loaded: https://github.com/RazrFalcon/fontdb/blob/master/src/lib.rs
    pub fn add_font_family(&mut self, mut family: FontFamily) {
        for font in &mut family.fonts {
            let assets = locate_folder::Search::KidsThenParents(3, 5)
                .for_folder("assets")
                .unwrap();
            let font_path = assets.join(&font.path);
            let (font_id, weight, style) = self.insert_font_from_file(font_path);
            font.font_id = font_id;
            font.weight_hint = weight;
            font.style_hint = style;
        }
        let key = family.name.clone();
        self.font_families.insert(key, family);
    }

    pub fn get_first_font_family(&self) -> &FontFamily {
        for (_, family) in self.font_families.iter() {
            return family;
        }

        panic!("No font family have been added, so we can not get the first.")
    }

    pub fn get_system_font_family(&self) -> &FontFamily {
        self.get_font_family(&self.default_font_family_name)
    }

    pub fn get_font_family(&self, name: &str) -> &FontFamily {
        if name == "system-font" {
            self.get_system_font_family()
        } else {
            self.font_families
                .get(name)
                .expect("Could not find a suitable font family")
        }
    }

    pub fn font_from_style(&self, style: &TextStyle) -> Font {
        let family = self.get_font_family(&style.font_family);
        let font_id = family.get_best_fit(style.font_weight, style.font_style);
        self.get_font(font_id)
    }

    pub fn font_id_from_style(&self, style: &TextStyle) -> FontId {
        let family = self.get_font_family(&style.font_family);
        family.get_best_fit(style.font_weight, style.font_style)
    }
}

impl InnerTextContext for TextContext {
    fn calculate_size(&mut self, id: TextId, requested_size: Dimension, env: &mut Environment) -> Dimension {
        if let Some(text) = self.map.get_mut(&id) {
            text.calculate_size(requested_size, env)
        } else {
            panic!("Update before calculate size");
        }
    }

    fn calculate_position(&mut self, id: TextId, requested_offset: Position, env: &mut Environment) {
        if let Some(text) = self.map.get_mut(&id) {
            text.position(requested_offset);
            text.ensure_glyphs_added_to_atlas(&self.fonts, &mut self.atlas, env.scale_factor());
        } else {
            panic!("Update before calculate size");
        }
    }

    fn hash(&self, id: TextId) -> Option<u64> {
        todo!()
    }

    fn update(&mut self, id: TextId, text: &str, style: &TextStyle) {

        let add = if let Some(internal) = self.map.get_mut(&id) {
            internal.string_that_generated_this() != text || internal.style_that_generated_this() != style
        } else {
            true
        };

        if add {
            let new = Text::new(text.to_string(), style.clone(), self, 2.0);
            self.map.insert(id, new);
        }
    }

    fn render(&self, id: TextId, ctx: &mut dyn InnerRenderContext) {
        if let Some(text) = self.map.get(&id) {
            for glyph in text.first_glyphs() {
                if let Some(bb) = glyph.bb() {
                    if let Some(index) = glyph.atlas_entry() {
                        if !index.borrow().is_active {
                            println!(
                                "Trying to show glyph that is not in the texture atlas 11111."
                            );
                        }
                        let coords = index.borrow().tex_coords;

                        let mode = if glyph.is_bitmap() { MODE_TEXT_COLOR } else { MODE_TEXT };

                        ctx.image(
                            ImageId::default(),
                            bb / 2.0,
                            Rect::new(Position::new(coords.min.x as Scalar, coords.min.y as Scalar), Dimension::new(coords.width() as Scalar, coords.height() as Scalar)),
                            mode
                        );
                    } else {
                        println!("Trying to show glyph that is not in the texture atlas.");
                    }
                }
            }
        } else {
            panic!("Update before calculate size");
        }
    }

    fn prepare_render(&mut self) {
        self.atlas.prepare_queued();
    }

    fn cache_queued(&mut self, uploader: &mut dyn FnMut(u32, u32, &DynamicImage)) {
        self.atlas.cache_queued(uploader);
    }
}