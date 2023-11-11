use carbide_core::draw::Scalar;
use carbide_core::environment::Environment;
use carbide_core::widget::AnyWidget;
use crate::glyph::Glyph;
use crate::text_context::TextContext;
use crate::text_style::TextStyle;

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
    Widget(Box<dyn AnyWidget>),
    NewLine,
}

impl TextSpan {
    pub fn new(string: &str, style: &TextStyle, context: &mut TextContext, scale_factor: f64) -> Vec<TextSpan> {
        let mut res = vec![];

        for line in string.split('\n') {
            let font = style.get_font(context);

            let ascend = font.ascend(style.font_size, scale_factor);
            let descend = font.descend(style.font_size, scale_factor);
            let line_gap = font.line_gap(style.font_size, scale_factor);
            let (widths, glyphs) = font.glyphs_for(line, style.font_size, scale_factor, context);

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
