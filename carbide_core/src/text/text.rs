use rusttype::{point, vector};

use crate::{Color, Scalar};
use crate::draw::{Dimension, Position, Rect};
use crate::text::{Font, FontId, Glyph};
use crate::text::text_decoration::TextDecoration;
use crate::text::text_span::TextSpan;
use crate::text::text_style::TextStyle;
use crate::widget::{Environment, GlobalState};
use crate::widget::types::justify::Justify;
use crate::widget::types::text_wrap::Wrap;

type BoundingBox = Rect;

#[derive(Debug, Clone)]
pub struct Text<GS> where GS: GlobalState {
    style: Option<TextStyle>,
    latest_requested_offset: Position,
    latest_requested_size: Dimension,
    spans: Vec<TextSpan<GS>>,
    latest_max_width: Scalar,
    scale_factor: Scalar,
    /// Wrapping mode
    pub wrap: Wrap,
    /// Justify the text
    pub justify: Justify,
}

impl<GS: GlobalState> Text<GS> {
    pub fn new(string: String, env: &mut Environment<GS>) -> Text<GS> {
        let spans = TextSpan::new_polar_bear_markup(&string, env);

        Text {
            style: None,
            latest_requested_offset: Default::default(),
            latest_requested_size: Dimension::new(-1.0, -1.0),
            spans,
            latest_max_width: 0.0,
            scale_factor: 0.0,
            wrap: Wrap::Character,
            justify: Justify::Left,
        }
    }

    pub fn first_glyphs(&self) -> Vec<Glyph> {
        if let Some(TextSpan::Text { glyphs, .. }) = self.spans.first() {
            glyphs.to_vec()
        } else {
            vec![]
        }
    }

    pub fn span_glyphs(&self) -> Vec<(Vec<Glyph>, FontId, Option<Color>, Vec<Rect>)> {
        self.spans.iter().filter_map(|a| {
            match a {
                TextSpan::Text { font_id, style, glyphs, .. } => {
                    let style = style.clone().unwrap();
                    Some((glyphs.to_vec(), *font_id, style.color, style.text_decoration.get_rects()))
                }
                TextSpan::Widget(_) => None,
                TextSpan::NewLine => None,
            }
        }).collect()
    }

    pub fn position(&mut self, requested_offset: Position) {
        if self.latest_requested_offset != requested_offset {
            let new_offset = requested_offset - self.latest_requested_offset;

            self.latest_requested_offset = requested_offset;
            let offset = vector(new_offset.x as f32 * self.scale_factor as f32, new_offset.y as f32 * self.scale_factor as f32);
            for span in &mut self.spans {
                match span {
                    TextSpan::Text { glyphs, style, .. } => {
                        for glyph in glyphs {
                            let new_position = glyph.position() + offset;
                            glyph.set_position(new_position);
                        }
                        if let Some(style) = style {
                            match &mut style.text_decoration {
                                TextDecoration::None => {}
                                TextDecoration::Overline(r) |
                                TextDecoration::Underline(r) |
                                TextDecoration::StrikeThrough(r) => {
                                    for rect in r {
                                        rect.position.x += new_offset.x;
                                        rect.position.y += new_offset.y;
                                    }
                                }
                            }
                        }
                    }
                    TextSpan::Widget(_) => {}
                    TextSpan::NewLine => {}
                }
            }
        }
    }

    /// Layout the text within a bounding box and return the dimensions of the resulting layout.
    pub fn calculate_size(&mut self, requested_size: Dimension, env: &Environment<GS>) -> Dimension {
        // Layout as if the layout is at x:0, y:0

        // Todo: If text is NoWrap, this is not needed.
        if self.latest_requested_size.width != requested_size.width {
            self.latest_requested_size = requested_size;
            match self.wrap {
                Wrap::Character => {
                    self.calculate_size_with_character_break(requested_size, env);
                }
                Wrap::Whitespace => {
                    self.calculate_size_with_word_break(requested_size, env);
                }
                Wrap::None => {}
            }
        }

        Dimension::new(self.latest_max_width / self.scale_factor as f64, 100.0)
    }

