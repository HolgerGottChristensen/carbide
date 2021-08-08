use std::fmt::{Debug, Formatter};

use image::{DynamicImage, GenericImage, Rgba};
use rusttype::{GlyphId, point, Scale, VMetrics};

use crate::draw::Position;
use crate::draw::Scalar;
use crate::environment::Environment;
use crate::text::{FontId, FontSize};
use crate::text::glyph::Glyph;

type RustTypeFont = rusttype::Font<'static>;
type RustTypeScale = rusttype::Scale;
type RustTypePoint = rusttype::Point<f32>;

const POINT_TO_PIXEL: f32 = 1.0;

#[derive(Clone)]
pub struct Font {
    font_id: FontId,
    path: String,
    // Should in the future be a collection of different font weights
    font: RustTypeFont,
    height: Scalar,
    bitmap_font: bool,
}

impl Debug for Font {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("is_bitmap", &self.bitmap_font)
            .finish()
    }
}

impl Font {
    pub fn id(&self) -> FontId {
        self.font_id
    }

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

    pub fn set_font_id(&mut self, font_id: FontId) {
        self.font_id = font_id;
    }

    pub fn get_glyph_id(&self, c: char) -> Option<GlyphId> {
        self.font.inner().glyph_index(c).map(|ttf_parser::GlyphId(id)| GlyphId(id))
    }

    pub fn is_bitmap(&self) -> bool {
        self.bitmap_font
    }

    pub fn get_glyph_raster_image_from_id(&self, id: GlyphId, font_size: FontSize, scale_factor: Scalar) -> Option<DynamicImage> {
        let face = self.font.inner();
        let raster_image = face.glyph_raster_image(ttf_parser::GlyphId(id.0), (font_size as f64 * scale_factor) as u16);
        raster_image.map(|raster| {
            image::load_from_memory(raster.data).unwrap()
        })
    }

    /// This will return None if the glyph has no boundingbox, for example the ' ' space character
    pub fn get_glyph_image_from_id(&self, id: GlyphId, font_size: FontSize, scale_factor: Scalar, position_offset: Position) -> Option<DynamicImage> {
        let scale = Font::size_to_scale(font_size, scale_factor);
        let positioned_glyph = self.get_inner().glyph(id).scaled(scale).positioned(point(position_offset.x as f32, position_offset.y as f32));
        if let Some(bb) = positioned_glyph.pixel_bounding_box() {
            let mut image_data = DynamicImage::new_rgba8(bb.width() as u32, bb.height() as u32);
            positioned_glyph.draw(|x, y, value| {
                image_data.put_pixel(x, y, Rgba::from([0, 0, 0, (value * 255.0) as u8]))
            });
            Some(image_data)
        } else {
            None
        }
    }

    pub fn get_inner(&self) -> RustTypeFont {
        // This clone should only be either an ARC clone or reference clone
        self.font.clone()
    }

    fn size_to_scale(font_size: FontSize, scale_factor: Scalar) -> RustTypeScale {
        Font::f32_pt_to_scale(font_size as f32 * scale_factor as f32)
    }

    pub fn get_glyph(&self, c: char, font_size: FontSize, scale_factor: Scalar) -> Option<(Scalar, Glyph)> {
        println!("Looking up glyph for char: {} in font: {}", c, self.path);
        let glyph_id = self.get_glyph_id(c);

        glyph_id.map(|id| {
            let scale = Font::size_to_scale(font_size, scale_factor);
            let glyph_scaled = self.font.glyph(id).scaled(scale);
            let w = glyph_scaled.h_metrics().advance_width;
            let positioned = glyph_scaled.positioned(point(0.0, 0.0));
            (w as f64, Glyph::from((font_size, self.font_id, positioned, self.bitmap_font)))
        })
    }

