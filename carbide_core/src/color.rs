//!
//! A library providing simple `Color` and `Gradient` render along with useful transformations and
//! presets.
//!
//!
//! Inspiration taken from [elm-lang's color module]
//! (https://github.com/elm-lang/core/blob/62b22218c42fb8ccc996c86bea450a14991ab815/src/Color.elm)
//!
//!
//! Module for working with colors. Includes [RGB](https://en.wikipedia.org/wiki/RGB_color_model)
//! and [HSL](http://en.wikipedia.org/wiki/HSL_and_HSV) creation, gradients and built-in names.
//!

use std::f32::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::animation::Animatable;
use crate::utils::{degrees, fmod, turns};

/// Color supporting RGB and HSL variants.
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Color {
    /// Red, Green, Blue, Alpha - All values' scales represented between 0.0 and 1.0.
    Rgba(f32, f32, f32, f32),
    /// Hue, Saturation, Lightness, Alpha - all values scales represented between 0.0 and 1.0.
    Hsla(f32, f32, f32, f32),
}

impl Default for Color {
    fn default() -> Self {
        // 0.8, 0.0, 0.0, 1.0
        RED
    }
}

impl Color {
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Color {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        Color::Rgba(r, g, b, 1.0)
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        return rgb(
            rng.gen_range(0.0, 1.0),
            rng.gen_range(0.0, 1.0),
            rng.gen_range(0.0, 1.0),
        );
    }

    pub fn time() -> Self {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Could not get duration since UNIX_EPOCH");
        let duration_since = duration.as_millis() / 8 % 360;
        return hsl(
            f32::to_radians(duration_since as f32),
            1.0,
            0.5
        )
    }
}

/// Create RGB colors with an alpha component for transparency.
/// The alpha component is specified with numbers between 0 and 1.
#[inline]
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::Rgba(r, g, b, a)
}

/// Create RGB colors from numbers between 0.0 and 1.0.
#[inline]
pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgba(r, g, b, 1.0)
}

/// Create RGB colors from numbers between 0 and 255 inclusive.
/// The alpha component is specified with numbers between 0 and 1.
#[inline]
pub fn rgba_bytes(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::Rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Create RGB colors from numbers between 0 and 255 inclusive.
#[inline]
pub fn rgb_bytes(r: u8, g: u8, b: u8) -> Color {
    rgba_bytes(r, g, b, 1.0)
}

/// Create [HSL colors](http://en.wikipedia.org/wiki/HSL_and_HSV) with an alpha component for
/// transparency.
#[inline]
pub fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Color {
    Color::Hsla(
        hue - turns((hue / (2.0 * PI)).floor()),
        saturation,
        lightness,
        alpha,
    )
}

/// Create [HSL colors](http://en.wikipedia.org/wiki/HSL_and_HSV). This gives you access to colors
/// more like a color wheel, where all hues are arranged in a circle that you specify with radians.
///
///   red        = hsl(degrees(0.0)   , 1.0 , 0.5)
///   green      = hsl(degrees(120.0) , 1.0 , 0.5)
///   blue       = hsl(degrees(240.0) , 1.0 , 0.5)
///   pastel_red = hsl(degrees(0.0)   , 0.7 , 0.7)
///
/// To cycle through all colors, just cycle through degrees. The saturation level is how vibrant
/// the color is, like a dial between grey and bright colors. The lightness level is a dial between
/// white and black.
#[inline]
pub fn hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
    hsla(hue, saturation, lightness, 1.0)
}

/// Produce a gray based on the input. 0.0 is white, 1.0 is black.
pub fn grayscale(p: f32) -> Color {
    Color::Hsla(0.0, 0.0, 1.0 - p, 1.0)
}

/// Clamp a f32 between 0f32 and 1f32.
fn clampf32(f: f32) -> f32 {
    if f < 0.0 {
        0.0
    } else if f > 1.0 {
        1.0
    } else {
        f
    }
}

impl Color {

