use std::hash::{BuildHasherDefault, DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};

pub use carbide_derive::Widget;
pub use common::*;
pub use shape::*;
pub use types::*;

use crate::draw::Position;

pub use self::absolute::*;
pub use self::aspect_ratio::*;
pub use self::background::*;
pub use self::blur::*;
pub use self::border::*;
pub use self::clip::*;
pub use self::clip_shape::*;
pub use self::empty::*;
pub use self::environment_updating_new2::*;
pub use self::environment_updating_new3::*;
pub use self::environment_updating_new::*;
pub use self::filter::*;
pub use self::flag::*;
pub use self::flexibility::*;
pub use self::foreach::*;
pub use self::frame::*;
pub use self::geometry_reader::*;
pub use self::h_grid::*;
pub use self::h_split::*;
pub use self::h_stack::*;
pub use self::hidden::*;
pub use self::hue_rotation::*;
pub use self::if_else::*;
pub use self::ignore::*;
pub use self::image::*;
pub use self::mask::*;
pub use self::menu::*;
pub use self::mouse_area::*;
pub use self::navigation_stack::*;
pub use self::offset::*;
pub use self::on_change::*;
pub use self::on_key::*;
pub use self::overlay::*;
pub use self::padding::*;
pub use self::progress_bar::*;
pub use self::progress_view::*;
pub use self::proxy::*;
pub use self::rotation_3d_effect::*;
pub use self::saturation::*;
pub use self::scroll::*;
pub use self::shadow::*;
pub use self::spacer::*;
pub use self::text::*;
pub use self::transform::*;
pub use self::v_grid::*;
pub use self::v_split::*;
pub use self::v_stack::*;
pub use self::z_stack::*;

pub mod canvas;
mod common;
mod shape;
mod types;

// Widgets
mod absolute;
mod background;
mod blur;
mod border;
mod clip;
mod clip_shape;
mod empty;
mod filter;
mod flag;
mod flexibility;
mod foreach;
mod frame;
mod geometry_reader;
mod h_grid;
mod h_split;
mod h_stack;
mod hidden;
mod if_else;
mod ignore;
mod image;
mod menu;
mod mouse_area;
mod navigation_stack;
mod offset;
mod on_change;
mod overlay;
mod padding;
mod progress_bar;
mod progress_view;
mod proxy;
mod rotation_3d_effect;
mod scroll;
mod spacer;
mod text;
mod transform;
mod v_grid;
mod v_split;
mod v_stack;
mod z_stack;
mod aspect_ratio;
mod on_key;
mod shadow;
mod hue_rotation;
mod saturation;
mod luminance;
mod mask;
mod environment_updating_new;
mod environment_updating_new2;
mod environment_updating_new3;
pub mod managers;
mod foreach_widget;

#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct WidgetId(u32, u64);

impl WidgetId {
    /// Generate a new widget ID.
    pub fn new() -> Self {
        static WIDGET_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

        let id = WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut hasher = DefaultHasher::new();
        hasher.write_u32(id);
        WidgetId(id, hasher.finish())
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn from_u32(id: u32) -> WidgetId {
        let mut hasher = DefaultHasher::new();
        hasher.write_u32(id);
        WidgetId(id, hasher.finish())
    }
}

impl Hash for WidgetId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state)
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        WidgetId::new()
    }
}

pub type BuildWidgetIdHasher = BuildHasherDefault<WidgetIdHasher>;

#[derive(Default, Clone, Copy, Debug)]
pub struct WidgetIdHasher(u64);

impl Hasher for WidgetIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("The hasher is only used with TypeId and it uses write_u64")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

pub type ColoredPoint = (Position, crate::color::Rgba);
