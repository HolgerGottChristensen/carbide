use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position};
use crate::common::flags::WidgetFlag;
use crate::state::ReadState;
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetId, WidgetProperties};
use crate::widget::properties::WidgetKindProxy;

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
// TODO: It could be possible to make this type generic over the WidgetKind, and allow it to be simple when its true and false branch widgets are.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Properties)]
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

    fn child(&self, index: usize) -> &dyn AnyWidget {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                panic!("The child is ignore, and thus can not be indexed.");
            }

            if self.when_true.is_proxy() {
                // Pass the index directly to the proxy child.
                return self.when_true.child(index);
            }

            // If the child is neither an ignore nor a proxy, we have only a single child.
            // We thus expect the index to be 0, otherwise we panic.
            if index != 0 {
                panic!("The index was not within the correct bounds.")
            }

            &self.when_true
        } else {
            if self.when_false.is_ignore() {
                panic!("The child is ignore, and thus can not be indexed.");
            }

            if self.when_false.is_proxy() {
                // Pass the index directly to the proxy child.
                return self.when_false.child(index);
            }

            // If the child is neither an ignore nor a proxy, we have only a single child.
            // We thus expect the index to be 0, otherwise we panic.
            if index != 0 {
                panic!("The index was not within the correct bounds.")
            }

            &self.when_false
        }
    }

    fn child_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                panic!("The child is ignore, and thus can not be indexed.");
            }

            if self.when_true.is_proxy() {
                // Pass the index directly to the proxy child.
                return self.when_true.child_mut(index);
            }

            // If the child is neither an ignore nor a proxy, we have only a single child.
            // We thus expect the index to be 0, otherwise we panic.
            if index != 0 {
                panic!("The index was not within the correct bounds.")
            }

            &mut self.when_true
        } else {
            if self.when_false.is_ignore() {
                panic!("The child is ignore, and thus can not be indexed.");
            }

            if self.when_false.is_proxy() {
                // Pass the index directly to the proxy child.
                return self.when_false.child_mut(index);
            }

            // If the child is neither an ignore nor a proxy, we have only a single child.
            // We thus expect the index to be 0, otherwise we panic.
            if index != 0 {
                panic!("The index was not within the correct bounds.")
            }

            &mut self.when_false
        }
    }

    fn child_count(&self) -> usize {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                return 0;
            }

            if self.when_true.is_proxy() {
                return self.when_true.child_count();
            }

            // If the child is neither an ignore nor a proxy, we have only a single child.
            1
        } else {
            if self.when_false.is_ignore() {
                return 0;
            }

            if self.when_false.is_proxy() {
                return self.when_false.child_count();
            }

            // If the child is neither an ignore nor a proxy, we have only a single child.
            1
        }
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn AnyWidget)) {
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

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
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

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
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

    fn foreach_child_direct(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        if *self.predicate.value() {
            f(&mut self.when_true)
        } else {
            f(&mut self.when_false)
        }
    }

    fn foreach_child_direct_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
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

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static> WidgetProperties for IfElse<T, F, S> {
    type Kind = WidgetKindProxy;
}