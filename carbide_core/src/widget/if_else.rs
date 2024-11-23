use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position};
use crate::flags::WidgetFlag;
use crate::state::ReadState;
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetExt, WidgetId};

/// # If-Else Widget
///
/// Show different widgets based on a predicate. The predicate must be some state that can be read
/// from. Since the `IfElse` will not modify the state, a simple read state is sufficient.
/// ```rust
/// use carbide_core::environment::EnvironmentColor;
/// use carbide_core::widget::{IfElse, Rectangle};
///
/// IfElse::new(true)
///     .when_true(Rectangle::new().fill(EnvironmentColor::Green))
///     .when_false(Rectangle::new().fill(EnvironmentColor::Red));
/// ```
/// In the above a green rectangle will be displayed, since the state is a constant true.
#[derive(Debug, Clone, Widget)]
pub struct IfElse<T, F, S> where
    T: Widget,
    F: Widget,
    S: ReadState<T=bool>
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] predicate: S,
    when_true: T,
    when_false: F,
}

impl IfElse<Empty, Empty, bool> {
    #[carbide_default_builder2]
    pub fn new<S: ReadState<T=bool> + Clone + 'static>(predicate: S) -> IfElse<Empty, Empty, S> {
        IfElse {
            id: WidgetId::new(),
            predicate,
            when_true: Empty::new(),
            when_false: Empty::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
        }
    }

}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static> IfElse<T, F, S> {
    pub fn when_true<T2: Widget>(self, when_true: T2) -> IfElse<T2, F, S> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true,
            when_false: self.when_false,
            position: self.position,
            dimension: self.dimension,
        }
    }

    pub fn when_false<F2: Widget>(self, when_false: F2) -> IfElse<T, F2, S> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_true,
            when_false,
            position: self.position,
            dimension: self.dimension,
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static> CommonWidget for IfElse<T, F, S> {
    fn flag(&self) -> WidgetFlag {
        WidgetFlag::PROXY
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                return;
            }

            if self.when_true.is_proxy() {
                self.when_true.foreach_child(f);
                return;
            }

            f(&self.when_true)
        } else {
            if self.when_false.is_ignore() {
                return;
            }

            if self.when_false.is_proxy() {
                self.when_false.foreach_child(f);
                return;
            }

            f(&self.when_false)
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                return;
            }

            if self.when_true.is_proxy() {
                self.when_true.foreach_child_mut(f);
                return;
            }

            f(&mut self.when_true)
        } else {
            if self.when_false.is_ignore() {
                return;
            }

            if self.when_false.is_proxy() {
                self.when_false.foreach_child_mut(f);
                return;
            }

            f(&mut self.when_false)
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                return;
            }

            if self.when_true.is_proxy() {
                self.when_true.foreach_child_rev(f);
                return;
            }

            f(&mut self.when_true)
        } else {
            if self.when_false.is_ignore() {
                return;
            }

            if self.when_false.is_proxy() {
                self.when_false.foreach_child_rev(f);
                return;
            }

            f(&mut self.when_false)
        }
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if *self.predicate.value() {
            f(&mut self.when_true)
        } else {
            f(&mut self.when_false)
        }
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if *self.predicate.value() {
            f(&mut self.when_true)
        } else {
            f(&mut self.when_false)
        }
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}