    /// Blend the colors between from and to, by retrieving their hsla values and
    /// interpolating the hue angle and s, l and a value. This will blend through the smallest
    /// way around the hue value, clockwise or counter clockwise.
    /// A related blending function is [Self::rgba_blend]
    pub fn hsla_blend(from: &Color, to: &Color, percentage: f64) -> Color {
        let from_hsla = from.to_hsl();
        let to_hsla = to.to_hsl();

        let from_h = from_hsla.0 * 180.0 / std::f32::consts::PI;
        let to_h = to_hsla.0 * 180.0 / std::f32::consts::PI;

        let d = to_h - from_h;
        let delta = d + if d.abs() > 180.0 {
            if d < 0.0 {
                360.0
            } else {
                -360.0
            }
        } else {
            0.0
        };

        let mut new_angle_deg = from_h as f64 + (percentage * delta as f64);

        if new_angle_deg < 0.0 {
            new_angle_deg = 360.0 + new_angle_deg;
        } else if new_angle_deg >= 360.0 {
            new_angle_deg = new_angle_deg - 360.0
        }

        Color::Hsla(
            (new_angle_deg as f32).to_radians().abs(),
            from_hsla.1.interpolate(&to_hsla.1, percentage),
            from_hsla.2.interpolate(&to_hsla.2, percentage),
            from_hsla.3.interpolate(&to_hsla.3, percentage),
        )
    }

    /// Blend between the color from and to by interpolating the r, g, b and a values
    /// of the two colors.
    /// A related blending function is [Self::hsla_blend]
    pub fn rgba_blend(from: &Color, to: &Color, percentage: f64) -> Color {
        let from_rgba = from.to_rgb();
        let to_rgba = to.to_rgb();
        Color::Rgba(
            from_rgba.0.interpolate(&to_rgba.0, percentage),
            from_rgba.1.interpolate(&to_rgba.1, percentage),
            from_rgba.2.interpolate(&to_rgba.2, percentage),
            from_rgba.3.interpolate(&to_rgba.3, percentage),
        )
    }

    /// The percent should be between 0 and 1.
    /// Lighting with negative values will darken the color.
    pub fn lightened(self, percent: f32) -> Color {
        let Hsla(h, s, l, a) = self.to_hsl();
        Color::Hsla(h, s, clampf32(l + percent), a)
    }

    /// The percent should be between 0 and 1.
    /// Darkening with negative values will lighten the color.
    pub fn darkened(self, percent: f32) -> Color {
        let Hsla(h, s, l, a) = self.to_hsl();
        Color::Hsla(h, s, clampf32(l - percent), a)
    }

    /// Produce a complementary color. The two colors will accent each other. This is the same as
    /// rotating the hue by 180 degrees.
    pub fn complement(self) -> Color {
        match self {
            Color::Hsla(h, s, l, a) => hsla(h + f32::to_radians(180.0), s, l, a),
            Color::Rgba(r, g, b, a) => {
                let (h, s, l) = rgb_to_hsl(r, g, b);
                hsla(h + f32::to_radians(180.0), s, l, a)
            }
        }
    }



    /// Return either black or white, depending which contrasts the Color the most. This will be
    /// useful for determining a readable color for text on any given background Color.
    pub fn plain_contrast(self) -> Color {
        match self {
            Color::Hsla(h, s, l, _) => {
                let (r, g, b) = hsl_to_rgb(h, s, l);
                rgb(r, g, b).plain_contrast()
            }
            Color::Rgba(r, g, b, _) => {
                let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                if l > 0.5 {
                    BLACK
                } else {
                    WHITE
                }
            }
        }
    }

    /// Extract the components of a color in the HSL format.
    pub fn to_hsl(self) -> Hsla {
        match self {
            Color::Hsla(h, s, l, a) => Hsla(h, s, l, a),
            Color::Rgba(r, g, b, a) => {
                let (h, s, l) = rgb_to_hsl(r, g, b);
                Hsla(h, s, l, a)
            }
        }
    }

    /// Extract the components of a color in the RGB format.
    pub fn to_rgb(self) -> Rgba {
        match self {
            Color::Rgba(r, g, b, a) => Rgba(r, g, b, a),
            Color::Hsla(h, s, l, a) => {
                let (r, g, b) = hsl_to_rgb(h, s, l);
                Rgba(r, g, b, a)
            }
        }
    }

    /// Pre multiply the color. https://microsoft.github.io/Win2D/WinUI3/html/PremultipliedAlpha.htm
    pub fn pre_multiply(self) -> Self {
        let Rgba(r, g, b, a) = self.to_rgb();
        Color::Rgba(r * a, g * a, b * a, a)
    }

    /// Extract the components of a color in the RGB format within a fixed-size array.
    pub fn to_fsa(self) -> [f32; 4] {
        let Rgba(r, g, b, a) = self.to_rgb();
        [r, g, b, a]
    }

