use crate::image_map;
use crate::text::FontId;
use crate::widget::primitive::Widget;

pub trait TWindow<S: 'static + Clone> {
    fn add_font(&mut self, path: &str) -> FontId;
    fn add_image(&mut self, path: &str) -> image_map::Id;
    fn set_widgets(&mut self, w: Box<dyn Widget<S>>);
}