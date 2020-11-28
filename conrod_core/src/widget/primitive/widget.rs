use crate::widget::primitive::shape::rectangle::Rectangle;
use ::{Color, Rect};
use color::rgb;
use graph::Container;
use widget::{Id, Oval, Line, Text, Image, common_widget};
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
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use std::fmt::Debug;
use std::fmt;
use std::ops::{Deref, DerefMut};

pub trait Widget: Event + Layout + Render {}

impl<T> Widget for T where T: Event + Layout + Render {}

pub trait WidgetExt: Widget + Sized + 'static {
    fn frame(self, width: Scalar, height: Scalar) -> Box<Frame> {
        Frame::init(width, height, Box::new(self))
    }

    fn padding(self, edge_insets: EdgeInsets) -> Box<Padding> {
        Padding::init(edge_insets, Box::new(self))
    }
}

//This does not currently work with intellisense
//impl<T> WidgetExt for T where T: Widget + 'static {}

impl CommonWidget for Box<Widget> {
    fn get_id(&self) -> Uuid {
        self.deref().get_id()
    }

    fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        self.deref().get_children()
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        self.deref_mut().get_children_mut()
    }

    fn get_position(&self) -> [f64; 2] {
        self.deref().get_position()
    }

    fn get_x(&self) -> f64 {
        self.deref().get_x()
    }

    fn set_x(&mut self, x: f64) {
        self.deref_mut().set_x(x)
    }

    fn get_y(&self) -> f64 {
        self.deref().get_y()
    }

    fn set_y(&mut self, y: f64) {
        self.deref_mut().set_y(y)
    }

    fn get_size(&self) -> [f64; 2] {
        self.deref().get_size()
    }

    fn get_width(&self) -> f64 {
        self.deref().get_width()
    }

    fn get_height(&self) -> f64 {
        self.deref().get_height()
    }
}

impl Event for Box<Widget> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        self.deref_mut().handle_mouse_event(event, consumed)
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.deref_mut().handle_keyboard_event(event)
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        self.deref_mut().handle_other_event(event)
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        self.deref_mut().process_mouse_event(event, consumed)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.deref_mut().process_keyboard_event(event)
    }
}

impl Layout for Box<Widget> {
    fn flexibility(&self) -> u32 {
        self.deref().flexibility()
    }

    fn calculate_size(&mut self, requested_size: [f64; 2], fonts: &Map) -> [f64; 2] {
        self.deref_mut().calculate_size(requested_size, fonts)
    }

    fn position_children(&mut self) {
        self.deref_mut().position_children()
    }
}

impl Render for Box<Widget> {
    fn layout(&mut self, proposed_size: [f64; 2], fonts: &Map, positioner: &dyn Fn(&mut dyn CommonWidget, [f64; 2])) {
        unimplemented!()
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, proposed_size: [f64; 2], fonts: &Map) -> Vec<Primitive> {
        self.deref().get_primitives(proposed_size, fonts)
    }
}


impl Debug for Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Widget: {}", self.get_id())
    }
}