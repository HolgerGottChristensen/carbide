use crate::Scalar;
use crate::text::{Font, FontId, FontSize};
use crate::text::paragraph::Paragraph;
use crate::widget::{Environment, GlobalState};
use crate::widget::types::text_wrap::Wrap;

#[derive(Debug, Clone)]
pub struct Text {
    text: String,
    max_width: Scalar,
    font_id: FontId,
    paragraphs: Vec<Paragraph>,
    latest_available_width: Scalar,
    latest_height: Scalar,
    wrap_mode: Wrap,
}

impl Text {
    pub fn new(font_id: FontId, wrap_mode: Wrap) -> Text {
        Text {
            text: "".to_string(),
            max_width: 0.0,
            font_id,
            paragraphs: vec![],
            latest_available_width: 0.0,
            latest_height: 0.0,
            wrap_mode,
        }
    }

    pub fn update<GS: GlobalState>(&mut self, text: &str, font_size: FontSize, env: &mut Environment<GS>) {
        if text != self.text {
            self.text = text.to_string();
            let font = env.get_font_mut(self.font_id);
            self.recalculate_paragraphs(font, font_size);
        }
    }

    pub fn recalculate_paragraphs(&mut self, font: &mut Font, font_size: FontSize) {
        let paragraphs: Vec<String> = self.text.replace("\r\n", "\n").split('\n').map(|c| c.to_string()).collect();

        self.paragraphs = paragraphs.iter()
            .map(|paragraph| {
                Paragraph::new(&paragraph, font, font_size, self.wrap_mode)
            }).collect();

        let mut max = 0.0;

        for i in self.paragraphs.iter().map(|paragraph| paragraph.max_width()) {
            if i > max {
                max = i
            }
        }

        self.max_width = max;
    }

    pub fn max_width(&self) -> Scalar {
        self.max_width
    }

    pub fn height(&mut self, width_bound: Scalar, font_size: FontSize) -> Scalar {
        if width_bound == self.latest_available_width {
            return self.latest_height;
        }

        self.latest_height = self.paragraphs.iter_mut()
            .map(|paragraph| {
                paragraph.height(font_size, width_bound)
            }).sum();
        self.latest_available_width = width_bound;

        self.latest_height
    }
}