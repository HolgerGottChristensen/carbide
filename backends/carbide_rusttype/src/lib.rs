//! RustType is a pure Rust alternative to libraries like FreeType.
//!
//! The current capabilities of RustType:
//!
//! * Reading TrueType formatted fonts and font collections. This includes
//!   `*.ttf` as well as a subset of `*.otf` font files.
//! * Retrieving glyph shapes and commonly used properties for a font and its
//!   glyphs.
//! * Laying out glyphs horizontally using horizontal and vertical metrics, and
//!   glyph-pair-specific kerning.
//! * Rasterising glyphs with sub-pixel positioning using an accurate analytical
//!   algorithm (not based on sampling).
//! * Managing a font cache on the GPU with the `gpu_cache` module. This keeps
//!   recently used glyph renderings in a dynamic cache in GPU memory to
//!   minimise texture uploads per-frame. It also allows you keep the draw call
//!   count for text very low, as all glyphs are kept in one GPU texture.
//!
//! Notable things that RustType does not support *yet*:
//!
//! * OpenType formatted fonts that are not just TrueType fonts (OpenType is a
//!   superset of TrueType). Notably there is no support yet for cubic Bezier
//!   curves used in glyphs.
//! * Font hinting.
//! * Ligatures of any kind.
//! * Some less common TrueType sub-formats.
//! * Right-to-left and vertical text layout.
//!
//! # Getting Started
//!
//! To hit the ground running with RustType, look at the `ascii.rs` example
//! supplied with the crate. It demonstrates loading a font file, rasterising an
//! arbitrary string, and displaying the result as ASCII art. If you prefer to
//! just look at the documentation, the entry point for loading fonts is
//! `Font`, from which you can access individual fonts, then their
//! glyphs.
//!
//! # Glyphs
//!
//! The glyph API uses wrapper structs to augment a glyph with information such
//! as scaling and positioning, making relevant methods that make use of this
//! information available as appropriate. For example, given a `Glyph` `glyph`
//! obtained directly from a `Font`:
//!
//! ```no_run
//! # use carbide_rusttype::*;
//! use carbide_rusttype::glyph::Glyph;
//! # let glyph: Glyph<'static> = unimplemented!();
//! // One of the few things you can do with an unsized, positionless glyph is get its id.
//! let id = glyph.id();
//! let glyph = glyph.scaled(Scale::uniform(10.0));
//! // Now glyph is a ScaledGlyph, you can do more with it, as well as what you can do with Glyph.
//! // For example, you can access the correctly scaled horizontal metrics for the glyph.
//! let h_metrics = glyph.h_metrics();
//! let glyph = glyph.positioned(point(5.0, 3.0));
//! // Now glyph is a PositionedGlyph, and you can do even more with it, e.g. drawing.
//! glyph.draw(|x, y, v| {}); // In this case the pixel values are not used.
//! ```
//!
//! # Unicode terminology
//!
//! This crate uses terminology for computerised typography as specified by the
//! Unicode standard. If you are not sure of the differences between a code
//! point, a character, and a glyph, you may want to check the [official Unicode
//! glossary](http://unicode.org/glossary/), or alternatively, here's my take on
//! it from a practical perspective:
//!
//! * A character is what you would conventionally call a single symbol,
//!   independent of its appearance or representation in a particular font.
//!   Examples include `a`, `A`, `ä`, `å`, `1`, `*`, `Ω`, etc.
//! * A Unicode code point is the particular number that the Unicode standard
//!   associates with a particular character. Note however that code points also
//!   exist for things not conventionally thought of as characters by
//!   themselves, but can be combined to form characters, such as diacritics
//!   like accents. These "characters" are known in Unicode as "combining
//!   characters". E.g., a diaeresis (`¨`) has the code point U+0308. If this
//!   code point follows the code point U+0055 (the letter `u`), this sequence
//!   represents the character `ü`. Note that there is also a single codepoint
//!   for `ü`, U+00FC. This means that what visually looks like the same string
//!   can have multiple different Unicode representations. Some fonts will have
//!   glyphs (see below) for one sequence of codepoints, but not another that
//!   has the same meaning. To deal with this problem it is recommended to use
//!   Unicode normalisation, as provided by, for example, the
//!   [unicode-normalization](http://crates.io/crates/unicode-normalization)
//!   crate, to convert to code point sequences that work with the font in
//!   question. Typically a font is more likely to support a single code point
//!   vs. a sequence with the same meaning, so the best normalisation to use is
//!   "canonical recomposition", known as NFC in the normalisation crate.
//! * A glyph is a particular font's shape to draw the character for a
//!   particular Unicode code point. This will have its own identifying number
//!   unique to the font, its ID.


