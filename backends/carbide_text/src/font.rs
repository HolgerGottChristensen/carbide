use std::fmt::{Debug, Formatter};
use std::path::Path;
use carbide_core::draw::{Position, Scalar};
use carbide_core::environment::Environment;
use carbide_core::image;
use carbide_core::image::{DynamicImage, GenericImage, Rgba};
use carbide_core::text::{FontId, FontSize, FontStyle, FontWeight};


use carbide_rusttype::{GlyphId, point, Scale, Weight};
use crate::glyph::Glyph;
use crate::text_context::TextContext;


type RustTypeFont = carbide_rusttype::Font<'static>;
type RustTypeScale = carbide_rusttype::Scale;

const POINT_TO_PIXEL: f32 = 1.0;

#[derive(Clone)]
pub struct Font {
    font_id: FontId,
    path: String,
    // Should in the future be a collection of different font weights
    font: RustTypeFont,
}

impl Debug for Font {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .finish()
    }
}

impl Font {
    pub fn glyph_id(&self, c: char) -> Option<GlyphId> {
        self.font.glyph_id(c)
    }

    pub fn glyph_image(&self, id: GlyphId, size: FontSize, scale_factor: Scalar, position_offset: Position) -> Option<DynamicImage> {
        let raster_image = self.font.glyph_raster_image(id, (size as f64 * scale_factor) as u16);

        // If we got a raster image, we return that
        if let Some(raster_image) = raster_image {
            return Some(image::load_from_memory(raster_image.data).unwrap());
        }

        // Lookup the glyph outline
        let scale = Font::size_to_scale(size, scale_factor);

        let positioned_glyph = self
            .get_inner()
            .glyph(id)
            .scaled(scale)
            .positioned(point(position_offset.x() as f32, position_offset.y() as f32));

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

    pub fn glyph_for(
        &self,
        c: char,
        font_size: FontSize,
        scale_factor: Scalar,
    ) -> Option<(Scalar, Glyph)> {
        println!("Looking up glyph for char: {} in font: {}", c, self.path);
        let glyph_id = self.glyph_id(c);

        glyph_id.map(|id| {
            let scale = Font::size_to_scale(font_size, scale_factor);
            let glyph_scaled = self.font.glyph(id).scaled(scale);
            let w = glyph_scaled.h_metrics().advance_width;
            let positioned = glyph_scaled.positioned(point(0.0, 0.0));
            (
                w as f64,
                Glyph::new(c, font_size, self.font_id, positioned, self.is_bitmap(id)),
            )
        })
    }

    fn is_bitmap(&self, id: GlyphId) -> bool {
        self.font.glyph_raster_image(id, 1).is_some()
    }

    pub fn glyphs_for(
        &self,
        text: &str,
        font_size: FontSize,
        scale_factor: Scalar,
        context: &mut TextContext,
    ) -> (Vec<Scalar>, Vec<Glyph>) {
        let scale = Font::size_to_scale(font_size, scale_factor);
        let mut next_width = 0.0;
        let mut widths = vec![];
        let mut glyphs = vec![];
        let mut last = None;

        let glyph_ids = text
            .chars()
            .map(|c| self.glyph_id(c).map(|id| (id, c)).ok_or(c))
            .collect::<Vec<_>>();

        for glyph_id in glyph_ids {
            match glyph_id {
                Ok((id, c)) => {
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
                    glyphs.push(Glyph::new(
                        c,
                        font_size,
                        self.font_id,
                        next,
                        false,
                    ));
                }
                Err(c) => {
                    // Font fallback
                    if let Some(_) = last {
                        widths.push(next_width);
                        next_width = 0.0;
                    }
                    last = None;

                    let (width, glyph) = context.get_glyph_from_fallback(c, font_size, scale_factor);
                    glyphs.push(glyph);
                    widths.push(width);
                }
            }
        }
        // Widths are pushed such that they contain the width and the kerning between itself and the
        // next character. If its the last, we push here.
        if let Some(_) = last {
            widths.push(next_width);
        }

        (widths, glyphs)
    }
}

impl Font {
    pub fn id(&self) -> FontId {
        self.font_id
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn weight(&self) -> FontWeight {
        match self.font.inner().weight() {
            Weight::Thin => FontWeight::Thin,
            Weight::ExtraLight => FontWeight::ExtraLight,
            Weight::Light => FontWeight::Light,
            Weight::Normal => FontWeight::Normal,
            Weight::Medium => FontWeight::Medium,
            Weight::SemiBold => FontWeight::SemiBold,
            Weight::Bold => FontWeight::Bold,
            Weight::ExtraBold => FontWeight::ExtraBold,
            Weight::Black => FontWeight::Black,
            Weight::Other(val) => FontWeight::Other(val),
        }
    }

    pub fn style(&self) -> FontStyle {
        if self.font.inner().is_italic() {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        }
    }

    pub fn set_font_id(&mut self, font_id: FontId) {
        self.font_id = font_id;
    }

    pub fn get_inner(&self) -> RustTypeFont {
        // This clone should only be either an ARC clone or reference clone
        self.font.clone()
    }

    fn size_to_scale(font_size: FontSize, scale_factor: Scalar) -> RustTypeScale {
        Font::f32_pt_to_scale(font_size as f32 * scale_factor as f32)
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
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        use std::io::Read;
        let path = path.as_ref();
        let mut file = std::fs::File::open(path).unwrap();
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer).unwrap();
        let inner_font = RustTypeFont::try_from_vec(file_buffer).unwrap();

        Font {
            font_id: usize::MAX,
            path: path.display().to_string(),
            font: inner_font,
        }
    }
}

// #[test]
// fn load_bitmap_font() {
//     let test_char = '😀';
//     //let test_char = 'j';
//     //let test_char = '􁂿';
//
//     let font = Font::from_file_bitmap("/System/Library/Fonts/Apple Color Emoji.ttc").unwrap();
//
//     //let font = Font::from_file("/System/Library/Fonts/HelveticaNeue.ttc").unwrap();
//     //let font = Font::from_file("/System/Library/Fonts/SFCompactText.ttf").unwrap();
//     println!("Ascender: {:?}", font.font.inner().ascender());
//     println!("Descender: {:?}", font.font.inner().descender());
//     println!("Height: {:?}", font.font.inner().height());
//
//     let _emoji_ranges: [std::ops::Range<u32>; 1] = [
//         0x1F601..0x1F64F,
//         //0x2702..0x27B0,
//         //0x1F680..0x1F6C0,
//         //0x1F170..0x1F251
//     ];
//
//     /*for range in &emoji_ranges {
//
//         for i in range.clone() {
//             let c = unsafe { std::mem::transmute::<u32, char>(i) };
//             let id = font.get_inner().inner().glyph_index(c).unwrap();
//             let bb = font.get_inner().inner().glyph_bounding_box(id);
//             println!("bb: {:?}", bb);
//             let advance_width = font.get_inner().inner().glyph_hor_advance(id).unwrap();
//             println!("Advance_width: {}", advance_width);
//             println!("{}", c);
//         }
//     }*/
//     let glyph = font.get_inner().glyph(test_char);
//     let scale = Font::size_to_scale(14, 1.0);
//     let scaled_glyph = glyph.scaled(scale);
//     let positioned_glyph = scaled_glyph.positioned(point(0.0, 20.0));
//     println!("Positioned bb: {:?}", positioned_glyph.pixel_bounding_box());
//     println!(
//         "Exact bb: {:?}",
//         positioned_glyph.unpositioned().exact_bounding_box()
//     );
//     println!("Glyph id: {:?}", positioned_glyph.id());
//
//     //let image = font.get_glyph_raster_image(test_char, 64).unwrap();
//     /*let id = font.get_inner().inner().glyph_index(test_char).unwrap();
//     let bb = font.get_inner().inner().glyph_bounding_box(id);
//     println!("glyph bb: {:?}", bb);
//     let advance_width = font.get_inner().inner().glyph_hor_advance(id).unwrap();
//     println!("Advance_width: {}", advance_width);
//     println!("Glyph name: {:?}", font.get_inner().inner().glyph_name(id));*/
//
//     //image.save("/Users/holgergottchristensen/Documents/carbide/target/smile_new.png").unwrap();
// }

/*#[test]
fn list_fonts() {
    use font_kit::family_name::FamilyName;
    use font_kit::source::SystemSource;
    let system_source = SystemSource::new();
    println!("{:#?}", system_source.select_by_postscript_name(".HelveticaNeueDeskInterface-Regular").unwrap().load().unwrap().postscript_name());
}*/
