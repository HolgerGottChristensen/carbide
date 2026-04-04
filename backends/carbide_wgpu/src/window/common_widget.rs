use crate::window::Window;
use carbide_core::draw::{Dimension, Position};
use carbide_core::identifiable::Identifiable;
use carbide_core::state::ReadState;
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetId};


impl<T: ReadState<T=String>, C: Widget> Identifiable for Window<T, C> {
    type Id = WidgetId;

    fn id(&self) -> WidgetId {
        match self {
            Window::UnInitialized { id, .. } => *id,
            Window::Initialized(initialized) => initialized.id,
            Window::Failed => panic!("Failed")
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> CommonWidget for Window<T, C> {
    fn child_mut(&mut self, index: usize) -> &mut dyn AnyWidget {
        todo!()
    }

    fn child_count(&mut self) -> usize {
        todo!()
    }

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        let child = match self {
            Window::UnInitialized { child, .. } => child,
            Window::Initialized(initialized) => &mut initialized.child,
            Window::Failed => panic!("Failed")
        };

        if child.is_ignore() {
            return;
        }

        if child.is_proxy() {
            child.foreach_child_mut(f);
            return;
        }
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        let child = match self {
            Window::UnInitialized { child, .. } => child,
            Window::Initialized(initialized) => &mut initialized.child,
            Window::Failed => panic!("Failed")
        };

        if child.is_ignore() {
            return;
        }

        if child.is_proxy() {
            child.foreach_child_rev(f);
            return;
        }
    }

    fn position(&self) -> Position {
        match self {
            Window::UnInitialized { position, .. } => *position,
            Window::Initialized (initialized) => initialized.position,
            Window::Failed => panic!("Failed")
        }
    }

    fn set_position(&mut self, position: Position) {
        match self {
            Window::UnInitialized { position: pos, .. } => *pos = position,
            Window::Initialized (initialized) => initialized.position = position,
            Window::Failed => panic!("Failed")
        };
    }

    fn dimension(&self) -> Dimension {
        match self {
            Window::UnInitialized { dimension, .. } => *dimension,
            Window::Initialized (initialized) => initialized.dimension,
            Window::Failed => panic!("Failed")
        }
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        match self {
            Window::UnInitialized { dimension: dim, .. } => *dim = dimension,
            Window::Initialized (initialized) => initialized.dimension = dimension,
            Window::Failed => panic!("Failed")
        };
    }
}