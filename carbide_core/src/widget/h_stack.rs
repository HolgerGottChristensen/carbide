use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::widget::CrossAxisAlignment;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct HStack {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Position,
    dimension: Dimension,
    spacing: Scalar,
    cross_axis_alignment: CrossAxisAlignment,
}

impl HStack {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(HStack {
            id: Uuid::new_v4(),
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

impl Layout for HStack {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let spacing = self.spacing;
        calculate_size_hstack(self, spacing, requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let spacing = self.spacing;
        let cross_axis_alignment = self.cross_axis_alignment;
        position_children_hstack(self, spacing, cross_axis_alignment)
    }
}

impl CommonWidget for HStack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        let contains_proxy_or_ignored = self.children.iter().fold(false, |a, b| a || (b.flag() == Flags::PROXY || b.flag() == Flags::IGNORE));
        if !contains_proxy_or_ignored {
            WidgetIter::Vec(self.children.iter())
        } else {
            self.children
                .iter()
                .filter(|x| x.flag() != Flags::IGNORE)
                .rfold(WidgetIter::Empty, |acc, x| {
                    if x.flag() == Flags::PROXY {
                        WidgetIter::Multi(Box::new(x.children()), Box::new(acc))
                    } else {
                        WidgetIter::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        let contains_proxy_or_ignored = self.children.iter().fold(false, |a, b| a || (b.flag() == Flags::PROXY || b.flag() == Flags::IGNORE));
        if !contains_proxy_or_ignored {
            WidgetIterMut::Vec(self.children.iter_mut())
        } else {
            self.children
                .iter_mut()
                .filter(|x| x.flag() != Flags::IGNORE)
                .rfold(WidgetIterMut::Empty, |acc, x| {
                    if x.flag() == Flags::PROXY {
                        WidgetIterMut::Multi(Box::new(x.children_mut()), Box::new(acc))
                    } else {
                        WidgetIterMut::Single(x, Box::new(acc))
                    }
                })
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Vec(self.children.iter_mut())
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::VecRev(self.children.iter_mut().rev())
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

impl WidgetExt for HStack {}
