use image::DynamicImage;
use std::path::Path;

use crate::draw::image::ImageId;
use crate::locate_folder::Search;
use crate::text::{FontFamily, FontId};
use crate::widget::Menu;
use crate::widget::Widget;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub usize);

pub trait TWindow {
    fn add_font_family(&mut self, family: FontFamily) -> String;
    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId;

    fn add_image_from_path(&mut self, path: &str) -> Option<ImageId> {
        let assets = Search::KidsThenParents(3, 5).for_folder("assets").unwrap();

        let path = assets.join(path);
        let image = carbide_core::image::open(&path).expect("Couldn't load logo");

        let id = ImageId::new(path);
        self.add_image(id.clone(), image);

        Some(id)
    }

    fn add_image(&mut self, id: ImageId, image: DynamicImage) -> Option<ImageId>;
    fn set_widgets(&mut self, w: Box<dyn Widget>);
    fn set_menu(&mut self, menu: Vec<Menu>);
}
