use crate::Scalar;
use crate::text::{Font, FontSize};

#[derive(Debug, Clone)]
pub struct Section {
    text: String,
    total_width: Scalar,
    widths: Vec<Scalar>,
    line_in_paragraph: u32,
}

impl Section {
    pub fn new(string: &str, font: &mut Font, font_size: FontSize) -> Section {
        let widths = font.get_char_widths(string, font_size).clone();
        let total_width: Scalar = widths.iter().sum();

        Section {
            text: string.to_string(),
            total_width,
            widths,
            line_in_paragraph: 0,
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
}