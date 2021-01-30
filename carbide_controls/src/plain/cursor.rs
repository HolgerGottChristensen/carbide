use carbide_core::{text, Point};
use carbide_core::text::PositionedGlyph;

#[derive(Debug, Clone)]
pub enum Cursor {
    Single(CursorIndex),
    Selection {start: CursorIndex, end: CursorIndex}
}

#[derive(Debug, Clone)]
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

    pub fn get_position(&self, positioned_glyphs: Vec<PositionedGlyph>) -> Point {
        if self.line == 0 {
            if self.char < positioned_glyphs.len() {
                let positioned = &positioned_glyphs[self.char];

                let point = positioned.position();

                let width = positioned.unpositioned().h_metrics().advance_width;

                [point.x as f64 + width as f64, point.y as f64]

            } else {
                panic!("The char index is outside of the letters: {} >= {}", self.char, positioned_glyphs.len()-1)
            }
        } else {
            panic!("For now only operate on single line things")
        }
    }


    // /// The cursor index of the beginning of the word (block of non-whitespace) before `self`.
    // ///
    // /// If `self` is at the beginning of the line, call previous, which returns the last
    // /// index position of the previous line, or None if it's the first line
    // ///
    // /// If `self` points to whitespace, skip past that whitespace, then return the index of
    // /// the start of the word that precedes the whitespace
    // ///
    // /// If `self` is in the middle or end of a word, return the index of the start of that word
    // pub fn previous_word_start<I>(self, text: &str, mut line_infos: I) -> Option<Self>
    //     where I: Iterator<Item=super::line::Info>,
    // {
    //     let CursorIndex { line, char } = self;
    //     if char > 0 {
    //         line_infos.nth(line).and_then(|line_info| {
    //             let line_count = line_info.char_range().count();
    //             let mut chars_rev = (&text[line_info.byte_range()]).chars().rev();
    //             if char != line_count {
    //                 chars_rev.nth(line_count - char - 1);
    //             }
    //             let mut new_char = 0;
    //             let mut hit_non_whitespace = false;
    //             for (i, char_) in chars_rev.enumerate() {
    //                 // loop until word starts, then continue until the word ends
    //                 if !char_.is_whitespace() { hit_non_whitespace = true; }
    //                 if char_.is_whitespace() && hit_non_whitespace {
    //                     new_char = char - i;
    //                     break
    //                 }
    //             }
    //             Some(CursorIndex { line, char: new_char })
    //         })
    //     } else {
    //         self.previous(line_infos)
    //     }
    // }
    //
    // /// The cursor index of the end of the first word (block of non-whitespace) after `self`.
    // ///
    // /// If `self` is at the end of the text, this returns `None`.
    // ///
    // /// If `self` is at the end of a line other than the last, this returns the first index of
    // /// the next line.
    // ///
    // /// If `self` points to whitespace, skip past that whitespace, then return the index of
    // /// the end of the word after the whitespace
    // ///
    // /// If `self` is in the middle or start of a word, return the index of the end of that word
    // pub fn next_word_end<I>(self, text: &str, mut line_infos: I) -> Option<Self>
    //     where I: Iterator<Item=super::line::Info>,
    // {
    //     let CursorIndex { line, char } = self;
    //     line_infos.nth(line)
    //         .and_then(|line_info| {
    //             let line_count = line_info.char_range().count();
    //             if char < line_count {
    //                 let mut chars = (&text[line_info.byte_range()]).chars();
    //                 let mut new_char = line_count;
    //                 let mut hit_non_whitespace = false;
    //                 if char != 0 {
    //                     chars.nth(char - 1);
    //                 }
    //                 for (i, char_) in chars.enumerate() {
    //                     // loop until word starts, then continue until the word ends
    //                     if !char_.is_whitespace() { hit_non_whitespace = true; }
    //                     if char_.is_whitespace() && hit_non_whitespace {
    //                         new_char = char + i;
    //                         break
    //                     }
    //                 }
    //                 Some(CursorIndex { line: line, char: new_char })
    //             } else {
    //                 line_infos.next().map(|_| CursorIndex { line: line + 1, char: 0 })
    //             }
    //         })
    // }
    //
    // /// The cursor index that comes before `self`.
    // ///
    // /// If `self` is at the beginning of the text, this returns `None`.
    // ///
    // /// If `self` is at the beginning of a line other than the first, this returns the last
    // /// index position of the previous line.
    // ///
    // /// If `self` is a position other than the start of a line, it will return the position
    // /// that is immediately to the left.
    // pub fn previous<I>(self, mut line_infos: I) -> Option<Self>
    //     where I: Iterator<Item=super::line::Info>,
    // {
    //     let CursorIndex { line, char } = self;
    //     if char > 0 {
    //         let new_char = char - 1;
    //         line_infos.nth(line)
    //             .and_then(|info| if new_char <= info.char_range().count() {
    //                 Some(CursorIndex { line: line, char: new_char })
    //             } else {
    //                 None
    //             })
    //     } else if line > 0 {
    //         let new_line = line - 1;
    //         line_infos.nth(new_line)
    //             .map(|info| {
    //                 let new_char = info.end_char() - info.start_char;
    //                 CursorIndex { line: new_line, char: new_char }
    //             })
    //     } else {
    //         None
    //     }
    // }
    //
    // /// The cursor index that follows `self`.
    // ///
    // /// If `self` is at the end of the text, this returns `None`.
    // ///
    // /// If `self` is at the end of a line other than the last, this returns the first index of
    // /// the next line.
    // ///
    // /// If `self` is a position other than the end of a line, it will return the position that
    // /// is immediately to the right.
    // pub fn next<I>(self, mut line_infos: I) -> Option<Self>
    //     where I: Iterator<Item=super::line::Info>,
    // {
    //     let CursorIndex { line, char } = self;
    //     line_infos.nth(line)
    //         .and_then(|info| {
    //             if char >= info.char_range().count() {
    //                 line_infos.next().map(|_| CursorIndex { line: line + 1, char: 0 })
    //             } else {
    //                 Some(CursorIndex { line: line, char: char + 1 })
    //             }
    //         })
    // }
    //
    // /// Clamps `self` to the given lines.
    // ///
    // /// If `self` would lie after the end of the last line, return the index at the end of the
    // /// last line.
    // ///
    // /// If `line_infos` is empty, returns cursor at line=0 char=0.
    // pub fn clamp_to_lines<I>(self, line_infos: I) -> Self
    //     where I: Iterator<Item=super::line::Info>,
    // {
    //     let mut last = None;
    //     for (i, info) in line_infos.enumerate() {
    //         if i == self.line {
    //             let num_chars = info.char_range().len();
    //             let char = std::cmp::min(self.char, num_chars);
    //             return CursorIndex { line: i, char: char };
    //         }
    //         last = Some((i, info));
    //     }
    //     match last {
    //         Some((i, info)) => CursorIndex {
    //             line: i,
    //             char: info.char_range().len(),
    //         },
    //         None => CursorIndex { line: 0, char: 0 },
    //     }
    // }

}