    /// Same as `to_fsa`, except r, g, b and a are represented in byte form.
    pub fn to_byte_fsa(self) -> [u8; 4] {
        let Rgba(r, g, b, a) = self.to_rgb();
        [
            f32_to_byte(r),
            f32_to_byte(g),
            f32_to_byte(b),
            f32_to_byte(a),
        ]
    }

    // /// Return the hex representation of this color in the format #RRGGBBAA
    // /// e.g. `Color(1.0, 0.0, 5.0, 1.0) == "#FF0080FF"`
    // pub fn to_hex(self) -> String {
    //     let vals = self.to_byte_fsa();
    //     let hex = vals.to_hex().to_ascii_uppercase();
    //     format!("#{}", &hex)
    // }

    /// Calculate and return the luminance of the Color.
    pub fn luminance(&self) -> f32 {
        match *self {
            Color::Rgba(r, g, b, _) => (r + g + b) / 3.0,
            Color::Hsla(_, _, l, _) => l,
        }
    }

    /// Return the same color but with the given luminance.
    pub fn with_luminance(self, l: f32) -> Color {
        let Hsla(h, s, _, a) = self.to_hsl();
        Color::Hsla(h, s, l, a)
    }

    /// Return the same color but with the alpha multiplied by the given alpha.
    pub fn alpha(self, alpha: f32) -> Color {
        match self {
            Color::Rgba(r, g, b, a) => Color::Rgba(r, g, b, a * alpha),
            Color::Hsla(h, s, l, a) => Color::Hsla(h, s, l, a * alpha),
        }
    }

    /// Return the same color but with the given alpha.
    pub fn with_alpha(self, a: f32) -> Color {
        match self {
            Color::Rgba(r, g, b, _) => Color::Rgba(r, g, b, a),
            Color::Hsla(h, s, l, _) => Color::Hsla(h, s, l, a),
        }
    }

    /// Return the opacity of the color.
    pub fn opacity(self) -> f32 {
        match self {
            Color::Rgba(_, _, _, a) => a,
            Color::Hsla(_, _, _, a) => a,
        }
    }

    /// Return the same color but with the given opacity/alpha.
    pub fn with_opacity(self, a: f32) -> Color {
        match self {
            Color::Rgba(r, g, b, _) => Color::Rgba(r, g, b, a),
            Color::Hsla(h, s, l, _) => Color::Hsla(h, s, l, a),
        }
    }

    /// Return a highlighted version of the current Color.
    pub fn highlighted(self) -> Color {
        let luminance = self.luminance();
        let Rgba(r, g, b, a) = self.to_rgb();
        let (r, g, b) = {
            if luminance > 0.8 {
                (r - 0.2, g - 0.2, b - 0.2)
            } else if luminance < 0.2 {
                (r + 0.2, g + 0.2, b + 0.2)
            } else {
                (
                    clampf32((1.0 - r) * 0.5 * r + r),
                    clampf32((1.0 - g) * 0.1 * g + g),
                    clampf32((1.0 - b) * 0.1 * b + b),
                )
            }
        };
        let a = clampf32((1.0 - a) * 0.5 + a);
        rgba(r, g, b, a)
    }

    /// Return a clicked version of the current Color.
    pub fn clicked(&self) -> Color {
        let luminance = self.luminance();
        let Rgba(r, g, b, a) = self.to_rgb();
        let (r, g, b) = {
            if luminance > 0.8 {
                (r, g - 0.2, b - 0.2)
            } else if luminance < 0.2 {
                (r + 0.4, g + 0.2, b + 0.2)
            } else {
                (
                    clampf32((1.0 - r) * 0.75 + r),
                    clampf32((1.0 - g) * 0.25 + g),
                    clampf32((1.0 - b) * 0.25 + b),
                )
            }
        };
        let a = clampf32((1.0 - a) * 0.75 + a);
        rgba(r, g, b, a)
    }

    /// Return the Color's invert.
    pub fn invert(self) -> Color {
        let Rgba(r, g, b, a) = self.to_rgb();
        rgba((r - 1.0).abs(), (g - 1.0).abs(), (b - 1.0).abs(), a)
    }

    /// Return the red component. The value returned should be between 0.0 and 1.0
    pub fn red(&self) -> f32 {
        let Rgba(r, _, _, _) = self.to_rgb();
        r
    }

