use crate::Color;
use crate::draw::{Dimension, Position, Rect, Scalar};
use crate::environment::Environment;
use crate::text::Glyph;
use crate::text::text_decoration::TextDecoration;
use crate::text::text_span::TextSpan;
use crate::text::text_span_generator::TextSpanGenerator;
use crate::text::text_style::TextStyle;
use crate::widget::Justify;
use crate::widget::Wrap;

type BoundingBox = Rect;

#[derive(Debug, Clone)]
pub struct Text {
    latest_requested_offset: Position,
    latest_requested_size: Dimension,
    spans: Vec<TextSpan>,
    latest_max_width: Scalar,
    latest_max_height: Scalar,
    scale_factor: Scalar,
    /// Wrapping mode
    pub wrap: Wrap,
    /// Justify the text
    pub justify: Justify,
    /// True if we moved the glyphs. We then need to add the glyphs to the image again.
    needs_to_update_atlas: bool,
    /// True if the glyphs have already been added. We might need to remove them if we
    /// need to update the atlas.
    already_added_to_atlas: bool,

    string_that_generated_this: String,
    style_that_generated_this: TextStyle,
}

impl Text {
    pub fn new(
        string: String,
        style: TextStyle,
        generator: &dyn TextSpanGenerator,
        env: &mut Environment,
    ) -> Text {
        let mut spans = generator.generate(&string, &style, env);

        Text {
            latest_requested_offset: Default::default(),
            latest_requested_size: Dimension::new(-1.0, -1.0),
            spans,
            latest_max_width: 0.0,
            latest_max_height: 0.0,
            scale_factor: 0.0,
            wrap: Wrap::Whitespace,
            justify: Justify::Left,
            needs_to_update_atlas: true,
            already_added_to_atlas: false,

            string_that_generated_this: string,
            style_that_generated_this: style,
        }
    }

    pub fn string_that_generated_this(&self) -> &String {
        &self.string_that_generated_this
    }

    pub fn first_glyphs(&self) -> Vec<Glyph> {
        if let Some(TextSpan::Text { glyphs, .. }) = self.spans.first() {
            glyphs.to_vec()
        } else {
            vec![]
        }
    }

    pub fn span_glyphs(&self) -> Vec<(Vec<Glyph>, Option<Color>, Vec<Rect>)> {
        self.spans
            .iter()
            .filter_map(|a| match a {
                TextSpan::Text { style, glyphs, .. } => {
                    let style = style.clone().unwrap();
                    Some((
                        glyphs.to_vec(),
                        style.color,
                        style.text_decoration.get_rects(),
                    ))
                }
                TextSpan::Widget(_) => None,
                TextSpan::NewLine => None,
            })
            .collect()
    }

    pub fn position(&mut self, requested_offset: Position) {
        if self.latest_requested_offset != requested_offset {
            // Todo: remove_glyphs_from_atlas
            let new_offset = (requested_offset - self.latest_requested_offset) * self.scale_factor;

            self.latest_requested_offset = requested_offset;
            for span in &mut self.spans {
                match span {
                    TextSpan::Text { glyphs, style, .. } => {
                        for glyph in glyphs {
                            glyph.set_position(glyph.position() + new_offset);
                        }
                        if let Some(style) = style {
                            match &mut style.text_decoration {
                                TextDecoration::None => {}
                                TextDecoration::Overline(r)
                                | TextDecoration::Underline(r)
                                | TextDecoration::StrikeThrough(r) => {
                                    for rect in r {
                                        rect.position.x += new_offset.x / self.scale_factor;
                                        rect.position.y += new_offset.y / self.scale_factor;
                                    }
                                }
                            }
                        }
                    }
                    TextSpan::Widget(_) => {}
                    TextSpan::NewLine => {}
                }
            }
            self.needs_to_update_atlas = true;
            // Todo: add_glyphs_to_atlas
        }
    }

