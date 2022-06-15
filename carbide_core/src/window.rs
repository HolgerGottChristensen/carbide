use std::path::Path;

use crate::image_map;
use crate::text::{FontFamily, FontId};
use crate::widget::Widget;
use crate::widget::Menu;

pub trait TWindow {
    fn add_font_family(&mut self, family: FontFamily) -> String;
    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId;
    fn add_image_from_path(&mut self, path: &str) -> Option<image_map::ImageId>;
    fn add_image(&mut self, image: image::DynamicImage) -> Option<image_map::ImageId>;
    fn set_widgets(&mut self, w: Box<dyn Widget>);
    fn set_menu(&mut self, menu: Vec<Menu>);
}
