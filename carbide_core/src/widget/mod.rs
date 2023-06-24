use std::sync::atomic::{AtomicU32, Ordering};

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
pub use self::flag::*;
pub use self::flexibility::*;
pub use self::foreach::*;
pub use self::frame::*;
pub use self::h_split::*;
pub use self::h_stack::*;
pub use self::hidden::*;
pub use self::if_else::*;
pub use self::image::*;
pub use self::match_view::*;
pub use self::menu::*;
pub use self::mouse_area::*;
pub use self::offset::*;
pub use self::overlaid_layer::*;
pub use self::overlay::*;
pub use self::padding::*;
pub use self::progress_bar::*;
pub use self::progress_view::*;
pub use self::rotation_3d_effect::*;
pub use self::scroll::*;
pub use self::spacer::*;
pub use self::text::*;
pub use self::transform::*;
pub use self::v_split::*;
pub use self::v_stack::*;
pub use self::z_stack::*;
pub use self::duplicated::*;
pub use self::ignore::*;

pub mod canvas;
mod common;
mod shape;
mod types;

// Widgets
mod background;
mod blur;
mod border;
mod clip;
mod clip_shape;
mod empty;
mod environment_updating;
mod filter;
mod flag;
mod flexibility;
mod foreach;
mod frame;
mod h_split;
mod h_stack;
mod hidden;
mod if_else;
mod image;
mod match_view;
mod menu;
mod mouse_area;
mod offset;
mod overlaid_layer;
mod overlay;
mod padding;
mod popup_menu;
mod progress_bar;
mod progress_view;
mod rotation_3d_effect;
mod scroll;
mod spacer;
mod text;
mod transform;
mod v_split;
mod v_stack;
mod window_menu;
mod z_stack;
mod duplicated;
mod ignore;

#[derive(Clone, Debug, Default, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WidgetId(u32);

impl WidgetId {
    /// Generate a new widget ID.
    pub fn new() -> Self {
        static WIDGET_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
        WidgetId(WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub type ColoredPoint = (Position, crate::color::Rgba);
