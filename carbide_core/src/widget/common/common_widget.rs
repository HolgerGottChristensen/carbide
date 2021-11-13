use crate::draw::{Dimension, Position, Scalar};
use crate::flags::Flags;
use crate::focus::Focus;
use crate::layout::{BasicLayouter, Layouter};
use crate::widget::common::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::widget::Id;

pub trait CommonWidget {
    fn id(&self) -> Id;
    fn set_id(&mut self, id: Id);
    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    /// Get the logical children. This means for example for a vstack with a foreach,
    /// the children of the foreach is retrieved.
    fn children(&self) -> WidgetIter;
    fn children_mut(&mut self) -> WidgetIterMut;

    /// Get the direct children. This means for example for a vstack with a foreach,
    /// the foreach widget is retrieved.
    fn children_direct(&mut self) -> WidgetIterMut;
    fn children_direct_rev(&mut self) -> WidgetIterMut;

    fn position(&self) -> Position;
    fn set_position(&mut self, position: Position);

    fn get_focus(&self) -> Focus {
        Focus::Unfocused
    }
    fn set_focus(&mut self, _focus: Focus) {}

    fn alignment(&self) -> Box<dyn Layouter> {
        Box::new(BasicLayouter::Center)
    }
    fn set_alignment(&mut self, alignment: Box<dyn Layouter>) {
        unimplemented!()
    }
    /// 0 is the most flexible and the largest number is the least flexible
    /// The flexibility of the widget determines the order of which the widgets are processed
    /// when laying out in a vertical or horizontal stack. The least flexible are processed first.
    /// If not overwritten, the default behavior is to either use the first child's flexibility or
    /// if no child are present, be 0.
    fn flexibility(&self) -> u32 {
        if let Some(first_child) = self.children().next() {
            first_child.flexibility()
        } else {
            0
        }
    }

    fn x(&self) -> Scalar {
        self.position().x
    }

    fn set_x(&mut self, x: Scalar) {
        self.set_position(Position::new(x, self.y()));
    }

    fn y(&self) -> Scalar {
        self.position().y
    }

    fn set_y(&mut self, y: Scalar) {
        self.set_position(Position::new(self.x(), y));
    }

    fn dimension(&self) -> Dimension;
    fn set_dimension(&mut self, dimension: Dimension);

    fn width(&self) -> Scalar {
        self.dimension().width
    }

    fn set_width(&mut self, width: Scalar) {
        self.set_dimension(Dimension::new(width, self.height()))
    }

    fn height(&self) -> Scalar {
        self.dimension().height
    }

    fn set_height(&mut self, height: Scalar) {
        self.set_dimension(Dimension::new(self.width(), height))
    }

    fn is_inside(&self, point: Position) -> bool {
        point.x >= self.x()
            && point.x < self.x() + self.width()
            && point.y >= self.y()
            && point.y < self.y() + self.height()
    }
}

#[macro_export]
macro_rules! CommonWidgetImpl {
    ($typ:ty, $self:ident, id: $id_expr:expr, child: $child:expr, position: $position:expr, dimension: $dimension:expr $(,flag: $flag:expr)? $(,flexibility: $flexibility:literal)?) => {
        impl carbide_core::widget::CommonWidget for $typ {
            fn id(&$self) -> carbide_core::widget::Id {
                $id_expr
            }

            fn set_id(&mut $self, id: carbide_core::widget::Id) {
                $id_expr = id;
            }

            $(
                fn flag(&$self) -> carbide_core::flags::Flags {
                    $flag
                }
            )?

            $(
                fn flexibility(&$self) -> u32 {
                    $flexibility
                }
            )?

            fn children(&$self) -> carbide_core::widget::WidgetIter {
                if $child.flag() == carbide_core::flags::Flags::PROXY {
                    $child.children()
                } else if $child.flag() == carbide_core::flags::Flags::IGNORE {
                    carbide_core::widget::WidgetIter::Empty
                } else {
                    carbide_core::widget::WidgetIter::single(&$child)
                }
            }

            fn children_mut(&mut $self) -> carbide_core::widget::WidgetIterMut {
                if $child.flag() == carbide_core::flags::Flags::PROXY {
                    $child.children_mut()
                } else if $child.flag() == carbide_core::flags::Flags::IGNORE {
                    carbide_core::widget::WidgetIterMut::Empty
                } else {
                    carbide_core::widget::WidgetIterMut::single(&mut $child)
                }
            }

            fn children_direct(&mut $self) -> carbide_core::widget::WidgetIterMut {
                carbide_core::widget::WidgetIterMut::single(&mut $child)
            }

            fn children_direct_rev(&mut $self) -> carbide_core::widget::WidgetIterMut {
                carbide_core::widget::WidgetIterMut::single(&mut $child)
            }

            fn position(&$self) -> carbide_core::draw::Position {
                $position
            }

            fn set_position(&mut $self, position: carbide_core::draw::Position) {
                $position = position;
            }

            fn dimension(&$self) -> carbide_core::draw::Dimension {
                $dimension
            }

            fn set_dimension(&mut $self, dimension: carbide_core::draw::Dimension) {
                $dimension = dimension
            }
        }
    };

    ($typ:ty, $self:ident, id: $id_expr:expr, position: $position:expr, dimension: $dimension:expr $(,flag: $flag:expr)? $(,flexibility: $flexibility:literal)?) => {
        impl carbide_core::widget::CommonWidget for $typ {
            fn id(&$self) -> carbide_core::widget::Id {
                $id_expr
            }

            fn set_id(&mut $self, id: carbide_core::widget::Id) {
                $id_expr = id;
            }

            $(
                fn flag(&$self) -> carbide_core::flags::Flags {
                    $flag
                }
            )?

            $(
                fn flexibility(&$self) -> u32 {
                    $flexibility
                }
            )?

            fn children(&$self) -> carbide_core::widget::WidgetIter {
                carbide_core::widget::WidgetIter::Empty
            }

            fn children_mut(&mut $self) -> carbide_core::widget::WidgetIterMut {
                carbide_core::widget::WidgetIterMut::Empty
            }

            fn children_direct(&mut $self) -> carbide_core::widget::WidgetIterMut {
                carbide_core::widget::WidgetIterMut::Empty
            }

            fn children_direct_rev(&mut $self) -> carbide_core::widget::WidgetIterMut {
                carbide_core::widget::WidgetIterMut::Empty
            }

            fn position(&$self) -> carbide_core::draw::Position {
                $position
            }

            fn set_position(&mut $self, position: carbide_core::draw::Position) {
                $position = position;
            }

            fn dimension(&$self) -> carbide_core::draw::Dimension {
                $dimension
            }

            fn set_dimension(&mut $self, dimension: carbide_core::draw::Dimension) {
                $dimension = dimension
            }
        }
    };
}
