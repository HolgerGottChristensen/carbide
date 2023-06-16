//! # Carbide
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to carbide, we recommend checking out [The Guide](./guide/index.html).
//!
//! This issue is a gamechanger: https://rust-lang.github.io/rfcs/2528-type-changing-struct-update-syntax.html
//!
//! https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=45ea0874855a115a2b5d41f947908652

#![deny(unsafe_code)]
//#![feature(associated_type_bounds)]
//#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]

extern crate self as carbide_core;

pub use futures::TryFutureExt;

pub use carbide_core::asynchronous::SpawnTask;
pub use scene::Scene;

pub use draw::color;

#[macro_export]
macro_rules! lens {
    ($type:ty; $i:ident $(. $field:ident)+) => {
        carbide_core::state::WidgetState::new(Box::new(
            carbide_core::state::FieldState::new(
                $i.clone(),
                |item: &$type| { &item$(.$field)+ },
                |item: &mut $type| { &mut item$(.$field)+ }
            )
        ))
    };
    ($type:ty; |$i:ident| $bl:block ) => {
        $i.mapped(|$i: &$type| $bl)
    }
}

#[macro_export]
macro_rules! set_state {
    ($env:ident, self.$i:ident := $bl:block) => {{
        let res = $bl;
        self.$i.set_value(res);
        self.sync($env);
    }};
}

pub mod animation;
pub mod asynchronous;
pub mod cursor;
pub mod dialog;
pub mod draw;
pub mod environment;
pub mod event;
pub mod flags;
pub mod focus;
pub mod layout;
pub mod locate_folder;
pub mod mesh;
pub mod platform;
pub mod render;
pub mod state;
pub mod text;
mod ui;
pub mod utils;
pub mod widget;
pub mod window;
mod scene;

/// Reexport of the image crate
pub mod image {
    pub use image::*;
}
