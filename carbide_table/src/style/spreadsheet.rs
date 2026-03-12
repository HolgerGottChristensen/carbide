use carbide::color::{ColorExt, BLUE, BROWN, DARK_GREEN, DARK_ORANGE, DARK_PURPLE, DARK_YELLOW, RED, WHITE};
use carbide::draw::{Dimension, DrawStyle, Scalar};
use carbide::state::{LocalState, ReadState, State};
use crate::cell::{Cell, CellSelection, ResizeHandles};
use crate::cell_border::{CellBorder, CellBorders};
use crate::style::TableStyle;

#[derive(Debug, Clone)]
pub struct SpreadsheetStyle {
    pub frozen_columns: LocalState<usize>,
    pub frozen_rows: LocalState<usize>,
    pub hovered_cell: LocalState<CellSelection>,
    pub widths: Vec<f64>,
    pub heights: Vec<f64>,
}

impl TableStyle for SpreadsheetStyle {
    fn widths(&self) -> &[f64] {
        &self.widths
    }

    fn heights(&self) -> &[f64] {
        &self.heights
    }

    fn frozen_rows(&self) -> usize {
        *self.frozen_rows.value()
    }

    fn frozen_columns(&self) -> usize {
        *self.frozen_columns.value()
    }

    fn set_hovered(&mut self, selection: CellSelection) {
        *self.hovered_cell.value_mut() = selection;
    }

    fn resize_cell(&mut self, col: u32, row: u32, dimension: Dimension) {
        self.widths[col as usize] = dimension.width;
        self.heights[row as usize] = dimension.height;
    }

    fn resize_width(&mut self, col: u32, width: Scalar) {
        self.widths[col as usize] = width;
    }

    fn resize_height(&mut self, row: u32, height: Scalar) {
        self.heights[row as usize] = height;
    }

    fn cell(&self, col: u32, row: u32) -> Cell {
        let frozen_columns = self.frozen_columns() as u32;
        let frozen_rows = self.frozen_rows() as u32;

        let base_color = if col < frozen_columns && row < frozen_rows {
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

        let resize_handles = if row == 0 && col == 0 {
            ResizeHandles::None
        } else if row == 3 && col == 3 {
            ResizeHandles::Both
        } else if row == 0 {
            ResizeHandles::Column
        } else if col == 0 {
            ResizeHandles::Row
        } else {
            ResizeHandles::None
        };

        let color = if let CellSelection::Single { row: hovered_row, column: hovered_col } = *self.hovered_cell.value() && hovered_row == row && hovered_col == col {
            base_color.lightened(0.3)
        } else {
            base_color
        };

        Cell {
            draw_style: DrawStyle::Color(color),
            text,
            text_style: Default::default(),
            resize_handles,
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