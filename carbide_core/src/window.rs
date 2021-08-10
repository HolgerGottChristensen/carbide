use std::path::Path;

use crate::image_map;
use crate::text::{FontFamily, FontId};
use crate::widget::Widget;

pub trait TWindow {
    fn add_font_family(&mut self, family: FontFamily) -> String;
    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId;
    fn add_image(&mut self, path: &str) -> image_map::Id;
    fn set_widgets(&mut self, w: Box<dyn Widget>);
}
