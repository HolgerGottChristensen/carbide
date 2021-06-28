use std::collections::HashMap;

use fxhash::{FxBuildHasher, FxHashMap};
use rusttype::{GlyphId, point, PositionedGlyph};

use crate::prelude::text_old;
use crate::Scalar;
use crate::text::FontSize;
use crate::text::glyph::Glyph;
use crate::text_old::{f32_pt_to_scale, FontCollection, pt_to_px};

type RustTypeFont = rusttype::Font<'static>;
type RustTypeScale = rusttype::Scale;
type RustTypePoint = rusttype::Point<f32>;

pub struct Font {
    // Should in the future be a collection of different font weights
    font: RustTypeFont,
    height: Scalar,

    /// Store a map from the size and string to a width,
    /// In the future we might be able to get rid of fontsize as this is probably just a multiplier
    dimension_cache: FxHashMap<(FontSize, String), Vec<Glyph>>,
}

impl Font {
    pub fn get_inner(&self) -> &RustTypeFont {
        &self.font
    }

    fn size_to_scale(font_size: FontSize, scale_factor: Scalar) -> RustTypeScale {
        f32_pt_to_scale(font_size as f32 * scale_factor as f32)
    }

    pub fn get_glyphs(&self, text: &str, font_size: FontSize, scale_factor: Scalar) -> (Vec<Scalar>, Vec<Glyph>) {
        let scale = Font::size_to_scale(font_size, scale_factor);
        let mut next_width = 0.0;
        let mut widths = vec![];
        let mut glyphs = vec![];
        let mut last = None;

        for glyph in self.font.glyphs_for(text.chars()) {
            let glyph_scaled = glyph.scaled(scale);
            if let Some(last) = last {
                let kerning = self.font.pair_kerning(scale, last, glyph_scaled.id());
                next_width += kerning as f64;
                widths.push(next_width);
                next_width = 0.0;
            }

            let w = glyph_scaled.h_metrics().advance_width;
            let next = glyph_scaled.positioned(point(0.0, 0.0));
            last = Some(next.id());
            next_width += w as f64;
            glyphs.push(next.standalone());
        };

        // Widths are pushed such that they contain the width and the kerning between itself and the
        // next character. If its the last, we push here.
        widths.push(next_width);

        (widths, glyphs)
    }

    pub fn height(font_size: FontSize, scale_factor: Scalar) -> Scalar {
        Font::size_to_scale(font_size, scale_factor).y as Scalar
    }

    pub fn baseline_offset(&self, font_size: FontSize, scale_factor: Scalar) -> Scalar {
        let scale = Font::size_to_scale(font_size, scale_factor);
        self.font.v_metrics(scale).ascent as Scalar
    }
}

/// New font creation
impl Font {
    /// Load a `super::FontCollection` from a file at a given path.
    pub fn collection_from_file<P>(path: P) -> Result<FontCollection, std::io::Error>
        where P: AsRef<std::path::Path>,
    {
        use std::io::Read;
        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer)?;
        Ok(FontCollection::from_bytes(file_buffer)?)
    }

    /// Load a single `Font` from a file at the given path.
    pub fn from_file<P>(path: P) -> Result<Self, Error>
        where P: AsRef<std::path::Path>
    {
        let collection = Font::collection_from_file(path)?;
        collection.into_font().or(Err(Error::NoFont)).map(|inner_font| {
            Font {
                font: inner_font,
                height: 0.0,
                dimension_cache: HashMap::with_hasher(FxBuildHasher::default()),
            }
        })
    }
}

/// Returned when loading new fonts from file or bytes.
#[derive(Debug)]
pub enum Error {
    /// Some error occurred while loading a `FontCollection` from a file.
    IO(std::io::Error),
    /// No `Font`s could be yielded from the `FontCollection`.
    NoFont,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Error::IO(ref e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let s = match *self {
            Error::IO(ref e) => return std::fmt::Display::fmt(e, f),
            Error::NoFont => "No `Font` found in the loaded `FontCollection`.",
        };
        write!(f, "{}", s)
    }
}