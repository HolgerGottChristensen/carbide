use carbide::draw::Alignment;
use carbide::widget::Identifiable;
use crate::misc::cursor::MouseCursor;
use crate::draw::{Dimension, Position, Rect, Scalar};
use crate::misc::flags::WidgetFlag;
use crate::focus::Focus;
use crate::widget::AnyWidget;

pub trait CommonWidget: Identifiable {
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

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget));
    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));
    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget));

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
        fn foreach_child<'a>(&'a $self, _f: &mut dyn FnMut(&'a dyn carbide::widget::AnyWidget)) {}
        fn foreach_child_mut<'a>(&'a mut $self, _f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {}
        fn foreach_child_rev<'a>(&'a mut $self, _f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {}
        fn foreach_child_direct<'a>(&'a mut $self, _f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {}
        fn foreach_child_direct_rev<'a>(&'a mut $self, _f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {}

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, child: $child:expr $(, $($rest:tt)*)?) => {
        #[allow(unused_imports)]
        fn foreach_child<'a>(&'a $self, f: &mut dyn FnMut(&'a dyn carbide::widget::AnyWidget)) {
            use carbide::widget::AnySequence;
            $child.foreach(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_mut<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {
            use carbide::widget::AnySequence;
            $child.foreach_mut(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {
            use carbide::widget::AnySequence;
            $child.foreach_rev(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_direct<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {
            use carbide::widget::AnySequence;
            $child.foreach_direct(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_direct_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn carbide::widget::AnyWidget)) {
            use carbide::widget::AnySequence;
            $child.foreach_direct_rev(f);
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, position: $position:expr $(, $($rest:tt)*)?) => {
        fn position(&$self) -> carbide::draw::Position {
            $position
        }

        fn set_position(&mut $self, position: carbide::draw::Position) {
            $position = position;
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, dimension: $dimension:expr $(, $($rest:tt)*)?) => {
        fn dimension(&$self) -> carbide::draw::Dimension {
            $dimension
        }

        fn set_dimension(&mut $self, dimension: carbide::draw::Dimension) {
            $dimension = dimension
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, flag: $flag:expr $(, $($rest:tt)*)?) => {
        fn flag(&$self) -> carbide::flags::WidgetFlag {
            $flag
        }

        $(CommonWidgetImpl!($self, $($rest)*);)?
    };

    ($self:ident, flexibility: $flexibility:expr $(, $($rest:tt)*)?) => {
        fn flexibility(&$self) -> u32 {
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

        fn flexibility(&$self) -> u32 {
            $child.flexibility()
        }

        fn dimension(&$self) -> $crate::draw::Dimension {
            $child.dimension()
        }

        fn set_dimension(&mut $self, dimension: $crate::draw::Dimension) {
            $child.set_dimension(dimension);
        }

        #[allow(unused_imports)]
        fn foreach_child<'a>(&'a $self, f: &mut dyn FnMut(&'a dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_mut<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach_mut(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach_rev(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_direct<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach_direct(f);
        }

        #[allow(unused_imports)]
        fn foreach_child_direct_rev<'a>(&'a mut $self, f: &mut dyn FnMut(&'a mut dyn $crate::widget::AnyWidget)) {
            use $crate::widget::AnySequence;
            $child.foreach_direct_rev(f);
        }
    }
}
