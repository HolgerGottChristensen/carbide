use std::ops::Deref;

use crate::draw::{Dimension, Position};
use crate::focus::Focus;
use crate::prelude::*;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct ZStack {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    alignment: BasicLayouter,
}

impl ZStack {
    pub fn initialize(children: Vec<Box<dyn Widget>>) -> Box<ZStack> {
        Box::new(ZStack {
            id: Uuid::new_v4(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: BasicLayouter::Center,
        })
    }

    pub fn alignment(mut self, alignment: BasicLayouter) -> Box<Self> {
        self.alignment = alignment;
        Box::new(self)
    }
}

impl Layout for ZStack {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let mut children_flexibility: Vec<(u32, &mut dyn Widget)> = self
            .children_mut()
            .map(|child| (child.flexibility(), child))
            .collect();
        children_flexibility.sort_by(|(a, _), (b, _)| a.cmp(&b));
        children_flexibility.reverse();

        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for (_, child) in children_flexibility {
            let chosen_size = child.calculate_size(requested_size, env);

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

    fn position_children(&mut self) {
        let positioning = self.alignment.positioner();
        let position = self.position;
        let dimension = self.dimension;

        for child in self.children_mut() {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl CommonWidget for ZStack {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        Box::new(self.alignment.clone())
    }

    fn children(&self) -> WidgetIter {
        self.children
            .iter()
            .map(|x| x.deref())
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.flag() == Flags::PROXY {
                    WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .map(|x| x.deref_mut())
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .map(|x| x.deref_mut())
            .fold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
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
}

impl WidgetExt for ZStack {}
