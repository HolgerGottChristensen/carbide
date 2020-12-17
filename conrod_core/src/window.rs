use text::font::{Id, Error};
use widget::primitive::Widget;
use image;

pub trait TWindow<S: 'static + Clone> {
    fn add_font(&mut self, path: &str) -> Result<Id, Error>;
    fn add_image(&mut self, path: &str) -> Result<image::Id, Error>;
    fn set_widgets(&mut self, w: Box<dyn Widget<S>>);
}