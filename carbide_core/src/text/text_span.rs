use crate::draw::Scalar;
use crate::environment::Environment;
use crate::text::glyph::Glyph;
use crate::text::text_style::TextStyle;
use crate::widget::Widget;

#[derive(Debug, Clone)]
pub enum TextSpan {
    Text {
        style: Option<TextStyle>,
        text: String,
        glyphs: Vec<Glyph>,
        widths: Vec<Scalar>,
        ascend: f64,
        descend: f64,
        line_gap: f64,
    },
    Widget(Box<dyn Widget>),
    NewLine,
}

impl TextSpan {
    pub fn new(string: &str, style: &TextStyle, env: &mut Environment) -> Vec<TextSpan> {
        let scale_factor = env.get_scale_factor();

        let mut res = vec![];

        for line in string.split('\n') {
            let font = style.get_font(env);

            let ascend = font.ascend(style.font_size, scale_factor);
            let descend = font.descend(style.font_size, scale_factor);
            let line_gap = font.line_gap(style.font_size, scale_factor);
            let (widths, glyphs) = font.get_glyphs(line, style.font_size, scale_factor, env);

            res.push(TextSpan::Text {
                style: Some(style.clone()),
                text: line.to_string(),
                glyphs,
                widths,
                ascend,
                descend,
                line_gap,
            });
            res.push(TextSpan::NewLine);
        }

        res.pop();
        res
    }
}