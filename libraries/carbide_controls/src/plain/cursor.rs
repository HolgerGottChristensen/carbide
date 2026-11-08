
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    Single(CursorIndex),
    Selection {
        start: CursorIndex,
        end: CursorIndex,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorIndex {
    /// The index of the line upon which the cursor is situated.
    pub line: usize,

    /// The grapheme index. 0 means the cursor is at the start of the string.
    pub index: usize,
}
