
use carbide_core::widget::Widget;
use crate::draw::{Dimension, Position, Scalar};
use crate::flags::Flags;
use crate::focus::Focus;
use crate::layout::{BasicLayouter, Layouter};
use crate::widget::{WidgetId};

pub trait CommonWidget {
    fn id(&self) -> WidgetId;
    fn flag(&self) -> Flags {
        Flags::EMPTY
    }
    fn is_proxy(&self) -> bool {
        self.flag() == Flags::PROXY
    }
    fn is_ignore(&self) -> bool {
        self.flag() == Flags::IGNORE
    }

    fn is_spacer(&self) -> bool {
        self.flag() == Flags::SPACER
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget));
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget));

    fn child_count(&self) -> usize {
        let mut count = 0;

        self.foreach_child(&mut |_child| {
            count += 1;
        });

        count
    }

    fn position(&self) -> Position;
    fn set_position(&mut self, position: Position);

    fn get_focus(&self) -> Focus {
        Focus::Unfocused
    }
    #[allow(unused_variables)]
    fn set_focus(&mut self, focus: Focus) {}

    fn alignment(&self) -> Box<dyn Layouter> {
        Box::new(BasicLayouter::Center)
    }

    #[allow(unused_variables)]
    fn set_alignment(&mut self, alignment: Box<dyn Layouter>) {
        unimplemented!()
    }

    /// 0 is the most flexible and the largest number is the least flexible
    /// The flexibility of the widget determines the order of which the widgets are processed
    /// when laying out in a vertical or horizontal stack. The least flexible are processed first.
    /// If not overwritten, the default behavior is to either use the first child's flexibility or
    /// if no child are present, be 0.
    fn flexibility(&self) -> u32 {
        let mut is_first = true;
        let mut flexibility = 0;

        self.foreach_child(&mut |s| {
            if !is_first { return }
            flexibility = s.flexibility();
            is_first = false;
        });

        flexibility
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
    ($self:ident, id: $id_expr:expr, child: $child:expr, position: $position:expr, dimension: $dimension:expr $(,flag: $flag:expr)? $(,flexibility: $flexibility:expr)? $(,alignment: $alignment:expr)?) => {
        fn id(&$self) -> carbide_core::widget::WidgetId {
            $id_expr
        }

        $(
            fn alignment(&$self) -> Box<dyn carbide_core::layout::Layouter> {
                $alignment.clone()
            }
        )?

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

        fn foreach_child<'a>(&'a $self, f: &mut dyn FnMut(&'a dyn Widget)) {
            if $child.is_ignore() {
                return;
            }

            if $child.is_proxy() {
                $child.foreach_child(f);
                return;
            }

            f(&$child);
        }

        fn foreach_child_mut<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            if $child.is_ignore() {
                return;
            }

            if $child.is_proxy() {
                $child.foreach_child_mut(f);
                return;
            }

            f(&mut $child);
        }

        fn foreach_child_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            if $child.is_ignore() {
                return;
            }

            if $child.is_proxy() {
                $child.foreach_child_rev(f);
                return;
            }

            f(&mut $child);
        }

        fn foreach_child_direct<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            f(&mut $child)
        }

        fn foreach_child_direct_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            f(&mut $child)
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
    };

    ($self:ident, id: $id_expr:expr, children: $children:expr, position: $position:expr, dimension: $dimension:expr $(,flag: $flag:expr)? $(,flexibility: $flexibility:literal)?) => {
        fn id(&$self) -> carbide_core::widget::WidgetId {
            $id_expr
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

        fn foreach_child<'a>(&'a $self, f: &mut dyn FnMut(&'a dyn Widget)) {
            for child in &$children {
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

        fn foreach_child_mut<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            for child in &mut $children {
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

        fn foreach_child_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            for child in &mut $children.iter_mut().rev() {
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

        fn foreach_child_direct<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            for child in &mut $children.iter_mut() {
                f(child);
            }
        }

        fn foreach_child_direct_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
            for child in &mut $children.iter_mut().rev() {
                f(child);
            }
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
    };

    ($self:ident, id: $id_expr:expr, position: $position:expr, dimension: $dimension:expr $(,flag: $flag:expr)? $(,flexibility: $flexibility:literal)?) => {
        fn id(&$self) -> carbide_core::widget::WidgetId {
            $id_expr
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

        fn foreach_child<'a>(&'a $self, _: &mut dyn FnMut(&'a dyn Widget)) {}
        fn foreach_child_mut<'a>(&'a mut $self, _: &mut dyn FnMut(&'a mut dyn Widget)) {}
        fn foreach_child_rev<'a>(&'a mut $self, _: &mut dyn FnMut(&'a mut dyn Widget)) {}
        fn foreach_child_direct<'a>(&'a mut $self, _: &mut dyn FnMut(&'a mut dyn Widget)) {}
        fn foreach_child_direct_rev<'a>(&'a mut $self, _: &mut dyn FnMut(&'a mut dyn Widget)) {}

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
    };
}
