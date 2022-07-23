//! # carbide
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to carbide, we recommend checking out [The Guide](./guide/index.html).

#![deny(unsafe_code)]
//#![feature(associated_type_bounds)]
//#![warn(missing_copy_implementations)]
//#![warn(missing_docs)]

#[macro_use]
extern crate carbide_derive;
extern crate lyon;
extern crate self as carbide_core;
/*
macro_rules! delegate {
    ($name:ident, |$($item:ident: $type:ty),*| $body:block) => {
        struct $name<T> where T: StateContract {
            $(
            $item: $type,
            )*
        }

        impl<T: StateContract> Delegate<T> for $name<T> {
            fn call(&self, item: TState<T>, index: UsizeState) -> Box<dyn Widget> {
                todo!()
            }
        }
    };
}

delegate!(Name, |item: UsizeState, index: UsizeState| {});
*/

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

pub use futures::TryFutureExt;

pub use carbide_core::asynchronous::SpawnTask;
pub use carbide_derive::*;
pub use draw::Scalar;

pub use crate::color::Color;
pub use crate::ui::Ui;
pub use scene::Scene;

pub mod animation;
pub mod asynchronous;
pub mod color;
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
pub mod prelude;
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
