pub use common::*;
pub use shape::*;
pub use types::*;

use crate::Point;

pub use self::border::*;
pub use self::clip::*;
pub use self::environment_updating::*;
pub use self::foreach::*;
pub use self::frame::*;
pub use self::h_stack::*;
pub use self::hidden::*;
pub use self::if_else::*;
pub use self::image::*;
pub use self::offset::*;
pub use self::overlaid_layer::*;
pub use self::padding::*;
pub use self::scroll::*;
pub use self::spacer::*;
pub use self::text::*;
pub use self::v_stack::*;
pub use self::z_stack::*;

pub mod canvas;
mod common;
mod shape;
mod types;

// Widgets
mod border;
mod clip;
mod environment_updating;
mod foreach;
mod frame;
mod h_stack;
mod hidden;
mod if_else;
mod image;
mod offset;
mod overlaid_layer;
mod padding;
mod scroll;
mod spacer;
mod text;
mod v_stack;
mod z_stack;

pub type Id = uuid::Uuid;
pub type ColoredPoint = (Point, crate::color::Rgba);



