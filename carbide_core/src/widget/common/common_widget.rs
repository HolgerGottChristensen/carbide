use crate::draw::Alignment;
use crate::widget::{WidgetId};
use crate::common::cursor::MouseCursor;
use crate::draw::{Dimension, Position, Rect, Scalar};
use crate::common::flags::WidgetFlag;
use crate::focus::Focus;
use crate::identifiable::Identifiable;
use crate::widget::AnyWidget;

// A Logical child is a widget that is not a proxy, meaning if we have a proxy, we must
// traverse it to get the actual logical children.

pub trait CommonWidget: Identifiable<Id=WidgetId> {
    fn flag(&self) -> WidgetFlag {
        WidgetFlag::EMPTY
    }
    fn is_proxy(&self) -> bool {
        self.flag() == WidgetFlag::PROXY
    }
    fn is_ignore(&self) -> bool {
        self.flag() == WidgetFlag::IGNORE
    }
    fn is_focusable(&self) -> bool {
        self.flag().contains(WidgetFlag::FOCUSABLE)
    }
    fn is_spacer(&self) -> bool {
        self.flag() == WidgetFlag::SPACER
    }

    /// Get a mutable reference to the logical child at the given index. Panics if the index is out of bounds.
    ///
    /// Implementations of this should focus on being efficient, and O(1), but it is not guaranteed.
    fn child(&mut self, index: usize) -> &mut dyn AnyWidget;

    /// Get the total number of logical children.
    ///
    /// Implementations of this should focus on being efficient, and O(1), but it is not guaranteed.
    fn child_count(&mut self) -> usize;

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget));
    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget));


    fn position(&self) -> Position;
    fn set_position(&mut self, position: Position);

    fn get_focus(&self) -> Focus {
        Focus::Unfocused
    }

    #[allow(unused_variables)]
    fn set_focus(&mut self, focus: Focus) {}

    fn alignment(&self) -> Alignment {
        Alignment::Center
    }

    #[allow(unused_variables)]
    fn set_alignment(&mut self, alignment: Alignment) {
        unimplemented!()
    }

    /// 0 is the most flexible and the largest number is the least flexible
    /// The flexibility of the widget determines the order of which the widgets are processed
    /// when laying out in a vertical or horizontal stack. The least flexible are processed first.
    /// If not overwritten, the default behavior is to either use the first child's flexibility or
    /// if no child are present, be 0.
    fn flexibility(&mut self) -> u32 {
        if self.child_count() != 0 {
            return self.child(0).flexibility()
        }

        0
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

    fn bounding_box(&self) -> Rect {
        Rect::new(
            self.position(),
            self.dimension()
        )
    }

    fn center_point(&self) -> Position {
        self.bounding_box().center()
    }

    fn cursor(&self) -> Option<MouseCursor> {
        None
    }
}

#[macro_export]
macro_rules! CommonWidgetImpl {
    ($self:ident, child: () $(, $($rest:tt)*)?) => {
        fn child(&mut $self, index: usize) -> &mut dyn $crate::widget::AnyWidget {
            panic!("Widget does not have children. Index out of bounds: {}", index)
        }
        fn child_count(&mut $self) -> usize {
            0
        }

        fn foreach_child(&mut $self, _f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {}
        fn foreach_child_rev(&mut $self, _f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {}

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, child: [$($child:expr),+] $(, $($rest:tt)*)?) => {
        #[allow(unused_imports)]
        fn child(&mut $self, i: usize) -> &mut dyn $crate::widget::AnyWidget {
            use $crate::widget::AnySequence;
            let mut passed = 0;

            $(
                let child_count = $child.count();

                if i < passed + child_count {
                    $child.index(i - passed)
                }

                passed += child_count;
            )+

            panic!("Index out of bounds")
        }

        #[allow(unused_imports)]
        fn child_count(&mut $self) -> usize {
            use $crate::widget::AnySequence;
            $($child.count() + )+ 0
        }

        #[allow(unused_imports)]
        fn foreach_child(&mut $self, f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $($child.foreach(f);)+
        }

        #[allow(unused_imports)]
        fn foreach_child_rev(&mut $self, f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            // TODO: Rev here does not actually reverse
            $($child.foreach_rev(f);)+
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, child: $child:expr $(, $($rest:tt)*)?) => {

        fn child(&mut $self, i: usize) -> &mut dyn $crate::widget::AnyWidget {
            use $crate::widget::AnySequence;
            $child.index(i)
        }
        fn child_count(&mut $self) -> usize {
            use $crate::widget::AnySequence;
            $child.count()
        }

        #[allow(unused_imports)]
        fn foreach_child(&mut $self, f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_rev(&mut $self, f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach_rev(f);
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };


    ($self:ident, position: $position:expr $(, $($rest:tt)*)?) => {
        fn position(&$self) -> $crate::draw::Position {
            $position
        }

        fn set_position(&mut $self, position: $crate::draw::Position) {
            $position = position;
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, dimension: $dimension:expr $(, $($rest:tt)*)?) => {
        fn dimension(&$self) -> $crate::draw::Dimension {
            $dimension
        }

        fn set_dimension(&mut $self, dimension: $crate::draw::Dimension) {
            $dimension = dimension
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, flag: $flag:expr $(, $($rest:tt)*)?) => {
        fn flag(&$self) -> $crate::flags::WidgetFlag {
            $flag
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, flexibility: $flexibility:expr $(, $($rest:tt)*)?) => {
        fn flexibility(&mut $self) -> u32 {
            $flexibility
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, alignment: $alignment:expr $(, $($rest:tt)*)?) => {
        fn alignment(&$self) -> Alignment {
            $alignment.clone()
        }

        fn set_alignment(&mut $self, alignment: Alignment) {
            $alignment = alignment;
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, focus: $focus:expr $(, $($rest:tt)*)?) => {
        fn get_focus(&$self) -> Focus {
            $focus.value().clone()
        }

        fn set_focus(&mut $self, focus: Focus) {
            $focus.set_value(focus);
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident,) => {};
    ($self:ident) => {};
}

#[macro_export]
macro_rules! ModifierWidgetImpl {
    ($self:ident, child: $child:expr) => {

        fn flag(&$self) -> $crate::flags::WidgetFlag {
            $child.flag()
        }

        fn position(&$self) -> $crate::draw::Position {
            $child.position()
        }

        fn set_position(&mut $self, position: $crate::draw::Position) {
            $child.set_position(position);
        }

        fn get_focus(&$self) -> $crate::focus::Focus {
            $child.get_focus()
        }

        fn set_focus(&mut $self, focus: $crate::focus::Focus) {
            $child.set_focus(focus);
        }

        fn alignment(&$self) -> $crate::draw::Alignment {
            $child.alignment()
        }

        fn flexibility(&mut $self) -> u32 {
            $child.flexibility()
        }

        fn dimension(&$self) -> $crate::draw::Dimension {
            $child.dimension()
        }

        fn set_dimension(&mut $self, dimension: $crate::draw::Dimension) {
            $child.set_dimension(dimension);
        }

        fn child(&mut $self, i: usize) -> &mut dyn $crate::widget::AnyWidget {
            use $crate::widget::AnySequence;
            $child.index(i)
        }
        fn child_count(&mut $self) -> usize {
            use $crate::widget::AnySequence;
            $child.count()
        }

        #[allow(unused_imports)]
        fn foreach_child(&mut $self, f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_rev(&mut $self, f: &mut dyn FnMut(&mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach_rev(f);
        }
    }
}
