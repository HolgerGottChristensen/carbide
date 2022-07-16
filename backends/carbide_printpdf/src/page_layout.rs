use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::flags::Flags;
use carbide_core::layout::Layout;
use carbide_core::prelude::WidgetExt;
use carbide_core::Scalar;
use carbide_core::widget::{CommonWidget, CrossAxisAlignment, EdgeInsets, Spacer, Widget, WidgetId, WidgetIter, WidgetIterMut};
use carbide_derive::Widget;
use crate::draw::{Dimension, Position};
use crate::prelude::*;

type NewPage = Spacer;

struct Page {
    dimension: Dimension,
    children: Vec<Box<dyn Widget>>,
    insets: EdgeInsets,
}

pub enum PageSize {
    A4Portrait
}

impl From<PageSize> for Dimension {
    fn from(size: PageSize) -> Self {
        match size {
            PageSize::A4Portrait => Dimension::new(210.0, 297.0)
        }
    }
}

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct PageLayout {
    id: WidgetId,
    pages: Vec<Page>,
    //cross_axis_alignment: CrossAxisAlignment,
}

impl PageLayout {
    pub fn new(size: impl Into<Dimension>, children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(PageLayout {
            id: WidgetId::new(),
            pages: vec![
                Page {
                    dimension: size.into(),
                    children,
                    insets: EdgeInsets::all(10.0)
                }
            ]
        })
    }
}

impl Layout for PageLayout {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let spacing = self.spacing;
        calculate_size_vstack(self, spacing, requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let spacing = self.spacing;
        let cross_axis_alignment = self.cross_axis_alignment;
        position_children_vstack(self, spacing, cross_axis_alignment)
    }
}

impl CommonWidget for PageLayout {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        let contains_proxy_or_ignored = self.children.iter().fold(false, |a, b| {
            a || (b.flag() == Flags::PROXY || b.flag() == Flags::IGNORE)
        });
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
        let contains_proxy_or_ignored = self.children.iter().fold(false, |a, b| {
            a || (b.flag() == Flags::PROXY || b.flag() == Flags::IGNORE)
        });
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

impl WidgetExt for PageLayout {}
