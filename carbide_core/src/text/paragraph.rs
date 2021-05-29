use crate::Scalar;
use crate::text::{Font, FontSize};
use crate::text::section::Section;
use crate::widget::types::text_wrap::Wrap;

#[derive(Debug, Clone)]
pub struct Paragraph {
    text: String,
    max_width: Scalar,
    sections: Vec<Section>,
}

impl Paragraph {
    pub fn new(string: &str, font: &mut Font, font_size: FontSize, wrap_mode: Wrap) -> Paragraph {
        let sections = match wrap_mode {
            Wrap::Character => {
                string
                    .chars()
                    .map(|word| {
                        Section::new(word.to_string().as_str(), font, font_size)
                    }).collect::<Vec<_>>()
            }
            Wrap::Whitespace => {
                string
                    .split(&[' ', '\t'][..])
                    .map(|word| {
                        Section::new(word, font, font_size)
                    }).collect::<Vec<_>>()
            }
            Wrap::None => {
                vec![Section::new(string, font, font_size)]
            }
        };

        let total_width: Scalar = sections.iter().map(|section| section.total_width()).sum();

        Paragraph {
            text: string.to_string(),
            max_width: total_width,
            sections,
        }
    }

    fn recalculate_lines(&mut self, bound_width: Scalar) {
        let mut current_width = 0.0;

        for section in &mut self.sections {
            current_width += section.total_width();
            section.set_line_number((current_width / bound_width) as u32)
        }
    }

    /// Calculates the height of the paragraph based on the parameters and returns the new height
    pub fn height(&mut self, font_size: FontSize, bound_width: Scalar) -> Scalar {
        if self.max_width > bound_width {
            self.recalculate_lines(bound_width);

            let number_of_lines_in_paragraph = self.sections[self.sections.len() - 1].line_number();
            let line_spacing = 1.0;

            number_of_lines_in_paragraph as Scalar
                * font_size as Scalar
                + (number_of_lines_in_paragraph - 1) as Scalar
                * line_spacing
        } else {
            font_size as f64
        }
    }

    pub fn max_width(&self) -> Scalar {
        self.max_width
    }
}