pub use owned_ttf_parser::OutlineBuilder;

pub use font::*;

pub use geometry::{point, Point, Rect, vector, Vector};
pub use glyph::Glyph;
pub use positioned_glyph::PositionedGlyph;
pub use scaled_glyph::ScaledGlyph;
pub use glyph_iter::GlyphIter;

mod font;
mod geometry;
mod outliner;
mod glyph;
mod positioned_glyph;
mod scaled_glyph;
mod glyph_iter;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GlyphId(pub u16);

impl From<owned_ttf_parser::GlyphId> for GlyphId {
    fn from(id: owned_ttf_parser::GlyphId) -> Self {
        Self(id.0)
    }
}
impl From<GlyphId> for owned_ttf_parser::GlyphId {
    fn from(id: GlyphId) -> Self {
        Self(id.0)
    }
}

/// The "horizontal metrics" of a glyph. This is useful for calculating the
/// horizontal offset of a glyph from the previous one in a string when laying a
/// string out horizontally.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct HMetrics {
    /// The horizontal offset that the origin of the next glyph should be from
    /// the origin of this glyph.
    pub advance_width: f32,
    /// The horizontal offset between the origin of this glyph and the leftmost
    /// edge/point of the glyph.
    pub left_side_bearing: f32,
}

/// The "vertical metrics" of a font at a particular scale. This is useful for
/// calculating the amount of vertical space to give a line of text, and for
/// computing the vertical offset between successive lines.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct VMetrics {
    /// The highest point that any glyph in the font extends to above the
    /// baseline. Typically positive.
    pub ascent: f32,
    /// The lowest point that any glyph in the font extends to below the
    /// baseline. Typically negative.
    pub descent: f32,
    /// The gap to leave between the descent of one line and the ascent of the
    /// next. This is of course only a guideline given by the font's designers.
    pub line_gap: f32,
}

impl core::ops::Mul<f32> for VMetrics {
    type Output = VMetrics;

    fn mul(self, rhs: f32) -> Self {
        Self {
            ascent: self.ascent * rhs,
            descent: self.descent * rhs,
            line_gap: self.line_gap * rhs,
        }
    }
}

/// Defines the size of a rendered face of a font, in pixels, horizontally and
/// vertically. A vertical scale of `y` pixels means that the distance between
/// the ascent and descent lines (see `VMetrics`) of the face will be `y`
/// pixels. If `x` and `y` are equal the scaling is uniform. Non-uniform scaling
/// by a factor *f* in the horizontal direction is achieved by setting `x` equal
/// to *f* times `y`.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Scale {
    /// Horizontal scale, in pixels.
    pub x: f32,
    /// Vertical scale, in pixels.
    pub y: f32,
}

impl Scale {
    /// Uniform scaling, equivalent to `Scale { x: s, y: s }`.
    #[inline]
    pub fn uniform(s: f32) -> Scale {
        Scale { x: s, y: s }
    }
}
/// A trait for types that can be converted into a `GlyphId`, in the context of
/// a specific font.
///
/// Many `rusttype` functions that operate on characters accept values of any
/// type that implements `IntoGlyphId`. Such types include `char`, `Codepoint`,
/// and obviously `GlyphId` itself.
pub trait IntoGlyphId {
    /// Convert `self` into a `GlyphId`, consulting the index map of `font` if
    /// necessary.
    fn into_glyph_id(self, font: &Font<'_>) -> GlyphId;
}
impl IntoGlyphId for char {
    #[inline]
    fn into_glyph_id(self, font: &Font<'_>) -> GlyphId {
        font.inner()
            .glyph_index(self)
            .unwrap_or(owned_ttf_parser::GlyphId(0))
            .into()
    }
}
impl<G: Into<GlyphId>> IntoGlyphId for G {
    #[inline]
    fn into_glyph_id(self, _font: &Font<'_>) -> GlyphId {
        self.into()
    }
}

pub(crate) trait NearZero {
    /// Returns if this number is kinda pretty much zero.
    fn is_near_zero(self) -> bool;
}
impl NearZero for f32 {
    #[inline]
    fn is_near_zero(self) -> bool {
        self.abs() <= f32::EPSILON
    }
}
