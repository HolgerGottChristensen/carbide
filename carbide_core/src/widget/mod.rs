pub use common::*;
pub use shape::*;
pub use types::*;

use crate::draw::Position;

pub use self::background::*;
pub use self::blur::*;
pub use self::border::*;
pub use self::clip::*;
pub use self::clip_shape::*;
pub use self::empty::*;
pub use self::environment_updating::*;
pub use self::filter::*;
pub use self::foreach::*;
pub use self::frame::*;
pub use self::h_stack::*;
pub use self::hidden::*;
pub use self::if_else::*;
pub use self::image::*;
pub use self::offset::*;
pub use self::overlaid_layer::*;
pub use self::overlay::*;
pub use self::padding::*;
pub use self::progress_view::*;
pub use self::rotation_3d_effect::*;
pub use self::scroll::*;
pub use self::spacer::*;
pub use self::text::*;
pub use self::transform::*;
pub use self::v_stack::*;
pub use self::z_stack::*;

pub mod canvas;
mod common;
mod shape;
mod types;

// Widgets
mod border;
mod clip;
mod clip_shape;
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
mod rotation_3d_effect;
mod transform;
mod blur;
mod filter;
mod overlay;
mod progress_view;
mod background;
mod empty;

pub type Id = uuid::Uuid;
pub type ColoredPoint = (Position, crate::color::Rgba);
