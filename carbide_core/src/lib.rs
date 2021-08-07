//! # carbide
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to carbide, we recommend checking out [The Guide](./guide/index.html).

#![deny(unsafe_code)] //Todo deny when unsafe code removed from foreach
//#![feature(associated_type_bounds)]
//#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]

#[macro_use]
extern crate carbide_derive;
extern crate self as carbide_core;

pub use serde::*;
pub use serde::de::*;

pub use carbide_derive::*;

pub use crate::color::{Color, Colorable};
pub use crate::position::{OldRect, Point, Range, Scalar};
pub use crate::ui::Ui;

pub mod color;
pub mod event;
pub mod image_map;
pub mod mesh;
pub mod position;
pub mod render;
mod ui;
pub mod utils;
pub mod widget;
pub mod cursor;
pub mod layout;
pub mod state;
pub mod flags;
pub mod draw;
pub mod window;
pub mod prelude;
pub mod focus;
mod environment;
pub mod text;

