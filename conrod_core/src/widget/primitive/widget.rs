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
use widget::layout::Layout;
use text::font::Map;
use widget::primitive::frame::Frame;
use widget::primitive::h_stack::HStack;
use widget::primitive::z_stack::ZStack;
use widget::primitive::spacer::Spacer;
use widget::primitive::edge_insets::EdgeInsets;
use widget::primitive::padding::Padding;

#[derive(Clone, Debug)]
pub enum CWidget {
    Rectangle(Rectangle),
    Line(Line),
    Oval(Oval<Full>),
    Text(Text),
    Image(Image),
    VStack(VStack),
    HStack(HStack),
    ZStack(ZStack),
    Frame(Frame),
    Spacer(Spacer),
    Padding(Padding),
    Complex
}

impl CWidget {
    pub fn frame(self, width: Scalar, height: Scalar) -> Self {
        Frame::init(width, height, self)
    }

    pub fn padding(self, edge_insets: EdgeInsets) -> Self {
        Padding::init(edge_insets, self)
    }
}

impl Layout for CWidget {
    fn flexibility(&self) -> u32 {
        match self {
            CWidget::Rectangle(n) => {n.flexibility()}
            CWidget::Line(n) => {n.flexibility()}
            CWidget::Oval(n) => {n.flexibility()}
            CWidget::Text(n) => {n.flexibility()}
            CWidget::Image(n) => {n.flexibility()}
            CWidget::VStack(n) => {n.flexibility()}
            CWidget::HStack(n) => {n.flexibility()}
            CWidget::ZStack(n) => {n.flexibility()}
            CWidget::Spacer(n) => {n.flexibility()}
            CWidget::Padding(n) => {n.flexibility()}
            CWidget::Complex => {0}
            CWidget::Frame(n) => {n.flexibility()}
        }
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        match self {
            CWidget::Rectangle(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::Line(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::Oval(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::Text(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::Image(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::VStack(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::HStack(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::ZStack(n) => {n.calculate_size(requested_size, fonts)}
            CWidget::Frame(n) => n.calculate_size(requested_size, fonts),
            CWidget::Padding(n) => n.calculate_size(requested_size, fonts),
            CWidget::Spacer(n) => n.calculate_size(requested_size, fonts),
            CWidget::Complex => {[0.0,0.0]}
        }
    }

    fn position_children(&mut self) {
        match self {
            CWidget::Rectangle(n) => {n.position_children()}
            CWidget::Line(n) => {n.position_children()}
            CWidget::Oval(n) => {n.position_children()}
            CWidget::Text(n) => {n.position_children()}
            CWidget::Image(n) => {n.position_children()}
            CWidget::VStack(n) => {n.position_children()}
            CWidget::HStack(n) => {n.position_children()}
            CWidget::ZStack(n) => {n.position_children()}
            CWidget::Complex => {}
            CWidget::Frame(n) => {n.position_children()}
            CWidget::Padding(n) => {n.position_children()}
            CWidget::Spacer(n) => {n.position_children()}
        }
    }
}

impl CommonWidget for CWidget {
    fn get_id(&self) -> Uuid {
        unimplemented!()
    }

    fn get_children(&self) -> &Vec<CWidget> {
        unimplemented!()
    }

    fn get_position(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_x(&self) -> Scalar {
        match self {
            CWidget::Rectangle(n) => {n.get_x()},
            CWidget::Line(n) => {n.get_x()},
            CWidget::Oval(n) => {n.get_x()},
            CWidget::Text(n) => {n.get_x()},
            CWidget::Image(n) => {n.get_x()},
            CWidget::VStack(n) => {n.get_x()},
            CWidget::HStack(n) => {n.get_x()},
            CWidget::ZStack(n) => {n.get_x()},
            CWidget::Frame(n) => {n.get_x()},
            CWidget::Padding(n) => {n.get_x()},
            CWidget::Spacer(n) => {n.get_x()},
            CWidget::Complex => {0.0},
        }
    }

    fn set_x(&mut self, x: f64) {
        match self {
            CWidget::Rectangle(n) => {n.set_x(x)},
            CWidget::Line(n) => {n.set_x(x)},
            CWidget::Oval(n) => {n.set_x(x)},
            CWidget::Text(n) => {n.set_x(x)},
            CWidget::Image(n) => {n.set_x(x)},
            CWidget::VStack(n) => {n.set_x(x)},
            CWidget::HStack(n) => {n.set_x(x)},
            CWidget::ZStack(n) => {n.set_x(x)},
            CWidget::Frame(n) => {n.set_x(x)},
            CWidget::Padding(n) => {n.set_x(x)},
            CWidget::Spacer(n) => {n.set_x(x)},
            CWidget::Complex => {},
        }
    }

    fn get_y(&self) -> f64 {
        match self {
            CWidget::Rectangle(n) => {n.get_y()},
            CWidget::Line(n) => {n.get_y()},
            CWidget::Oval(n) => {n.get_y()},
            CWidget::Text(n) => {n.get_y()},
            CWidget::Image(n) => {n.get_y()},
            CWidget::VStack(n) => {n.get_y()},
            CWidget::HStack(n) => {n.get_y()},
            CWidget::ZStack(n) => {n.get_y()},
            CWidget::Frame(n) => {n.get_y()},
            CWidget::Spacer(n) => {n.get_y()},
            CWidget::Padding(n) => {n.get_y()},
            CWidget::Complex => {0.0},
        }
    }

    fn set_y(&mut self, y: f64) {
        match self {
            CWidget::Rectangle(n) => {n.set_y(y)},
            CWidget::Line(n) => {n.set_y(y)},
            CWidget::Oval(n) => {n.set_y(y)},
            CWidget::Text(n) => {n.set_y(y)},
            CWidget::Image(n) => {n.set_y(y)},
            CWidget::VStack(n) => {n.set_y(y)},
            CWidget::HStack(n) => {n.set_y(y)},
            CWidget::ZStack(n) => {n.set_y(y)},
            CWidget::Frame(n) => {n.set_y(y)},
            CWidget::Spacer(n) => {n.set_y(y)},
            CWidget::Padding(n) => {n.set_y(y)},
            CWidget::Complex => {},
        }
    }

    fn get_size(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        match self {
            CWidget::Rectangle(n) => {n.get_width()},
            CWidget::Line(n) => {n.get_width()},
            CWidget::Oval(n) => {n.get_width()},
            CWidget::Text(n) => {n.get_width()},
            CWidget::Image(n) => {n.get_width()},
            CWidget::VStack(n) => {n.get_width()},
            CWidget::ZStack(n) => {n.get_width()},
            CWidget::HStack(n) => {n.get_width()},
            CWidget::Frame(n) => {n.get_width()},
            CWidget::Spacer(n) => {n.get_width()},
            CWidget::Padding(n) => {n.get_width()},
            CWidget::Complex => {0.0},
        }
    }

    fn get_height(&self) -> Scalar {
        match self {
            CWidget::Rectangle(n) => {n.get_height()},
            CWidget::Line(n) => {n.get_height()},
            CWidget::Oval(n) => {n.get_height()},
            CWidget::Text(n) => {n.get_height()},
            CWidget::Image(n) => {n.get_height()},
            CWidget::VStack(n) => {n.get_height()},
            CWidget::HStack(n) => {n.get_height()},
            CWidget::ZStack(n) => {n.get_height()},
            CWidget::Frame(n) => {n.get_height()},
            CWidget::Spacer(n) => {n.get_height()},
            CWidget::Padding(n) => {n.get_height()},
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
            CWidget::HStack(_) => {0.0},
            CWidget::ZStack(_) => {0.0},
            CWidget::Frame(_) => {0.0},
            CWidget::Spacer(_) => {0.0},
            CWidget::Padding(_) => {0.0},
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
            CWidget::HStack(_) => {0.0},
            CWidget::ZStack(_) => {0.0},
            CWidget::Frame(_) => {0.0},
            CWidget::Spacer(_) => {0.0},
            CWidget::Padding(_) => {0.0},
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
            CWidget::HStack(n) => {n.layout(proposed_size, fonts, positioner)}
            CWidget::ZStack(n) => {n.layout(proposed_size, fonts, positioner)}
            CWidget::Frame(n) => {n.layout(proposed_size, fonts, positioner)}
            CWidget::Spacer(n) => {n.layout(proposed_size, fonts, positioner)}
            CWidget::Padding(n) => {n.layout(proposed_size, fonts, positioner)}
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
            CWidget::HStack(n) => {n.render(id, clip, container)}
            CWidget::ZStack(n) => {n.render(id, clip, container)}
            CWidget::Frame(n) => {n.render(id, clip, container)}
            CWidget::Spacer(n) => {n.render(id, clip, container)}
            CWidget::Padding(n) => {n.render(id, clip, container)}
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
            CWidget::HStack(n) => {n.get_primitives(proposed_dimensions, fonts)}
            CWidget::ZStack(n) => {n.get_primitives(proposed_dimensions, fonts)}
            CWidget::Frame(n) => {n.get_primitives(proposed_dimensions, fonts)}
            CWidget::Spacer(n) => {n.get_primitives(proposed_dimensions, fonts)}
            CWidget::Padding(n) => {n.get_primitives(proposed_dimensions, fonts)}
        }
    }
}