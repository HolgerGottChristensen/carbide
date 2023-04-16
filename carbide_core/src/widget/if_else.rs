use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::flags::Flags;
use crate::state::{ReadState, RState};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, WidgetIterMut};

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
pub struct IfElse<T, F> where
    T: Widget + Clone,
    F: Widget + Clone,
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] predicate: RState<bool>,
    when_true: T,
    when_false: F,
}

impl IfElse<Empty, Empty> {
    #[carbide_default_builder]
    pub fn new(predicate: impl Into<RState<bool>>) -> Box<IfElse<Empty, Empty>> {}

    pub fn new(predicate: impl Into<RState<bool>>) -> Box<IfElse<Empty, Empty>> {
        Box::new(IfElse {
            id: WidgetId::new(),
            predicate: predicate.into(),
            when_true: *Empty::new(),
            when_false: *Empty::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
        })
    }
}

impl<T: Widget + Clone, F: Widget + Clone> IfElse<T, F> {
    pub fn when_true<T2: Widget + Clone>(self, when_true: T2) -> Box<IfElse<T2, F>> {
        Box::new(IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true,
            when_false: self.when_false,
            position: self.position,
            dimension: self.dimension,
        })
    }

    pub fn when_false<F2: Widget + Clone>(self, when_false: F2) -> Box<IfElse<T, F2>> {
        Box::new(IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_true,
            when_false,
            position: self.position,
            dimension: self.dimension,
        })
    }
}

impl<T: Widget + Clone, F: Widget + Clone> CommonWidget for IfElse<T, F> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        Flags::PROXY
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn Widget)) {
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

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn Widget)) {
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

    fn children_mut(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            if self.when_true.flag() == Flags::PROXY {
                self.when_true.children_mut()
            } else if self.when_true.flag() == Flags::IGNORE {
                WidgetIterMut::Empty
            } else {
                WidgetIterMut::owned(Box::new(self.when_true.clone()))
            }
        } else {
            if self.when_false.flag() == Flags::PROXY {
                self.when_false.children_mut()
            } else if self.when_false.flag() == Flags::IGNORE {
                WidgetIterMut::Empty
            } else {
                WidgetIterMut::owned(Box::new(self.when_false.clone()))
            }
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            WidgetIterMut::owned(Box::new(self.when_true.clone()))
        } else {
            WidgetIterMut::owned(Box::new(self.when_false.clone()))
        }
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            WidgetIterMut::owned(Box::new(self.when_true.clone()))
        } else {
            WidgetIterMut::owned(Box::new(self.when_false.clone()))
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

impl<T: Widget + Clone, F: Widget + Clone> WidgetExt for IfElse<T, F> {}