    /// Layout the text within a bounding box and return the dimensions of the resulting layout.
    pub fn calculate_size(&mut self, requested_size: Dimension, env: &Environment) -> Dimension {
        // Layout as if the layout is at x:0, y:0

        // Todo: If text is NoWrap, this is not needed.
        if self.latest_requested_size.width != requested_size.width {
            // Todo: remove_glyphs_from_atlas
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
            // Todo: add_glyphs_to_atlas
            self.needs_to_update_atlas = true;
        }

        Dimension::new(
            self.latest_max_width / self.scale_factor as f64,
            self.latest_max_height / self.scale_factor as f64,
        )
    }

    pub fn ensure_glyphs_added_to_atlas(&mut self, env: &mut Environment) {
        if self.needs_to_update_atlas {
            if self.already_added_to_atlas {
                //env.remove_glyphs_from_atlas()
            }
            let glyphs_to_queue = self
                .spans
                .iter_mut()
                .filter_map(|span| match span {
                    TextSpan::Text { glyphs, .. } => Some(glyphs),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<&mut Glyph>>();

            env.add_glyphs_to_atlas(glyphs_to_queue);
            self.needs_to_update_atlas = false;
            self.already_added_to_atlas = true;
        }
    }

    // Todo: add underline, strikethrough, and fix when that is char wraps for the first word in the span even when its not the first span.
    fn calculate_size_with_word_break(&mut self, requested_size: Dimension, env: &Environment) {
        self.scale_factor = env.get_scale_factor();
        let requested_width = requested_size.width * self.scale_factor;
        let mut max_width = 0.0;
        let mut current_x = 0.0;
        let mut current_line = 0.0;
        let mut latest_break_glyph_index = None;

        for current_span in &mut self.spans {
            match current_span {
                TextSpan::Text {
                    text,
                    widths,
                    glyphs,
                    style,
                    ascend,
                    ..
                } => {
                    let mut current_glyph_index = 0;

                    // Initiate strike lines
                    let mut strike_lines = vec![];
                    let mut current_strike_line = Rect {
                        position: Position::new(current_x / self.scale_factor, current_line),
                        dimension: Default::default(),
                    };

                    // Get all the chars from the text
                    let current_chars = text.chars().collect::<Vec<char>>();

                    // While we have not layout all the glyphs
                    while current_glyph_index < glyphs.len() {
                        let current_width = widths[current_glyph_index];
                        let current_glyph = &mut glyphs[current_glyph_index];
                        let current_char = current_chars[current_glyph_index];

                        current_glyph.set_position(Position::new(current_x, current_line));

                        // If our current char is a whitespace
                        if current_char.is_whitespace() {
                            // If the space is not the first glyph in a line.
                            // This eliminates spaces at the front of text.
                            if current_x != 0.0 {
                                // We mark the glyph as a potential soft break point
                                if let Some(last) = latest_break_glyph_index {
                                    // If we have multiple consecutive whitespaces, only mark the first as a potential break point
                                    if last + 1 != current_glyph_index || current_glyph_index == 1 {
                                        latest_break_glyph_index = Some(current_glyph_index);
                                    }
                                } else {
                                    latest_break_glyph_index = Some(current_glyph_index);
                                }

                                current_x += current_width;
                            }
                        } else {
                            // All other glyphs we position them, and add their width to the current line.
                            current_x += current_width;
                        }

                        // If the new width is larger than the requested_width, we need to wrap in some way.
                        if current_x > requested_width {
                            if let Some(latest_break) = latest_break_glyph_index {
                                let mut current_max_width = current_x;
                                current_line += 1.0;
                                current_x = 0.0;
                                for i in latest_break..=current_glyph_index {
                                    current_max_width -= widths[i];
                                }

                                current_strike_line.dimension = Dimension::new(
                                    current_max_width / self.scale_factor
                                        - current_strike_line.position.x,
                                    1.0,
                                );
                                strike_lines.push(current_strike_line);

                                current_strike_line = Rect {
                                    position: Position::new(
                                        current_x / self.scale_factor,
                                        current_line,
                                    ),
                                    dimension: Default::default(),
                                };

                                if current_max_width > max_width {
                                    max_width = current_max_width;
                                }
                                current_glyph_index = latest_break;
                            } else {
                                current_strike_line.dimension = Dimension::new(
                                    requested_width / self.scale_factor
                                        - current_strike_line.position.x,
                                    1.0,
                                );
                                strike_lines.push(current_strike_line);
                                current_line += 1.0;
                                current_x = 0.0;

                                current_strike_line = Rect {
                                    position: Position::new(
                                        current_x / self.scale_factor,
                                        current_line,
                                    ),
                                    dimension: Default::default(),
                                };

                                max_width = requested_width;
                                current_glyph.set_position(Position::new(current_x, current_line));
                                current_x += current_width;
                                current_glyph_index += 1;
                            }

                            latest_break_glyph_index = None;
                        } else {
                            current_glyph_index += 1;
                        }
                    }

                    if current_x > max_width {
                        max_width = current_x;
                    }

                    current_strike_line.dimension = Dimension::new(
                        current_x / self.scale_factor - current_strike_line.position.x,
                        1.0,
                    );
                    strike_lines.push(current_strike_line);

                    if let Some(style) = style {
                        match &mut style.text_decoration {
                            TextDecoration::None => {}
                            TextDecoration::StrikeThrough(l) => {
                                *l = strike_lines;
                            }
                            TextDecoration::Overline(l) => {
                                *l = strike_lines;
                            }
                            TextDecoration::Underline(l) => {
                                *l = strike_lines;
                            }
                        }
                    }

                    latest_break_glyph_index = Some(0);
                }
                TextSpan::Widget(_) => {}
                TextSpan::NewLine => {
                    current_x = 0.0;
                    current_line += 1.0; // 1.0
                }
            }
        }

        self.calculate_line_heights(requested_size, env);

        self.latest_requested_offset = Position::new(0.0, 0.0);
        self.latest_max_width = max_width as f64;
    }

    fn calculate_size_with_character_break(
        &mut self,
        requested_size: Dimension,
        env: &Environment,
    ) {
        self.scale_factor = env.get_scale_factor();
        let width = requested_size.width * self.scale_factor;
        let mut max_width = 0.0;
        let mut current_x = 0.0;
        let mut current_line = 0.0;

        // Flatten the spans into lines by layout the widths and x axis.
        for span in &mut self.spans {
            match span {
                TextSpan::Text {
                    widths,
                    glyphs,
                    style,
                    ascend: ascending_pixels,
                    ..
                } => {
                    // Initiate strike lines
                    let mut strike_lines = vec![];
                    let mut current_strike_line = Rect {
                        position: Position::new(
                            current_x / self.scale_factor,
                            current_line / self.scale_factor,
                        ),
                        dimension: Default::default(),
                    };

                    for (glyph, w) in glyphs.iter_mut().zip(widths) {
                        glyph.set_position(Position::new(current_x, current_line));
                        current_x += *w;

                        if current_x > width {
                            current_strike_line.dimension = Dimension::new(
                                width / self.scale_factor - current_strike_line.position.x,
                                1.0,
                            );
                            strike_lines.push(current_strike_line);

                            current_line += 1.0;
                            current_x = 0.0;
                            max_width = width;

                            current_strike_line = Rect {
                                position: Position::new(
                                    current_x / self.scale_factor,
                                    current_line / self.scale_factor,
                                ),
                                dimension: Default::default(),
                            };

                            glyph.set_position(Position::new(current_x, current_line));
                            current_x += *w;
                        }

                        if current_x > max_width {
                            max_width = current_x;
                        }
                    }

                    current_strike_line.dimension = Dimension::new(
                        current_x / self.scale_factor - current_strike_line.position.x,
                        1.0,
                    );
                    strike_lines.push(current_strike_line);

                    if let Some(style) = style {
                        match &mut style.text_decoration {
                            TextDecoration::None => {}
                            TextDecoration::StrikeThrough(l) => {
                                *l = strike_lines;
                            }
                            TextDecoration::Overline(l) => {
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
                    current_line += 1.0; // 1.0
                }
            }
        }

        self.calculate_line_heights(requested_size, env);
        self.latest_requested_offset = Position::new(0.0, 0.0);
        self.latest_max_width = max_width as f64;
    }

    fn calculate_line_heights(&mut self, requested_size: Dimension, env: &Environment) {
        let mut line_descends = vec![0.0];
        let mut line_ascends = vec![];
        let mut line_gaps = vec![];
        let mut line_positions = vec![];
        let mut position = 0.0;
        let mut max_descend_this_line: f64 = 0.0;
        let mut max_ascend_this_line: f64 = 0.0;
        let mut max_line_gap: f64 = 0.0;
        let mut current_line = 0.0;

        for current_span in &mut self.spans {
            match current_span {
                TextSpan::Text {
                    glyphs,
                    ascend,
                    descend,
                    line_gap,
                    ..
                } => {
                    for glyph in glyphs {
                        if current_line == glyph.position().y {
                            max_ascend_this_line = max_ascend_this_line.max(*ascend);
                            max_descend_this_line = max_descend_this_line.max(-(*descend));
                            max_line_gap = max_line_gap.max(*line_gap);
                        } else {
                            let prev_line = current_line as u32;
                            let next_line = glyph.position().y as u32;
                            for _ in prev_line..next_line {
                                current_line = glyph.position().y;
                                line_descends.push(max_descend_this_line);
                                line_ascends.push(max_ascend_this_line);
                                line_gaps.push(max_line_gap);
                                max_descend_this_line = -*descend;
                                max_ascend_this_line = *ascend;
                                max_line_gap = *line_gap;
                            }
                        }
                    }
                }
                TextSpan::Widget(_) => {}
                TextSpan::NewLine => {}
            }
        }

        line_descends.push(max_descend_this_line);
        line_ascends.push(max_ascend_this_line);
        line_ascends.push(0.0);
        line_gaps.push(max_line_gap);
        line_gaps.push(0.0);

        for i in 0..line_ascends.len() {
            position += line_ascends[i];
            position += line_descends[i];
            position += line_gaps[i];
            line_positions.push(position);
        }

        //println!("Line_acends: {:?}", line_ascends);
        //println!("Line_descends: {:?}", line_descends);
        //println!("Line_gaps: {:?}", line_gaps);
        //println!("Line positions: {:?}", line_positions);

        for current_span in &mut self.spans {
            match current_span {
                TextSpan::Text {
                    glyphs,
                    style,
                    ascend,
                    ..
                } => {
                    for glyph in glyphs {
                        let position = glyph.position();
                        glyph.set_position(Position::new(
                            position.x,
                            line_positions[position.y as usize],
                        ));
                    }

                    if let Some(style) = style {
                        match &mut style.text_decoration {
                            TextDecoration::None => {}
                            TextDecoration::StrikeThrough(l) => {
                                for line in l {
                                    let position = line.position;
                                    line.position = Position::new(
                                        position.x,
                                        line_positions[position.y as usize] / self.scale_factor,
                                    );
                                    line.position.y -= *ascend * 0.3 / self.scale_factor;
                                }
                            }
                            TextDecoration::Overline(l) => {
                                for line in l {
                                    let position = line.position;
                                    line.position = Position::new(
                                        position.x,
                                        line_positions[position.y as usize] / self.scale_factor,
                                    );
                                    line.position.y -= *ascend / self.scale_factor;
                                }
                            }
                            TextDecoration::Underline(l) => {
                                for line in l {
                                    let position = line.position;
                                    line.position = Position::new(
                                        position.x,
                                        line_positions[position.y as usize] / self.scale_factor,
                                    );
                                }
                            }
                        }
                    }
                }
                TextSpan::Widget(_) => {}
                TextSpan::NewLine => {}
            }
        }

        self.latest_max_height = position;
    }
}
