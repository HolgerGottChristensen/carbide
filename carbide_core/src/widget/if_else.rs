use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use carbide::widget::properties::WidgetKind;
use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position};
use crate::common::flags::WidgetFlag;
use crate::state::ReadState;
use crate::widget::{AnyWidget, CommonWidget, Empty, Widget, WidgetId, WidgetProperties};
use crate::widget::properties::{WidgetKindDynamic, WidgetKindIgnore, WidgetKindProxy, WidgetKindSimple};

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
#[derive(Widget)]
#[carbide_exclude(Properties)]
pub struct IfElse<T, F, S, K, KTrue, KFalse> where
    T: Widget,
    F: Widget,
    S: ReadState<T=bool>,
    K: WidgetKind,
    KTrue: WidgetKind,
    KFalse: WidgetKind
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state] predicate: S,
    when_true: T,
    when_false: F,

    phantom_data: PhantomData<K>,
    phantom_data_true: PhantomData<KTrue>,
    phantom_data_false: PhantomData<KFalse>,
}

impl IfElse<Empty, Empty, bool, WidgetKindSimple, WidgetKindSimple, WidgetKindSimple> {
    pub fn new<S: ReadState<T=bool> + Clone + 'static>(predicate: S) -> IfElse<Empty, Empty, S, WidgetKindProxy, WidgetKindIgnore, WidgetKindIgnore> { // TODO: Not actually simple
        IfElse {
            id: WidgetId::new(),
            predicate,
            when_true: Empty::new(),
            when_false: Empty::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }

}

// If the true is simple, we only care about the kind of the new true
impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind> IfElse<T, F, S, K, KTrue, WidgetKindSimple> {
    pub fn when_true<T2: Widget>(self, when_true: T2) -> IfElse<T2, F, S, T2::Kind, T2::Kind, WidgetKindSimple> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true,
            when_false: self.when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KFalse: WidgetKind> IfElse<T, F, S, K, WidgetKindSimple, KFalse> {
    pub fn when_false<F2: Widget>(self, when_false: F2) -> IfElse<T, F2, S, F2::Kind, WidgetKindSimple, F2::Kind> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_true,
            when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind> IfElse<T, F, S, K, KTrue, WidgetKindProxy> {
    pub fn when_true<T2: Widget>(self, when_true: T2) -> IfElse<T2, F, S, WidgetKindProxy, T2::Kind, WidgetKindSimple> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true,
            when_false: self.when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KFalse: WidgetKind> IfElse<T, F, S, K, WidgetKindProxy, KFalse> {
    pub fn when_false<F2: Widget>(self, when_false: F2) -> IfElse<T, F2, S, WidgetKindProxy, WidgetKindProxy, F2::Kind> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_true,
            when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind> IfElse<T, F, S, K, KTrue, WidgetKindIgnore> {
    pub fn when_true<T2: Widget>(self, when_true: T2) -> IfElse<T2, F, S, WidgetKindProxy, T2::Kind, WidgetKindSimple> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true,
            when_false: self.when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KFalse: WidgetKind> IfElse<T, F, S, K, WidgetKindIgnore, KFalse> {
    pub fn when_false<F2: Widget>(self, when_false: F2) -> IfElse<T, F2, S, WidgetKindProxy, WidgetKindIgnore, F2::Kind> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_true,
            when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind> IfElse<T, F, S, K, KTrue, WidgetKindDynamic> {
    pub fn when_true<T2: Widget>(self, when_true: T2) -> IfElse<T2, F, S, WidgetKindProxy, T2::Kind, WidgetKindSimple> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true,
            when_false: self.when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KFalse: WidgetKind> IfElse<T, F, S, K, WidgetKindDynamic, KFalse> {
    pub fn when_false<F2: Widget>(self, when_false: F2) -> IfElse<T, F2, S, WidgetKindProxy, WidgetKindDynamic, F2::Kind> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_true,
            when_false,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind, KFalse: WidgetKind> IfElse<T, F, S, K, KTrue, KFalse> {
    pub fn inverse(self) -> IfElse<F, T, S, K, KFalse, KTrue> {
        IfElse {
            id: self.id,
            predicate: self.predicate,
            when_true: self.when_false,
            when_false: self.when_true,
            position: self.position,
            dimension: self.dimension,
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind, KFalse: WidgetKind> CommonWidget for IfElse<T, F, S, K, KTrue, KFalse> {
    fn flag(&self) -> WidgetFlag {
        WidgetFlag::PROXY
    }

    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
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

            &mut self.when_true
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

            &mut self.when_false
        }
    }

    fn child_count(&mut self) -> usize {
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

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        if *self.predicate.value() {
            if self.when_true.is_ignore() {
                return;
            }

            if self.when_true.is_proxy() {
                self.when_true.foreach_child(f);
                return;
            }

            f(&mut self.when_true)
        } else {
            if self.when_false.is_ignore() {
                return;
            }

            if self.when_false.is_proxy() {
                self.when_false.foreach_child(f);
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

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind, KFalse: WidgetKind> WidgetProperties for IfElse<T, F, S, K, KTrue, KFalse> {
    type Kind = K;
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind, KFalse: WidgetKind> Debug for IfElse<T, F, S, K, KTrue, KFalse> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IfElse")
            .field("id", &self.id)
            .field("predicate", &self.predicate)
            .field("when_true", &self.when_true)
            .field("when_false", &self.when_false)
            .finish_non_exhaustive()
    }
}

impl<T: Widget, F: Widget, S: ReadState<T=bool> + Clone + 'static, K: WidgetKind, KTrue: WidgetKind, KFalse: WidgetKind> Clone for IfElse<T, F, S, K, KTrue, KFalse> {
    fn clone(&self) -> Self {
        IfElse {
            id: self.id.clone(),
            position: self.position.clone(),
            dimension: self.dimension.clone(),
            predicate: self.predicate.clone(),
            when_true: self.when_true.clone(),
            when_false: self.when_false.clone(),
            phantom_data: Default::default(),
            phantom_data_true: Default::default(),
            phantom_data_false: Default::default(),
        }
    }
}