    pub fn get_glyphs(&self, text: &str, font_size: FontSize, scale_factor: Scalar, env: &mut Environment) -> (Vec<Scalar>, Vec<Glyph>) {
        let scale = Font::size_to_scale(font_size, scale_factor);
        let mut next_width = 0.0;
        let mut widths = vec![];
        let mut glyphs = vec![];
        let mut last = None;

        let glyph_ids = text.chars().map(|c| self.get_glyph_id(c).ok_or(c)).collect::<Vec<_>>();

        for glyph_id in glyph_ids {
            match glyph_id {
                Ok(id) => {
                    // If we have the glyph in our font.
                    let glyph = self.font.glyph(id);
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
                    glyphs.push(Glyph::from((font_size, self.font_id, next, self.bitmap_font)));
                }
                Err(c) => {
                    // Font fallback
                    if let Some(_) = last {
                        widths.push(next_width);
                        next_width = 0.0;
                    }
                    last = None;

                    let (width, glyph) = env.get_glyph_from_fallback(c, font_size, scale_factor);
                    glyphs.push(glyph);
                    widths.push(width);
                }
            }
        };
        // Widths are pushed such that they contain the width and the kerning between itself and the
        // next character. If its the last, we push here.
        if let Some(_) = last {
            widths.push(next_width);
        }

        (widths, glyphs)
    }

    pub fn height(font_size: FontSize, scale_factor: Scalar) -> Scalar {
        Font::size_to_scale(font_size, scale_factor).y as Scalar
    }

    pub fn ascend(&self, font_size: FontSize, scale_factor: Scalar) -> Scalar {
        let scale = Font::size_to_scale(font_size, scale_factor);
        self.font.v_metrics(scale).ascent as Scalar
    }

    pub fn descend(&self, font_size: FontSize, scale_factor: Scalar) -> Scalar {
        let scale = Font::size_to_scale(font_size, scale_factor);
        let metrics = self.font.v_metrics(scale);
        metrics.descent as f64
    }

    pub fn line_gap(&self, font_size: FontSize, scale_factor: Scalar) -> Scalar {
        let scale = Font::size_to_scale(font_size, scale_factor);
        let metrics = self.font.v_metrics(scale);
        metrics.line_gap as f64
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
            font_id: usize::MAX,
            path: path.display().to_string(),
            font: inner_font,
            height: 0.0,
            bitmap_font: false,
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
            ascent: 1200.0,
            descent: 0.0,
            line_gap: 0.0,
        });

        Ok(Font {
            font_id: usize::MAX,
            path: path.display().to_string(),
            font: inner_font,
            height: 0.0,
            bitmap_font: true,
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
    //let test_char = '􁂿';

    let font = Font::from_file_bitmap("/System/Library/Fonts/Apple Color Emoji.ttc").unwrap();

    //let font = Font::from_file("/System/Library/Fonts/HelveticaNeue.ttc").unwrap();
    //let font = Font::from_file("/System/Library/Fonts/SFCompactText.ttf").unwrap();
    println!("Ascender: {:?}", font.font.inner().ascender());
    println!("Descender: {:?}", font.font.inner().descender());
    println!("Height: {:?}", font.font.inner().height());

    let emoji_ranges: [std::ops::Range<u32>; 1] = [
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
    println!("Glyph id: {:?}", positioned_glyph.id());

    //let image = font.get_glyph_raster_image(test_char, 64).unwrap();
    /*let id = font.get_inner().inner().glyph_index(test_char).unwrap();
    let bb = font.get_inner().inner().glyph_bounding_box(id);
    println!("glyph bb: {:?}", bb);
    let advance_width = font.get_inner().inner().glyph_hor_advance(id).unwrap();
    println!("Advance_width: {}", advance_width);
    println!("Glyph name: {:?}", font.get_inner().inner().glyph_name(id));*/

    //image.save("/Users/holgergottchristensen/Documents/carbide/target/smile_new.png").unwrap();
}

/*#[test]
fn list_fonts() {
    use font_kit::family_name::FamilyName;
    use font_kit::source::SystemSource;
    let system_source = SystemSource::new();
    println!("{:#?}", system_source.select_by_postscript_name(".HelveticaNeueDeskInterface-Regular").unwrap().load().unwrap().postscript_name());
}*/