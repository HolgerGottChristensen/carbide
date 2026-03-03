mod spreadsheet;

use std::fmt::Debug;
use dyn_clone::{clone_trait_object, DynClone};
use carbide::environment::EnvironmentKey;
use crate::cell::Cell;

pub use spreadsheet::*;
use crate::cell_border::CellBorders;

#[derive(Debug, Copy, Clone)]
pub(crate) struct TableStyleKey;

impl EnvironmentKey for TableStyleKey {
    type Value = Box<dyn TableStyle>;
}

pub trait TableStyle: Debug + DynClone + 'static {
    fn frozen_rows(&self) -> usize;
    fn frozen_columns(&self) -> usize;

    fn cell(&self, col: u32, row: u32) -> Cell;
    fn cell_border(&self, col: u32, row: u32) -> CellBorders;
}

clone_trait_object!(TableStyle);