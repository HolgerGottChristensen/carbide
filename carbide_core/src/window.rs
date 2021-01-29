use crate::image_map;
use crate::text::font::{Error, Id};
use crate::widget::primitive::Widget;

pub trait TWindow<S: 'static + Clone> {
    fn add_font(&mut self, path: &str) -> Result<Id, Error>;
    fn add_image(&mut self, path: &str) -> Result<image_map::Id, Error>;
    fn set_widgets(&mut self, w: Box<dyn Widget<S>>);
}