    /// Return the same color but with the given red component.
    /// The value provided should be between 0.0 and 1.0
    pub fn with_red(self, r: f32) -> Color {
        assert!((0.0 <= r && r <= 1.0), "The value r={} should be [0.0, 1.0]", r);
        let Rgba(_, g, b, a) = self.to_rgb();
        rgba(r, g, b, a)
    }

    /// Return the green value. The value returned should be between 0.0 and 1.0
    pub fn green(&self) -> f32 {
        let Rgba(_, g, _, _) = self.to_rgb();
        g
    }

    /// Return the same color but with the given green component.
    /// The value provided should be between 0.0 and 1.0
    pub fn with_green(self, g: f32) -> Color {
        assert!((0.0 <= g && g <= 1.0), "The value g={} should be [0.0, 1.0]", g);
        let Rgba(r, _, b, a) = self.to_rgb();
        rgba(r, g, b, a)
    }

    /// Return the blue value. The value returned should be between 0.0 and 1.0
    pub fn blue(&self) -> f32 {
        let Rgba(_, _, b, _) = self.to_rgb();
        b
    }

    /// Return the same color but with the given green component.
    /// The value provided should be between 0.0 and 1.0
    pub fn with_blue(self, b: f32) -> Color {
        assert!((0.0 <= b && b <= 1.0), "The value b={} should be [0.0, 1.0]", b);
        let Rgba(r, g, _, a) = self.to_rgb();
        rgba(r, g, b, a)
    }

    /// Return the hue value. The value returned should be between 0.0 and 1.0
    pub fn hue(&self) -> f32 {
        let Hsla(h, _, _, _) = self.to_hsl();
        h
    }

    /// Return the same color but with the given hue.
    /// The value returned should be between 0.0 and 1.0
    pub fn with_hue(self, h: f32) -> Color {
        assert!((0.0 <= h && h <= 1.0), "The value h={} should be [0.0, 1.0]", h);
        let Hsla(_, s, l, a) = self.to_hsl();
        hsla(h, s, l, a)
    }

    /// Return the saturation value. The value returned should be between 0.0 and 1.0
    pub fn saturation(&self) -> f32 {
        let Hsla(_, s, _, _) = self.to_hsl();
        s
    }

    /// Return the same color but with the given saturation.
    /// The value returned should be between 0.0 and 1.0
    pub fn with_saturation(self, s: f32) -> Color {
        assert!((0.0 <= s && s <= 1.0), "The value s={} should be [0.0, 1.0]", s);
        let Hsla(h, _, l, a) = self.to_hsl();
        hsla(h, s, l, a)
    }

    /// Return the lightness value. The value returned should be between 0.0 and 1.0
    pub fn lightness(&self) -> f32 {
        let Hsla(_, s, _, _) = self.to_hsl();
        s
    }

    /// Return the same color but with the given lightness.
    /// The value returned should be between 0.0 and 1.0
    pub fn with_lightness(self, l: f32) -> Color {
        assert!((0.0 <= l && l <= 1.0), "The value l={} should be [0.0, 1.0]", l);
        let Hsla(h, s, _, a) = self.to_hsl();
        hsla(h, s, l, a)
    }

    /// Set the red value.
    /// Notice: This will mutate self. Use [Self::with_red()] for a pure function.
    pub fn set_red(&mut self, r: f32) {
        let Rgba(_, g, b, a) = self.to_rgb();
        *self = rgba(r, g, b, a);
    }

    /// Set the green
    /// Notice: This will mutate self. Use [Self::with_green()] for a pure function.
    pub fn set_green(&mut self, g: f32) {
        let Rgba(r, _, b, a) = self.to_rgb();
        *self = rgba(r, g, b, a);
    }

    /// Set the blue value.
    /// Notice: This will mutate self. Use [Self::with_blue()] for a pure function.
    pub fn set_blue(&mut self, b: f32) {
        let Rgba(r, g, _, a) = self.to_rgb();
        *self = rgba(r, g, b, a);
    }
}

/// The parts of HSL along with an alpha for transparency.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hsla(pub f32, pub f32, pub f32, pub f32);

impl From<Color> for Hsla {
    fn from(color: Color) -> Self {
        color.to_hsl()
    }
}

impl From<Hsla> for Color {
    fn from(Hsla(h, s, l, a): Hsla) -> Self {
        Color::Hsla(h, s, l, a)
    }
}

/// The parts of RGB along with an alpha for transparency.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl From<Color> for Rgba {
    fn from(color: Color) -> Self {
        color.to_rgb()
    }
}

