use std::path::{Path, PathBuf};

use crate::text::font_style::FontStyle;
use crate::text::font_weight::FontWeight;
use crate::text::FontId;

/// Font families should only contain the same sets of glyphs.
#[derive(Clone, Debug)]
pub struct FontFamily {
    pub name: String,
    pub fonts: Vec<FontDescriptor>,
}

impl FontFamily {
    pub fn new(name: &str) -> FontFamily {
        FontFamily {
            name: name.to_string(),
            fonts: vec![],
        }
    }

    pub fn add_font<P: AsRef<Path>>(
        &mut self,
        path: P,
        weight_hint: FontWeight,
        style_hint: FontStyle,
    ) {
        self.fonts.push(FontDescriptor {
            path: path.as_ref().to_path_buf(),
            font_id: 0,
            weight_hint,
            style_hint,
            is_bitmap: false,
        })
    }

    pub fn add_bitmap_font<P: AsRef<Path>>(
        &mut self,
        path: P,
        weight_hint: FontWeight,
        style_hint: FontStyle,
    ) {
        self.fonts.push(FontDescriptor {
            path: path.as_ref().to_path_buf(),
            font_id: 0,
            weight_hint,
            style_hint,
            is_bitmap: true,
        })
    }

    pub fn get_best_fit(&self, weight_hint: FontWeight, style_hint: FontStyle) -> FontId {
        let mut best_fit = 0;
        let mut best_score = 0.0;

        for font in &self.fonts {
            let mut font_score = 0.0;
            match (font.style_hint, style_hint) {
                (FontStyle::Italic, FontStyle::Italic) => {
                    font_score += 1000.0;
                }
                (FontStyle::Normal, FontStyle::Normal) => {
                    font_score += 1000.0;
                }
                (_, _) => (),
            }
            let font_weight = font.weight_hint.weight();
            let target_weight = weight_hint.weight();

            let diff = (font_weight - target_weight).abs();

            font_score += 900.0 - diff;

            if font_score > best_score {
                best_score = font_score;
                best_fit = font.font_id;
            }
        }

        best_fit
    }

    //Todo: create a method to get closest font, specified by weight and bold/italic
}

#[derive(Clone, Debug)]
pub struct FontDescriptor {
    pub path: PathBuf,
    pub font_id: FontId,
    weight_hint: FontWeight,
    style_hint: FontStyle,
    pub(crate) is_bitmap: bool,
}
