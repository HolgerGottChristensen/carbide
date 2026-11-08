use crate::carbide_window::Window;
use crate::carbide_window::WindowState;
use carbide_core::draw::{Dimension, Position};
use carbide_core::identifiable::Identifiable;
use carbide_core::state::ReadState;
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetId};


impl<T: ReadState<T=String>, C: Widget> Identifiable for Window<T, C> {
    type Id = WidgetId;

    fn id(&self) -> WidgetId {
        match &self.state {
            WindowState::UnInitialized { id, .. } => *id,
            WindowState::Initialized(initialized) => initialized.id,
            WindowState::Failed => panic!("Failed")
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> CommonWidget for Window<T, C> {
    fn child(&mut self, _index: usize) -> &mut dyn AnyWidget {
        todo!()
    }

    fn child_count(&mut self) -> usize {
        todo!()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        let child = match &mut self.state {
            WindowState::UnInitialized { child, .. } => child,
            WindowState::Initialized(initialized) => &mut initialized.child,
            WindowState::Failed => panic!("Failed")
        };

        if child.is_ignore() {
            return;
        }

        if child.is_proxy() {
            child.foreach_child(f);
            return;
        }

        f(child)
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        let child = match &mut self.state {
            WindowState::UnInitialized { child, .. } => child,
            WindowState::Initialized(initialized) => &mut initialized.child,
            WindowState::Failed => panic!("Failed")
        };

        if child.is_ignore() {
            return;
        }

        if child.is_proxy() {
            child.foreach_child_rev(f);
            return;
        }

        f(child)
    }

    fn position(&self) -> Position {
        match &self.state {
            WindowState::UnInitialized { position, .. } => *position,
            WindowState::Initialized (initialized) => initialized.position,
            WindowState::Failed => panic!("Failed")
        }
    }

    fn set_position(&mut self, position: Position) {
        match &mut self.state {
            WindowState::UnInitialized { position: pos, .. } => *pos = position,
            WindowState::Initialized (initialized) => initialized.position = position,
            WindowState::Failed => panic!("Failed")
        };
    }

    fn dimension(&self) -> Dimension {
        match &self.state {
            WindowState::UnInitialized { dimension, .. } => *dimension,
            WindowState::Initialized (initialized) => initialized.dimension,
            WindowState::Failed => panic!("Failed")
        }
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        match &mut self.state {
            WindowState::UnInitialized { dimension: dim, .. } => *dim = dimension,
            WindowState::Initialized (initialized) => initialized.dimension = dimension,
            WindowState::Failed => panic!("Failed")
        };
    }
}