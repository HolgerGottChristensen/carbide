use crate::widget::primitive::shape::rectangle::Rectangle;
use ::{Color, Rect};
use color::rgb;
use graph::Container;
use widget::{Id, Oval};
use widget::render::Render;
use widget::primitive::shape::oval::Full;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use render::primitive::Primitive;

#[derive(Copy, Clone)]
pub enum CWidget {
    Rectangle(Rectangle),
    Oval(Oval<Full>),
    Complex
}

impl Render for CWidget {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        match self {
            CWidget::Rectangle(n) => n.render(id, clip, container),
            CWidget::Oval(n) => n.render(id, clip, container),
            CWidget::Complex => {
                let kind = PrimitiveKind::Rectangle { color: Color::random()};
                return Some(new_primitive(id, kind, clip, container.rect));
            },

        }
    }
}