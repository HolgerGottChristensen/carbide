use crate::window::Window;
use carbide_core::accessibility::Accessibility;
use carbide_core::draw::Dimension;
use carbide_core::focus::Focusable;
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::lifecycle::{Update, UpdateContext};
use carbide_core::scene::AnyScene;
use carbide_core::state::ReadState;
use carbide_core::widget::{AnyWidget, Widget, WidgetExt, WidgetSync};

impl<T: ReadState<T=String>, C: Widget> WidgetSync for Window<T, C> {}

impl<T: ReadState<T=String>, C: Widget> Focusable for Window<T, C> {}

impl<T: ReadState<T=String>, C: Widget> Layout for Window<T, C> {
    fn calculate_size(&mut self, _requested_size: Dimension, _ctx: &mut LayoutContext) -> Dimension {
        Dimension::new(0.0, 0.0)
    }

    fn position_children(&mut self, _ctx: &mut LayoutContext) {}
}

impl<T: ReadState<T=String>, C: Widget> Update for Window<T, C> {
    fn update(&mut self, _ctx: &mut UpdateContext) {}

    fn process_update(&mut self, _ctx: &mut UpdateContext) {}
}

impl<T: ReadState<T=String>, C: Widget> AnyWidget for Window<T, C> {
    fn as_widget(&self) -> &dyn AnyWidget {
        self
    }

    fn as_widget_mut(&mut self) -> &mut dyn AnyWidget {
        self
    }
}

impl<T: ReadState<T=String>, C: Widget> WidgetExt for Window<T, C> {}

impl<T: ReadState<T=String>, C: Widget> AnyScene for Window<T, C> {
    /// Request the window to redraw next frame
    fn request_redraw(&self) -> bool {
        match self {
            Window::UnInitialized { .. } => false,
            Window::Initialized(initialized) => {
                initialized.inner.request_redraw();
                true
            }
            Window::Failed => false,
        }
    }

    fn has_application_focus(&self) -> bool {
        match self {
            Window::UnInitialized { .. } => false,
            Window::Initialized(initialized) => {
                initialized.inner.has_focus()
            }
            Window::Failed => false,
        }
    }

    fn is_daemon(&self) -> bool {
        match self {
            Window::UnInitialized { .. } => true,
            Window::Initialized(initialized) => {
                false
            }
            Window::Failed => true
        }
    }
}