/*use crate::{Scalar, Point};
use crate::text::{Font, FontSize};
use crate::text::glyph::Glyph;
use crate::text::text_style::TextStyle;
use crate::text::section::Section::{Char, Word};
use crate::widget::{GlobalState, Environment};
use test::stats::Stats;

#[derive(Debug, Clone)]
pub enum Section { // Todo: Should probably only be a word, such that font kerning works correctly.
    /// Contains a single glyph with cached width and position
    Char(Glyph),
    /// Contains a whole word, with cached width, position, and individual glyph positions
    Word {
        width: Scalar,
        position: Point,
        text: String,
        char_positions: Vec<Point>,
        char_widths: Vec<Scalar>
    }
}

impl Section {
    pub fn new<GS: GlobalState>(string: &str, style: TextStyle, env: &mut Environment<GS>) -> Section {
        if string.chars().count() == 1 {
            let c = string.chars().next().unwrap();

            let glyph = Glyph::new(c, style, env);
            Char(glyph)
        } else {
            let font_size = style.get_font_size();
            let font = style.get_font(env);
            let widths = font.get_char_widths(string, font_size).clone();
            let positions: Vec<Point> = vec![[0.0, 0.0]; widths.len()];
            let position = [0.0, 0.0];
            let width = widths.iter().sum();

            Word {
                width,
                position,
                text: string.to_string(),
                char_positions: positions,
                char_widths: widths
            }
        }
    }

    pub fn total_width(&self) -> Scalar {
        self.total_width
    }

    pub fn line_number(&self) -> u32 {
        self.line_in_paragraph
    }

    pub fn set_line_number(&mut self, number: u32) {
        self.line_in_paragraph = number;
    }
}*/
