use carbide_core::draw::Position;
use carbide_core::Scalar;
use carbide_core::text::Glyph;
use carbide_core::utils::binary_search;

#[derive(Debug, Clone, Copy)]
pub enum Cursor {
    Single(CursorIndex),
    Selection { start: CursorIndex, end: CursorIndex },
}

impl Cursor {
    pub fn width(&self, text: &str, glyphs: &Vec<Glyph>) -> Scalar {
        if let Cursor::Selection { start, end } = self {
            let start_point = start.position(text, glyphs);
            let end_point = end.position(text, glyphs);
            end_point.x() - start_point.x()
        } else {
            0.0
        }
    }

    /*pub fn get_char_index_split_points(positioned_glyphs: &Vec<Glyph>) -> Vec<f32> {
        let splits = vec![0.0].into_iter().chain(positioned_glyphs.iter().map(|val| {
            let middle = val.position().x + val.unpositioned().h_metrics().advance_width;
            middle
        }));

        let collected: Vec<f32> = splits.collect();
        collected
    }*/

    pub fn char_index(relative_offset: f64, glyphs: &Vec<Glyph>) -> usize {
        let splits = vec![0.0].into_iter().chain(glyphs.iter().map(|glyph| {
            let middle = glyph.position().x() + glyph.advance_width();
            middle as f32
        }));
        let splits = splits.collect::<Vec<_>>();
        let rightmost_closest = binary_search(relative_offset as f32, &splits);

        let new_closest = if rightmost_closest < splits.len() - 1 && ((relative_offset as f32) - splits[rightmost_closest + 1]).abs() < ((relative_offset as f32) - splits[rightmost_closest]).abs() {
            rightmost_closest + 1
        } else {
            rightmost_closest
        };

        new_closest
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CursorIndex {
    /// The index of the line upon which the cursor is situated.
    pub line: usize,
    /// The index within all possible cursor positions for the line.
    ///
    /// For example, for the line `foo`, a `char` of `1` would indicate the cursor's position
    /// as `f|oo` where `|` is the cursor.
    pub char: usize,
}


impl CursorIndex {
    /// Get the position of the cursor, based on the glyphs. Index 0 is before all the text, and
    /// the cursor can be in range 0..glyphs.len()+1
    pub fn position(&self, text: &str, glyphs: &Vec<Glyph>) -> Position {
        if self.line == 0 {
            if self.char == 0 {
                return Position::new(0.0, 0.0);
            }
            if self.char <= glyphs.len() {
                let positioned = &glyphs[self.char - 1];

                let point = positioned.position().translate_x(positioned.advance_width());

                point
            } else {
                panic!("The char index is outside of the letters({}): {} > {}", text, self.char, glyphs.len() + 1)
            }
        } else {
            panic!("For now only operate on single line things")
        }
    }
}