impl From<Rgba> for Color {
    fn from(Rgba(r, g, b, a): Rgba) -> Self {
        Color::Rgba(r, g, b, a)
    }
}

impl Into<[f32; 4]> for Rgba {
    fn into(self) -> [f32; 4] {
        let Rgba(r, g, b, a) = self;
        [r, g, b, a]
    }
}

/// Convert an f32 color to a byte.
#[inline]
pub fn f32_to_byte(c: f32) -> u8 {
    (c * 255.0) as u8
}

/// Pure function for converting rgb to hsl.
/// * Inputs expected to be between `0.0` and `1.0`.
/// * Outputs `[0.0, 2*PI)` for `h`, `[0.0, 1.0]` for both `s` and `l`
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let c_max = r.max(g).max(b);
    let c_min = r.min(g).min(b);
    let c = c_max - c_min;

    let hue = if c == 0.0 {
        // If there's no difference in the channels we have grayscale, so the hue is undefined.
        0.0
    } else {
        f32::to_radians(60.0)
            * if c_max == r {
            fmod((g - b) / c, 6)
        } else if c_max == g {
            ((b - r) / c) + 2.0
        } else {
            ((r - g) / c) + 4.0
        }
    };

    let lightness = (c_max + c_min) / 2.0;
    let saturation = if lightness == 0.0 || lightness == 1.0 {
        0.0
    } else {
        c / (1.0 - (2.0 * lightness - 1.0).abs())
    };
    (hue, saturation, lightness)
}

/// Pure function for converting hsl to rgb.
pub fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue = hue / f32::to_radians(60.0);
    let x = chroma * (1.0 - (fmod(hue, 2) - 1.0).abs());
    let (r, g, b) = match hue {
        hue if hue < 0.0 => (0.0, 0.0, 0.0),
        hue if hue < 1.0 => (chroma, x, 0.0),
        hue if hue < 2.0 => (x, chroma, 0.0),
        hue if hue < 3.0 => (0.0, chroma, x),
        hue if hue < 4.0 => (0.0, x, chroma),
        hue if hue < 5.0 => (x, 0.0, chroma),
        hue if hue < 6.0 => (chroma, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };
    let m = lightness - chroma / 2.0;
    (r + m, g + m, b + m)
}
//
// /// Linear or Radial Gradient.
// #[derive(Clone, Debug)]
// pub enum Gradient {
//     /// Takes a start and end point and then a series of color stops that indicate how to
//     /// interpolate between the start and end points.
//     Linear((f64, f64), (f64, f64), Vec<(f64, Color)>),
//     /// First takes a start point and inner radius. Then takes an end point and outer radius.
//     /// It then takes a series of color stops that indicate how to interpolate between the
//     /// inner and outer circles.
//     Radial((f64, f64), f64, (f64, f64), f64, Vec<(f64, Color)>),
// }
//
// /// Create a linear gradient.
// pub fn linear(start: (f64, f64), end: (f64, f64), colors: Vec<(f64, Color)>) -> Gradient {
//     Gradient::Linear(start, end, colors)
// }
//
// /// Create a radial gradient.
// pub fn radial(
//     start: (f64, f64),
//     start_r: f64,
//     end: (f64, f64),
//     end_r: f64,
//     colors: Vec<(f64, Color)>,
// ) -> Gradient {
//     Gradient::Radial(start, start_r, end, end_r, colors)
// }

/// Built-in colors.
///
/// These colors come from the
/// [Tango palette](http://tango.freedesktop.org/Tango_Icon_Theme_Guidelines) which provides
/// aesthetically reasonable defaults for colors. Each color also comes with a light and dark
/// version.

macro_rules! make_color {
    ($r:expr, $g:expr, $b:expr) => {
        Color::Rgba($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0, 1.0)
    };
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        Color::Rgba(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            $a as f32 / 255.0,
        )
    };
}

/// Scarlet Red - Light - #EF2929                         
pub const LIGHT_RED: Color = make_color!(239, 41, 41);
/// Scarlet Red - Regular - #CC0000                       
pub const RED: Color = make_color!(204, 0, 0);
/// Scarlet Red - Dark - #A30000                          
pub const DARK_RED: Color = make_color!(164, 0, 0);

/// Orange - Light - #FCAF3E                              
pub const LIGHT_ORANGE: Color = make_color!(252, 175, 62);
/// Orange - Regular - #F57900                            
pub const ORANGE: Color = make_color!(245, 121, 0);
/// Orange - Dark - #CE5C00                               
pub const DARK_ORANGE: Color = make_color!(206, 92, 0);

