use crate::widget::primitive::shape::rectangle::Rectangle;
use ::{Color, Rect};
use color::rgb;
use graph::Container;
use widget::{Id, Oval, Line, Text, Image};
use widget::render::Render;
use widget::primitive::shape::oval::Full;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;
use text;
use position::Dimensions;
use widget::common_widget::CommonWidget;

#[derive(Clone, Debug)]
pub enum CWidget {
    Rectangle(Rectangle),
    Line(Line),
    Oval(Oval<Full>),
    Text(Text),
    Image(Image),
    Complex
}

impl Render for CWidget {
    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &dyn Fn(&mut dyn CommonWidget, Dimensions)) {
        match self {
            CWidget::Rectangle(n) => {n.layout(proposed_size, fonts, positioner)},
            CWidget::Oval(n) => {n.layout(proposed_size, fonts, positioner)},
            CWidget::Complex => {()},
            CWidget::Line(n) => {n.layout(proposed_size, fonts, positioner)}
            CWidget::Text(n) => {n.layout(proposed_size, fonts, positioner)}
            CWidget::Image(n) => {n.layout(proposed_size, fonts, positioner)}
        }
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        match self {
            CWidget::Rectangle(n) => n.render(id, clip, container),
            CWidget::Oval(n) => n.render(id, clip, container),
            CWidget::Complex => {
                let kind = PrimitiveKind::Rectangle { color: Color::random()};
                return Some(new_primitive(id, kind, clip, container.rect));
            },

            CWidget::Line(n) => {n.render(id, clip, container)}
            CWidget::Text(n) => {n.render(id, clip, container)}
            CWidget::Image(n) => {n.render(id, clip, container)}
        }
    }

    fn get_primitives(&self, proposed_dimensions: Dimensions, fonts: &text::font::Map) -> Vec<Primitive> {
        match self {
            CWidget::Rectangle(n) => {n.get_primitives(proposed_dimensions, fonts)},
            CWidget::Oval(n) => {n.get_primitives(proposed_dimensions, fonts)},
            CWidget::Complex => {vec![]},
            CWidget::Line(n) => {n.get_primitives(proposed_dimensions, fonts)}
            CWidget::Text(n) => {n.get_primitives(proposed_dimensions, fonts)}
            CWidget::Image(n) => {n.get_primitives(proposed_dimensions, fonts)}
        }
    }
}