    fn calculate_size_with_word_break(&mut self, requested_size: Dimension, env: &Environment<GS>) {
        self.scale_factor = env.get_scale_factor();
        let width = requested_size.width as f32 * self.scale_factor as f32;
        let mut max_width = 0.0;
        let mut current_x = 0.0;
        let mut current_line = 0.0;

        for current_span in &mut self.spans {
            match current_span {
                TextSpan::Text { text, widths, glyphs, .. } => {
                    let mut current_glyph_index = 0;
                    let mut latest_break_glyph_index = None;
                    let current_chars = text.chars().collect::<Vec<char>>();

                    while current_glyph_index < glyphs.len() {
                        let current_width = widths[current_glyph_index];
                        let current_glyph = &mut glyphs[current_glyph_index];
                        let current_char = current_chars[current_glyph_index];
                        if current_char.is_whitespace() {
                            if current_x != 0.0 {
                                latest_break_glyph_index = Some(current_glyph_index);
                                current_glyph.set_position(point(current_x, current_line));
                                current_x += current_width as f32;
                            }
                        } else {
                            current_glyph.set_position(point(current_x, current_line));
                            current_x += current_width as f32;
                        }

                        if current_x > width {
                            if current_char.is_whitespace() && current_x != 0.0 {
                                current_x -= current_width as f32;
                                if current_x > max_width {
                                    max_width = current_x;
                                }
                                current_line += 28.0; // 1.0
                                current_x = 0.0;
                            } else {
                                if let Some(latest_break) = latest_break_glyph_index {
                                    let mut current_max_width = current_x;
                                    current_line += 28.0; // 1.0
                                    current_x = 0.0;
                                    for i in latest_break..current_glyph_index {
                                        current_max_width -= widths[i] as f32;
                                    }
                                    if current_max_width > max_width {
                                        max_width = current_max_width;
                                    }
                                    current_glyph_index = latest_break;
                                } else {
                                    current_line += 28.0; // 1.0
                                    current_x = 0.0;
                                    max_width = width;
                                    current_glyph.set_position(point(current_x, current_line));
                                    current_x += current_width as f32;
                                    current_glyph_index += 1;
                                }
                            }
                            latest_break_glyph_index = None;
                        } else {
                            current_glyph_index += 1;
                        }
                    }

                    if current_x > max_width {
                        max_width = current_x;
                    }
                }
                TextSpan::Widget(_) => {}
                TextSpan::NewLine => {
                    current_x = 0.0;
                    current_line += 28.0; // 1.0
                }
            }
        }

        self.latest_requested_offset = Position::new(0.0, 0.0);
        self.latest_max_width = max_width as f64;
    }

    fn calculate_size_with_character_break(&mut self, requested_size: Dimension, env: &Environment<GS>) {
        self.scale_factor = env.get_scale_factor();
        let width = requested_size.width as f32 * self.scale_factor as f32;
        let mut max_width = 0.0;
        let mut current_x = 0.0;
        let mut current_line = 28.0;//0.0;

        // Flatten the spans into lines by layout the widths and x axis.
        for span in &mut self.spans {
            match span {
                TextSpan::Text { widths, glyphs, style, ascending_pixels, .. } => {
                    // Initiate strike lines
                    let mut strike_lines = vec![];
                    let mut current_strike_line = Rect { position: Position::new(current_x as f64 / self.scale_factor, current_line as f64 / self.scale_factor), dimension: Default::default() };

                    for (glyph, w) in glyphs.iter_mut().zip(widths) {
                        glyph.set_position(point(current_x, current_line));
                        current_x += *w as f32;

                        if current_x > width {
                            current_strike_line.dimension = Dimension::new((width as f64 / self.scale_factor - current_strike_line.position.x), 1.0);
                            strike_lines.push(current_strike_line);

                            current_line += 28.0; // 1.0
                            current_x = 0.0;
                            max_width = width;

                            current_strike_line = Rect { position: Position::new(current_x as f64 / self.scale_factor, current_line as f64 / self.scale_factor), dimension: Default::default() };

                            glyph.set_position(point(current_x, current_line));
                            current_x += *w as f32;
                        }

                        if current_x > max_width {
                            max_width = current_x;
                        }
                    }

                    current_strike_line.dimension = Dimension::new(current_x as f64 / self.scale_factor - current_strike_line.position.x, 1.0);
                    strike_lines.push(current_strike_line);

                    if let Some(style) = style {
                        match &mut style.text_decoration {
                            TextDecoration::None => {}
                            TextDecoration::StrikeThrough(l) => {
                                for line in &mut strike_lines {
                                    line.position.y -= *ascending_pixels * 0.3;
                                }
                                *l = strike_lines;
                            }
                            TextDecoration::Overline(l) => {
                                for line in &mut strike_lines {
                                    line.position.y -= *ascending_pixels;
                                }
                                *l = strike_lines;
                            }
                            TextDecoration::Underline(l) => {
                                *l = strike_lines;
                            }
                        }
                    }
                }
                TextSpan::Widget(widget) => {}
                TextSpan::NewLine => {
                    current_x = 0.0;
                    current_line += 28.0; // 1.0
                }
            }
        }

        self.latest_requested_offset = Position::new(0.0, 0.0);
        self.latest_max_width = max_width as f64;
    }
}