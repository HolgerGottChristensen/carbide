use std::fmt::Debug;
use std::rc::Rc;

use crate::draw::{Dimension, Position};
use crate::flags::WidgetFlag;
use crate::focus::Focus;
use crate::layout::Layouter;
use crate::render::Render;
use crate::state::ValueCell;
use crate::update::Update;
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Widget, Debug, Clone)]
pub struct Duplicated<T>(Rc<ValueCell<T>>) where T: Widget;

impl Duplicated<Empty> {
    pub fn new<T: Widget>(widget: T) -> Duplicated<T> {
        Duplicated(Rc::new(ValueCell::new(widget)))
    }
}

impl<T: Widget> Duplicated<T> {
    pub fn duplicate(&self) -> Duplicated<T> {
        Duplicated(self.0.clone())
    }
}

impl<T: Widget> CommonWidget for Duplicated<T> {
    fn id(&self) -> WidgetId {
        self.0.borrow().id()
    }

    fn flag(&self) -> WidgetFlag {
        self.0.borrow().flag()
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.0.borrow().alignment()
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        self.0.borrow().apply(f, |a, b| a.foreach_child(b))
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_mut(b))
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_rev(b))
    }

    fn position(&self) -> Position {
        self.0.borrow().position()
    }

    fn set_position(&mut self, position: Position) {
        self.0.borrow_mut().set_position(position)
    }

    fn get_focus(&self) -> Focus {
        self.0.borrow().get_focus()
    }

    fn set_focus(&mut self, focus: Focus) {
        self.0.borrow_mut().set_focus(focus)
    }

    fn flexibility(&self) -> u32 {
        self.0.borrow().flexibility()
    }

    fn dimension(&self) -> Dimension {
        self.0.borrow().dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.0.borrow_mut().set_dimension(dimension)
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_direct(b))
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        self.0.borrow_mut().apply(f, |a, b| a.foreach_child_direct_rev(b))
    }
}

impl<T: Widget> WidgetExt for Duplicated<T> {}