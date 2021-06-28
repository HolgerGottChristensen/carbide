//use crate::text::paragraph::Paragraph;
use crate::{Color, OldRect, Point, Scalar};
use crate::color::BLACK;
use crate::draw::{Dimension, Position, Rect};
use crate::text::Font;
use crate::text::font_family::FontFamily;
use crate::text::font_style::FontStyle;
use crate::text::font_weight::FontWeight;
use crate::text::glyph::Glyph;
use crate::text::text_decoration::TextDecoration;
use crate::text::text_style::TextStyle;
use crate::widget::{Environment, GlobalState, Widget};
use crate::widget::types::justify::Justify;
use crate::widget::types::text_wrap::Wrap;

#[derive(Debug, Clone)]
pub enum TextSpan<GS> where GS: GlobalState {
    Text {
        style: Option<TextStyle>,
        text: String,
        glyphs: Vec<Glyph>,
        widths: Vec<Scalar>,
    },
    Widget(Box<dyn Widget<GS>>),
    NewLine,
}

impl<GS: GlobalState> TextSpan<GS> {
    pub fn new(string: &str, env: &mut Environment<GS>) -> Vec<TextSpan<GS>> {
        let default_font_family = FontFamily {
            fonts: 0
        };

        let scale_factor = env.get_scale_factor();

        let mut res = vec![];

        for line in string.split('\n') {
            let style = TextStyle {
                font_family: default_font_family,
                font_size: 14,
                font_style: FontStyle::Normal,
                font_weight: FontWeight::Normal,
                text_decoration: TextDecoration::None,
                color: Color::random(),
            };
            let font = style.get_font(env);

            let (widths, glyphs) = font.get_glyphs(line, style.font_size, scale_factor);

            res.push(TextSpan::Text {
                style: Some(style.clone()),
                text: line.to_string(),
                glyphs,
                widths,
            });
            res.push(TextSpan::NewLine);
        }

        res.pop();
        res
    }
}