use std::path::{Path, PathBuf};

use crate::text::types::font_style::FontStyle;
use crate::text::types::font_weight::FontWeight;
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

    /// This will treat all paths as they are normal non-bitmap fonts.
    pub fn new_from_paths<P: AsRef<Path>>(name: &str, paths: Vec<P>) -> FontFamily {
        let mut family = FontFamily {
            name: name.to_string(),
            fonts: vec![],
        };
        for path in paths {
            family.add_font(path)
        }

        family
    }

    pub fn add_font<P: AsRef<Path>>(&mut self, path: P) {
        self.add_font_with_hints(path, FontWeight::Normal, FontStyle::Normal)
    }

    /// This will add a normal font to the font family. The hints are overridden by the hints
    /// within the font if these are present.
    pub fn add_font_with_hints<P: AsRef<Path>>(
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
        })
    }

    /// Get the best fitting font in the font family based on the weight and style hints.
    /// Since all fonts variations might not exist we try to score them according to some
    /// parameters and return the closest.
    ///
    /// For example if you have a family with W400 and W700, and you request a font with
    /// W900, it will return the W700 font and not the W400.
    ///
    /// The implementation is somewhat arbitrary and might change over time, but with the
    /// requirement that if there is a perfectly matching font in the family this should
    /// be returned.
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
}

#[derive(Clone, Debug)]
pub struct FontDescriptor {
    pub path: PathBuf,
    pub font_id: FontId,
    pub(crate) weight_hint: FontWeight,
    pub(crate) style_hint: FontStyle,
}
