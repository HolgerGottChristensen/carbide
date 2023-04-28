use std::ops::DerefMut;


use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut, WidgetValMut};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct ZStack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    alignment: Box<dyn Layouter>,
}

impl ZStack {

    #[carbide_default_builder]
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<ZStack> {}

    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<ZStack> {
        Box::new(ZStack {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Box::new(BasicLayouter::Center),
        })
    }

    pub fn with_alignment(mut self, layouter: BasicLayouter) -> Box<Self> {
        self.alignment = Box::new(layouter);
        Box::new(self)
    }
}

impl Layout for ZStack {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let mut children_flexibility: Vec<(u32, &mut dyn Widget)> = vec![];

        self.foreach_child_mut(&mut |child| {
            children_flexibility.push((child.flexibility(), child));
        });

        children_flexibility.sort_by(|(a, _), (b, _)| a.cmp(&b));
        children_flexibility.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, mut child) in children_flexibility {
            let new_requested_size = Dimension::new(
                requested_size.width.max(max_width),
                requested_size.height.max(max_height),
            );
            let chosen_size = child.calculate_size(new_requested_size, env);

            if chosen_size.width > max_width {
                max_width = chosen_size.width;
            }

            if chosen_size.height > max_height {
                max_height = chosen_size.height;
            }
        }

        self.dimension = Dimension::new(max_width, max_height);
        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let positioning = self.alignment.positioner();
        let position = self.position;
        let dimension = self.dimension;

        self.foreach_child_mut(&mut |child| {
            positioning(position, dimension, child);
            child.position_children(env);
        });
    }
}

impl CommonWidget for ZStack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.alignment.clone()
    }

    fn set_alignment(&mut self, alignment: Box<dyn Layouter>) {
        self.alignment = alignment;
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    // Todo: This should maybe be the flexibility of the least flexible child?
    fn flexibility(&self) -> u32 {
        1
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
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
}

impl WidgetExt for ZStack {}
