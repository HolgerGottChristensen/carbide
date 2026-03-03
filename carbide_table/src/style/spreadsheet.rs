use carbide::color::{BLUE, BROWN, DARK_GREEN, DARK_ORANGE, DARK_PURPLE, DARK_YELLOW, RED, WHITE};
use carbide::draw::DrawStyle;
use carbide::state::{LocalState, ReadState};
use crate::cell::Cell;
use crate::cell_border::{CellBorder, CellBorders};
use crate::style::TableStyle;

#[derive(Debug, Clone)]
pub struct SpreadsheetStyle {
    pub frozen_columns: LocalState<usize>,
    pub frozen_rows: LocalState<usize>,
}

impl TableStyle for SpreadsheetStyle {
    fn frozen_rows(&self) -> usize {
        *self.frozen_rows.value()
    }

    fn frozen_columns(&self) -> usize {
        *self.frozen_columns.value()
    }

    fn cell(&self, col: u32, row: u32) -> Cell {
        let frozen_columns = self.frozen_columns() as u32;
        let frozen_rows = self.frozen_rows() as u32;

        let color = if col < frozen_columns && row < frozen_rows {
            if (row + col) % 2 == 0 { DARK_PURPLE } else { DARK_ORANGE }
        } else if col < frozen_columns || row < frozen_rows {
            if (row + col) % 2 == 0 { DARK_YELLOW } else { DARK_GREEN }
        } else {
            if (row + col) % 2 == 0 { RED } else { BLUE }
        };

        let text = if col < frozen_columns && row < frozen_rows {
            None
        } else {
            Some(format!("R{}C{}", row, col))
        };

        Cell {
            draw_style: DrawStyle::Color(color),
            text,
            text_style: Default::default(),
        }
    }

    fn cell_border(&self, col: u32, row: u32) -> CellBorders {
        CellBorders {
            top: CellBorder {
                style: DrawStyle::Color(WHITE),
                width: 1.0
            },
            bottom: CellBorder {
                style: DrawStyle::Color(WHITE),
                width: 1.0
            },
            left: CellBorder {
                style: DrawStyle::Color(WHITE),
                width: 1.0
            },
            right: CellBorder {
                style: DrawStyle::Color(WHITE),
                width: 1.0
            },
        }
    }
}