use std::path::Path;
use image::DynamicImage;

use crate::draw::image::{ImageId};
use crate::locate_folder::Search;
use crate::text::{FontFamily, FontId};
use crate::widget::Widget;
use crate::widget::Menu;

pub trait TWindow {
    fn add_font_family(&mut self, family: FontFamily) -> String;
    fn add_font<P: AsRef<Path>>(&mut self, path: P) -> FontId;

    fn add_image_from_path(&mut self, path: &str) -> Option<ImageId> {
        let assets = Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .unwrap();

        let image = carbide_core::image::open(assets.join(path)).expect("Couldn't load logo");

        let id = ImageId::new();
        self.add_image(id, image);

        Some(id)
    }

    fn add_image(&mut self, id: ImageId, image: DynamicImage) -> Option<ImageId>;
    fn set_widgets(&mut self, w: Box<dyn Widget>);
    fn set_menu(&mut self, menu: Vec<Menu>);
}
