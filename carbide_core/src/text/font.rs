use std::collections::HashMap;

use fxhash::{FxBuildHasher, FxHashMap};
use rusttype::GlyphId;

use crate::prelude::text_old;
use crate::Scalar;
use crate::text::FontSize;
use crate::text_old::{FontCollection, pt_to_px};

type RustTypeFont = rusttype::Font<'static>;

pub struct Font {
    // Should in the future be a collection of different font weights
    font: RustTypeFont,

    /// Store a map from the size and string to a width,
    /// In the future we might be able to get rid of fontsize as this is probably just a multiplier
    dimension_cache: FxHashMap<(FontSize, String), (Scalar, Vec<Scalar>)>,
}

impl Font {
    pub fn get_inner(&self) -> &RustTypeFont {
        &self.font
    }

    /// Warning: This currently ignores newlines and tabs, spaces are included
    pub fn calculate_width(&mut self, text: &str, font_size: FontSize) -> Scalar {
        if let Some((cache_hit, _)) = self.dimension_cache.get(&(font_size, text.to_string())) {
            *cache_hit
        } else {
            let mut total_width = 0.0 as Scalar;
            let mut total_char_widths = vec![];

            for word in text.split_ascii_whitespace() {
                if let Some((cache_hit, _)) = self.dimension_cache.get(&(font_size, word.to_string())) {
                    total_width += *cache_hit;
                } else {
                    println!("Cache miss on: {}", word);
                    let (calculated_width, char_widths) = Font::calculate_uncached_width(self, word, font_size);
                    total_char_widths.extend(char_widths.clone());
                    self.dimension_cache.insert((font_size, word.to_string()), (calculated_width, char_widths));

                    total_width += calculated_width;
                }
            }

            text.chars()
                .filter(|c| {
                    c.is_whitespace()
                })
                .for_each(|c| {
                    if let Some((cache_hit, _)) = self.dimension_cache.get(&(font_size, c.to_string())) {
                        total_width += *cache_hit;
                    } else {
                        let string = c.to_string();
                        let (calculated_width, char_widths) = Font::calculate_uncached_width(self, &string, font_size);
                        total_char_widths.extend(char_widths.clone());
                        self.dimension_cache.insert((font_size, string), (calculated_width, char_widths));

                        total_width += calculated_width;
                    }
                });

            self.dimension_cache.insert((font_size, text.to_string()), (total_width, total_char_widths));
            total_width
        }
    }

    fn calculate_uncached_width(&self, text: &str, font_size: FontSize) -> (Scalar, Vec<Scalar>) {
        let scale = rusttype::Scale::uniform(pt_to_px(font_size));

        let mut total_width = 0.0 as Scalar;
        let mut char_widths = vec![];

        text.chars().map(|c| self.font.glyph(c)).fold(None, |state, glyph| {
            let scaled_glyph = glyph.scaled(scale);
            let mut char_width = scaled_glyph.h_metrics().advance_width as Scalar;
            if let Some(last) = state {
                char_width += self.font.pair_kerning(scale, last, scaled_glyph.id()) as Scalar;
            }

            char_widths.push(char_width);
            total_width += char_width;
            Some(scaled_glyph.id())
        });

        (total_width, char_widths)
    }


    pub fn get_char_widths(&mut self, text: &str, font_size: FontSize) -> &Vec<Scalar> {
        // Make sure we have cached the widths of all the letters
        self.calculate_width(text, font_size);

        &self.dimension_cache.get(&(font_size, text.to_string())).unwrap().1
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