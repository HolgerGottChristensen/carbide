
use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::{calculate_size_vstack, Layout, position_children_vstack};
use crate::Scalar;
use crate::widget::{CommonWidget, CrossAxisAlignment, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct VStack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl VStack {

    #[carbide_default_builder]
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {}

    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(VStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            spacing: 10.0,
            cross_axis_alignment: CrossAxisAlignment::Center,
        })
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Box<Self> {
        self.cross_axis_alignment = alignment;
        Box::new(self)
    }

    pub fn spacing(mut self, spacing: f64) -> Box<Self> {
        self.spacing = spacing;
        Box::new(self)
    }
}

impl Layout for VStack {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let spacing = self.spacing;
        calculate_size_vstack(self, spacing, requested_size, env);
        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let spacing = self.spacing;
        let cross_axis_alignment = self.cross_axis_alignment;
        position_children_vstack(self, spacing, cross_axis_alignment, env)
    }
}

impl CommonWidget for VStack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        for child in &self.children {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for child in &mut self.children {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child_mut(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for child in self.children.iter_mut().rev() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child_rev(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for child in self.children.iter_mut() {
            f(child);
        }
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        for child in self.children.iter_mut().rev() {
            f(child);
        }
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn flexibility(&self) -> u32 {
        1
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for VStack {}
