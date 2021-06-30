use std::collections::HashMap;

use fxhash::{FxBuildHasher, FxHashMap};
use rusttype::{GlyphId, point, PositionedGlyph, Scale};

use crate::prelude::text_old;
use crate::Scalar;
use crate::text::FontSize;
use crate::text::glyph::Glyph;

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
    pub fn get_inner(&self) -> RustTypeFont {
        // This clone should only be either an ARC clone or reference clone
        self.font.clone()
    }

    fn size_to_scale(font_size: FontSize, scale_factor: Scalar) -> RustTypeScale {
        Font::f32_pt_to_scale(font_size as f32 * scale_factor as f32)
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
            glyphs.push(Glyph::from(next));
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

    /// Converts the given font size in "points" to its font size in pixels.
    /// This is useful for when the font size is not an integer.
    pub fn f32_pt_to_px(font_size_in_points: f32) -> f32 {
        font_size_in_points * 4.0 / 3.0
    }

    /// Converts the given font size in "points" to a uniform `rusttype::Scale`.
    /// This is useful for when the font size is not an integer.
    pub fn f32_pt_to_scale(font_size_in_points: f32) -> Scale {
        Scale::uniform(Font::f32_pt_to_px(font_size_in_points))
    }

    /// Converts the given font size in "points" to its font size in pixels.
    pub fn pt_to_px(font_size_in_points: FontSize) -> f32 {
        Font::f32_pt_to_px(font_size_in_points as f32)
    }

    /// Converts the given font size in "points" to a uniform `rusttype::Scale`.
    pub fn pt_to_scale(font_size_in_points: FontSize) -> Scale {
        Scale::uniform(Font::pt_to_px(font_size_in_points))
    }
}

/// New font creation
impl Font {
    /// Load a single `Font` from a file at the given path.
    pub fn from_file<P>(path: P) -> Result<Self, Error>
        where P: AsRef<std::path::Path>
    {
        use std::io::Read;
        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer)?;
        let inner_font = RustTypeFont::try_from_vec(file_buffer).unwrap();

        Ok(Font {
            font: inner_font,
            height: 0.0,
            dimension_cache: HashMap::with_hasher(FxBuildHasher::default()),
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

#[test]
fn load_bitmap_font() {
    use ttf_parser::Face;
    use image::ImageFormat;

    let emoji_path = "/System/Library/Fonts/Apple Color Emoji.ttc";

    let emoji_data = std::fs::read(emoji_path).unwrap();

    let face = Face::from_slice(&emoji_data, 0).unwrap();
    let glyph_id = face.glyph_index('😀').unwrap();
    println!("Glyph ID: {:?}", glyph_id);
    let raster_image = face.glyph_raster_image(glyph_id, 200).unwrap();
    println!("{:?}", raster_image);

    let image = image::load_from_memory(raster_image.data).unwrap();
    image.into_luma_alpha().save("/Users/holgergottchristensen/Documents/carbide/target/smile.png").unwrap();
}