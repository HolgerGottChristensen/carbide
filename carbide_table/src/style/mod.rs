mod spreadsheet;

use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::draw::{Dimension, DrawOptions, DrawShape, Position, Rect, Scalar};
use carbide::draw::fill::FillOptions;
use carbide::environment::EnvironmentKey;
use carbide::render::RenderContext;
use crate::cell::{Cell, CellSelection};

pub use spreadsheet::*;
use crate::cell_border::CellBorders;

#[derive(Debug, Copy, Clone)]
pub(crate) struct TableStyleKey;

impl EnvironmentKey for TableStyleKey {
    type Value = Box<dyn TableStyle>;
}

pub trait TableStyle: Debug + DynClone + 'static {
    fn widths(&self) -> &[f64];
    fn heights(&self) -> &[f64];

    fn frozen_rows(&self) -> usize;
    fn frozen_columns(&self) -> usize;

    fn set_hovered(&mut self, selection: CellSelection);

    fn resize_cell(&mut self, col: u32, row: u32, dimension: Dimension);
    fn resize_width(&mut self, col: u32, width: Scalar);
    fn resize_height(&mut self, row: u32, height: Scalar);

    fn cell(&self, col: u32, row: u32) -> Cell;
    fn cell_border(&self, col: u32, row: u32) -> CellBorders;

    fn draw_cell(&self, col: u32, row: u32, rect: Rect, ctx: &mut RenderContext) {
        let cell = self.cell(col, row);

        ctx.style(cell.draw_style, |ctx| {
            ctx.shape(
                DrawShape::Rectangle(rect),
                DrawOptions::Fill(FillOptions::default()),
            );
        });

        if let Some(text) = &cell.text {
            let text_dimensions = ctx.measure_text(text, &cell.text_style, None);

            let position = Position::new(
                rect.position.x + rect.dimension.width / 2.0 - text_dimensions.width / 2.0,
                rect.position.y + rect.dimension.height / 2.0 - text_dimensions.height / 2.0
            );

            ctx.text(text, &cell.text_style, position, None);
        }
    }
}

clone_trait_object!(TableStyle);