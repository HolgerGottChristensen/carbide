use std::fmt::Debug;
use carbide::accessibility::Accessibility;
use carbide::draw::{Dimension, Position};
use carbide::event::{AccessibilityEventHandler, ApplicationEventHandler, KeyboardEventHandler, MouseEventHandler, OtherEventHandler, WindowEventHandler};
use carbide::focus::Focusable;
use carbide::identifiable::Identifiable;
use carbide::layout::Layout;
use carbide::lifecycle::{Initialize, Update};
use carbide::render::Render;
use carbide::state::LocalState;
use carbide::widget::{AnyWidget, CommonWidget, WidgetId, WidgetSync};
use crate::core::any_studio_widget::AnyStudioWidget;

#[derive(Clone, Debug)]
pub struct StudioWidget(LocalState<Box<dyn AnyStudioWidget>>);

impl MouseEventHandler for StudioWidget {}

impl CommonWidget for StudioWidget {
    fn child(&mut self, index: usize) -> &mut dyn AnyWidget {
        todo!()
    }

    fn child_count(&mut self) -> usize {
        todo!()
    }

    fn foreach_child(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        todo!()
    }

    fn foreach_child_rev(&mut self, f: &mut dyn FnMut(&mut dyn AnyWidget)) {
        todo!()
    }

    fn position(&self) -> Position {
        todo!()
    }

    fn set_position(&mut self, position: Position) {
        todo!()
    }

    fn dimension(&self) -> Dimension {
        todo!()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        todo!()
    }
}

impl Identifiable<Id=WidgetId> for StudioWidget {
    type Id = ();

    fn id(&self) -> Self::Id {
        todo!()
    }
}

impl WidgetSync for StudioWidget {}

impl Focusable for StudioWidget {}

impl KeyboardEventHandler for StudioWidget {}

impl WindowEventHandler for StudioWidget {}

impl ApplicationEventHandler for StudioWidget {}

impl AccessibilityEventHandler for StudioWidget {}

impl OtherEventHandler for StudioWidget {}

impl Initialize for StudioWidget {}

impl Update for StudioWidget {}

impl Accessibility for StudioWidget {}

impl Layout for StudioWidget {}

impl Render for StudioWidget {}

impl AnyWidget for StudioWidget {
    fn as_widget(&self) -> &dyn AnyWidget {
        todo!()
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        todo!()
    }
}

