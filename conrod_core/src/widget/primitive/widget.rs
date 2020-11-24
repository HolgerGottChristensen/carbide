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
use ::{text, Scalar};
use position::Dimensions;
use widget::common_widget::CommonWidget;
use widget::primitive::v_stack::VStack;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum CWidget {
    Rectangle(Rectangle),
    Line(Line),
    Oval(Oval<Full>),
    Text(Text),
    Image(Image),
    VStack(VStack),
    Complex
}

impl CommonWidget for CWidget {
    fn get_id(&self) -> Uuid {
        unimplemented!()
    }

    fn get_children(&self) -> &Vec<CWidget> {
        unimplemented!()
    }

    fn get_position(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_x(&self) -> f64 {
        unimplemented!()
    }

    fn set_x(&mut self, x: f64) {
        unimplemented!()
    }

    fn get_y(&self) -> f64 {
        unimplemented!()
    }

    fn set_y(&mut self, y: f64) {
        unimplemented!()
    }

    fn get_size(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        match self {
            CWidget::Rectangle(n) => {n.get_width()},
            CWidget::Line(_) => {0.0},
            CWidget::Oval(_) => {0.0},
            CWidget::Text(_) => {0.0},
            CWidget::Image(_) => {0.0},
            CWidget::VStack(_) => {0.0},
            CWidget::Complex => {0.0},
        }
    }

    fn get_height(&self) -> Scalar {
        match self {
            CWidget::Rectangle(n) => {n.get_height()},
            CWidget::Line(_) => {0.0},
            CWidget::Oval(_) => {0.0},
            CWidget::Text(_) => {0.0},
            CWidget::Image(_) => {0.0},
            CWidget::VStack(_) => {0.0},
            CWidget::Complex => {0.0},
        }
    }

    fn calc_width(&self, pref_width: f64) -> f64 {
        match self {
            CWidget::Rectangle(n) => {n.calc_width(pref_width)},
            CWidget::Line(_) => {0.0},
            CWidget::Oval(_) => {0.0},
            CWidget::Text(_) => {0.0},
            CWidget::Image(_) => {0.0},
            CWidget::VStack(_) => {0.0},
            CWidget::Complex => {0.0},
        }
    }

    fn calc_height(&self, pref_height: f64) -> f64 {
        match self {
            CWidget::Rectangle(n) => {n.calc_height(pref_height)},
            CWidget::Line(_) => {0.0},
            CWidget::Oval(_) => {0.0},
            CWidget::Text(_) => {0.0},
            CWidget::Image(_) => {0.0},
            CWidget::VStack(_) => {0.0},
            CWidget::Complex => {0.0},
        }
    }
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
            CWidget::VStack(n) => {n.layout(proposed_size, fonts, positioner)}
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
            CWidget::VStack(n) => {n.render(id, clip, container)}
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
            CWidget::VStack(n) => {n.get_primitives(proposed_dimensions, fonts)}
        }
    }
}