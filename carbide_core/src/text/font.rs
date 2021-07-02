#![allow(unsafe_code)]

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Range;

use fxhash::{FxBuildHasher, FxHashMap};
use image::DynamicImage;
use rusttype::{GlyphId, IntoGlyphId, point, PositionedGlyph, Scale, VMetrics};

use crate::prelude::text_old;
use crate::Scalar;
use crate::text::FontSize;
use crate::text::glyph::Glyph;

type RustTypeFont = rusttype::Font<'static>;
type RustTypeScale = rusttype::Scale;
type RustTypePoint = rusttype::Point<f32>;

const POINT_TO_PIXEL: f32 = 4.0 / 3.0;

pub struct Font {
    // Should in the future be a collection of different font weights
    font: RustTypeFont,
    height: Scalar,
    bitmap_font: bool,

    /// Store a map from the size and string to a width,
    /// In the future we might be able to get rid of fontsize as this is probably just a multiplier
    dimension_cache: FxHashMap<(FontSize, String), Vec<Glyph>>,
}

impl Debug for Font {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("is_bitmap", &self.bitmap_font)
            .finish()
    }
}

impl Font {
    pub fn get_glyph_raster_image(&self, character: char, font_size: FontSize) -> Option<DynamicImage> {
        let face = self.font.inner();
        if let Some(id) = face.glyph_index(character) {
            let raster_image = face.glyph_raster_image(id, font_size as u16);
            raster_image.map(|raster| {
                image::load_from_memory(raster.data).unwrap()
            })
        } else {
            None
        }
    }

    pub fn get_glyph_id(&self, c: char) -> Option<GlyphId> {
        self.font.inner().glyph_index(c).map(|ttf_parser::GlyphId(id)| GlyphId(id))
    }

    pub fn is_bitmap(&self) -> bool {
        self.bitmap_font
    }

    pub fn get_glyph_raster_image_from_id(&self, id: GlyphId, font_size: FontSize) -> Option<DynamicImage> {
        let face = self.font.inner();
        let raster_image = face.glyph_raster_image(ttf_parser::GlyphId(id.0), font_size as u16);
        raster_image.map(|raster| {
            image::load_from_memory(raster.data).unwrap()
        })
    }

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
            println!("VMetrics: {:?}", self.font.v_metrics(scale));
            println!("Scale used: {:?}", scale);
            let next = glyph_scaled.positioned(point(0.0, 0.0));
            println!("Glyph: {:?}", next);
            last = Some(next.id());
            next_width += w as f64;
            glyphs.push(Glyph::from(next));
        };

        println!("Next width: {:?}", next_width);
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
        font_size_in_points * POINT_TO_PIXEL
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
            bitmap_font: false,
            dimension_cache: HashMap::with_hasher(FxBuildHasher::default()),
        })
    }

    /// Load a single `Font` from a file at the given path.
    pub fn from_file_bitmap<P>(path: P) -> Result<Self, Error>
        where P: AsRef<std::path::Path>
    {
        use std::io::Read;
        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer)?;
        let mut inner_font = RustTypeFont::try_from_vec(file_buffer).unwrap();
        inner_font.with_custom_v_metrics(VMetrics {
            ascent: 800.0,
            descent: 0.0,
            line_gap: 0.0,
        });

        Ok(Font {
            font: inner_font,
            height: 0.0,
            bitmap_font: true,
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
    let test_char = '😀';
    //let test_char = 'j';

    let font = Font::from_file_bitmap("/System/Library/Fonts/Apple Color Emoji.ttc").unwrap();

    //let font = Font::from_file("/System/Library/Fonts/HelveticaNeue.ttc").unwrap();
    println!("Ascender: {:?}", font.font.inner().ascender());
    println!("Descender: {:?}", font.font.inner().descender());
    println!("Height: {:?}", font.font.inner().height());

    let emoji_ranges: [Range<u32>; 1] = [
        0x1F601..0x1F64F,
        //0x2702..0x27B0,
        //0x1F680..0x1F6C0,
        //0x1F170..0x1F251
    ];

    /*for range in &emoji_ranges {

        for i in range.clone() {
            let c = unsafe { std::mem::transmute::<u32, char>(i) };
            let id = font.get_inner().inner().glyph_index(c).unwrap();
            let bb = font.get_inner().inner().glyph_bounding_box(id);
            println!("bb: {:?}", bb);
            let advance_width = font.get_inner().inner().glyph_hor_advance(id).unwrap();
            println!("Advance_width: {}", advance_width);
            println!("{}", c);
        }
    }*/
    let glyph = font.get_inner().glyph(test_char);
    let scale = Font::size_to_scale(14, 1.0);
    let scaled_glyph = glyph.scaled(scale);
    let positioned_glyph = scaled_glyph.positioned(point(0.0, 20.0));
    println!("Positioned bb: {:?}", positioned_glyph.pixel_bounding_box());
    println!("Exact bb: {:?}", positioned_glyph.unpositioned().exact_bounding_box());

    //let image = font.get_glyph_raster_image(test_char, 64).unwrap();
    let id = font.get_inner().inner().glyph_index(test_char).unwrap();
    let bb = font.get_inner().inner().glyph_bounding_box(id);
    println!("glyph bb: {:?}", bb);
    let advance_width = font.get_inner().inner().glyph_hor_advance(id).unwrap();
    println!("Advance_width: {}", advance_width);
    println!("Glyph name: {:?}", font.get_inner().inner().glyph_name(id));

    //image.save("/Users/holgergottchristensen/Documents/carbide/target/smile_new.png").unwrap();
}