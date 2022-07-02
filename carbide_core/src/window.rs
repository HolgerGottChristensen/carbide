use std::path::Path;
use image::DynamicImage;

use crate::draw::image::{ImageId};
use crate::text::{FontFamily, FontId};
use crate::widget::Widget;
use crate::widget::Menu;

pub trait TWindow {
    fn add_font_family(&mut self, family: FontFamily) -> String;
    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId;
    fn add_image_from_path(&mut self, path: &str) -> Option<ImageId>;
    fn add_image(&mut self, id: ImageId, image: DynamicImage) -> Option<ImageId>;
    fn set_widgets(&mut self, w: Box<dyn Widget>);
    fn set_menu(&mut self, menu: Vec<Menu>);
}
