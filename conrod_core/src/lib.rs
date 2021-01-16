//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to Conrod, we recommend checking out [The Guide](./guide/index.html).

#![warn(unsafe_code)] //Todo deny when unsafe code removed from foreach
//#![feature(associated_type_bounds)]
//#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate bytemuck;
#[macro_use]
extern crate conrod_derive;
extern crate copypasta;
extern crate core;
extern crate daggy;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate dyn_clone;
extern crate fnv;
extern crate input as piston_input;
extern crate instant;
extern crate num;
extern crate ron;
extern crate rusttype;
extern crate serde;
extern crate uuid;
extern crate wgpu;

extern crate self as conrod_core;

pub use ron::from_str as from_ron;
pub use ron::to_string as to_ron;

pub use conrod_derive::*;

pub use crate::border::{Borderable, Bordering};
pub use crate::color::{Color, Colorable};
pub use crate::label::{FontSize, Labelable};
pub use crate::position::{Dimension, Point, Position, Positionable, Range, Rect, Scalar, Sizeable};
pub use crate::theme::Theme;
pub use crate::ui::{Ui, UiBuilder, UiCell};

mod border;
pub mod color;
pub mod event;
pub mod graph;
pub mod guide;
pub mod image;
pub mod input;
mod label;
pub mod mesh;
pub mod position;
pub mod render;
pub mod text;
pub mod theme;
mod ui;
pub mod utils;
pub mod widget;
pub mod cursor;
pub mod layout;
pub mod event_handler;
pub mod state;
pub mod flags;
pub mod draw;
pub mod window;