/// Butter - Light - #FCE94F                              
pub const LIGHT_YELLOW: Color = make_color!(252, 233, 79);
/// Butter - Regular - #EDD400                            
pub const YELLOW: Color = make_color!(237, 212, 0);
/// Butter - Dark - #C4A000                               
pub const DARK_YELLOW: Color = make_color!(196, 160, 0);

/// Chameleon - Light - #8AE234                           
pub const LIGHT_GREEN: Color = make_color!(138, 226, 52);
/// Chameleon - Regular - #73D216                         
pub const GREEN: Color = make_color!(115, 210, 22);
/// Chameleon - Dark - #4E9A06                            
pub const DARK_GREEN: Color = make_color!(78, 154, 6);

/// Sky Blue - Light - #729FCF                            
pub const LIGHT_BLUE: Color = make_color!(114, 159, 207);
/// Sky Blue - Regular - #3465A4                          
pub const BLUE: Color = make_color!(52, 101, 164);
/// Sky Blue - Dark - #204A87                             
pub const DARK_BLUE: Color = make_color!(32, 74, 135);

/// Plum - Light - #AD7FA8                                
pub const LIGHT_PURPLE: Color = make_color!(173, 127, 168);
/// Plum - Regular - #75507B                              
pub const PURPLE: Color = make_color!(117, 80, 123);
/// Plum - Dark - #5C3566                                 
pub const DARK_PURPLE: Color = make_color!(92, 53, 102);

/// Chocolate - Light - #E9B96E                           
pub const LIGHT_BROWN: Color = make_color!(233, 185, 110);
/// Chocolate - Regular - #C17D11                         
pub const BROWN: Color = make_color!(193, 125, 17);
/// Chocolate - Dark - #8F5902                            
pub const DARK_BROWN: Color = make_color!(143, 89, 2);

/// Straight Black.                                       
pub const BLACK: Color = make_color!(0, 0, 0);
/// Straight White.                                       
pub const WHITE: Color = make_color!(255, 255, 255);

/// Aluminium - Light - #EEEEEC                           
pub const LIGHT_GREY: Color = make_color!(238, 238, 236);
/// Aluminium - Regular - #D3D7CF                         
pub const GREY: Color = make_color!(211, 215, 207);
/// Aluminium - Dark - #BABDB6                            
pub const DARK_GREY: Color = make_color!(186, 189, 182);

/// Charcoal - Light - #888A85                            
pub const LIGHT_CHARCOAL: Color = make_color!(136, 138, 133);
/// Charcoal - Regular - #555753                          
pub const CHARCOAL: Color = make_color!(85, 87, 83);
/// Charcoal - Dark - #2E3436                             
pub const DARK_CHARCOAL: Color = make_color!(46, 52, 54);

/// Transparent
pub const TRANSPARENT: Color = Color::Rgba(0.0, 0.0, 0.0, 0.0);

#[test]
fn plain_contrast_should_weight_colors() {
    // Contrast tests.
    // Black and white : Simple tests.
    let white_contrast = rgb(1.0, 1.0, 1.0).plain_contrast();
    let Rgba(r, g, b, _) = white_contrast.to_rgb();

    assert_eq!(r, 0.0);
    assert_eq!(g, 0.0);
    assert_eq!(b, 0.0);

    let black_contrast = rgb(0.0, 0.0, 0.0).plain_contrast();
    let Rgba(r, g, b, _) = black_contrast.to_rgb();

    assert_eq!(r, 1.0);
    assert_eq!(g, 1.0);
    assert_eq!(b, 1.0);

    // Weighting for greenish colors.
    // 0.29+0.9+0.29 = 1.48 -> Non-weighted contrast would be white.
    let greenish = rgb(0.29, 0.90, 0.29).plain_contrast();
    let Rgba(r, g, b, _) = greenish.to_rgb();

    assert_eq!(r, 0.0);
    assert_eq!(g, 0.0);
    assert_eq!(b, 0.0);

    // Weighting for non-greenish colors.
    // 0.71+0.1+0.71 = 1.52 -> Non-weighted contrast would be black.
    let purplish = rgb(0.71, 0.10, 0.71).plain_contrast();
    let Rgba(r, g, b, _) = purplish.to_rgb();

    assert_eq!(r, 1.0);
    assert_eq!(g, 1.0);
    assert_eq!(b, 1.0);
}
