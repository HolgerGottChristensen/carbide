#![feature(in_band_lifetimes)]
//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to Conrod, we recommend checking out [The Guide](./guide/index.html).

#![warn(unsafe_code)] //Todo deny when unsafe code removed from foreach
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]

#[macro_use] extern crate conrod_derive;
#[macro_use] extern crate bitflags;
extern crate daggy;
extern crate fnv;
extern crate num;
extern crate input as piston_input;
extern crate rusttype;
extern crate copypasta;
extern crate uuid;
extern crate instant;
extern crate glium;
extern crate serde;
extern crate ron;
#[macro_use] extern crate dyn_clone;

pub use color::{Color, Colorable};
pub use conrod_derive::*;
pub use border::{Bordering, Borderable};
pub use label::{FontSize, Labelable};
pub use position::{Dimension, Point, Position, Positionable, Range, Rect, Scalar, Sizeable};
pub use theme::Theme;
pub use ui::{Ui, UiCell, UiBuilder};
pub use widget::{scroll, OldWidget};

pub use ron::to_string as to_ron;
pub use ron::from_str as from_ron;

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

