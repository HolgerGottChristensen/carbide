mod common_widget;
mod initialize;
mod widget_sync;
mod util;
mod event_handlers;
mod render;
mod initialized_window;
mod accessibility;

use crate::window::initialized_window::InitializedWindow;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::{EnvironmentColor, EnvironmentStack};
use carbide_core::state::{IntoReadState, ReadState};
use carbide_core::widget::{AnyWidget, CommonWidget, Empty, IntoWidget, NavigationStack, Overlay, Rectangle, Widget, WidgetExt, WidgetId, ZStack};
use std::fmt::{Debug, Formatter};
use carbide_core::draw::theme::Theme;
use carbide_core::widget::managers::{FontSizeManager, ThemeManager};

pub enum Window<T: ReadState<T=String>, C: Widget> {
    UnInitialized {
        id: WidgetId,
        title: T,
        position: Position,
        dimension: Dimension,
        child: C,
        close_application_on_window_close: bool,
    },
    Initialized(InitializedWindow<T, C>),
    Failed
}

impl Window<String, Empty> {
    pub fn new<T: IntoReadState<String>, C: IntoWidget>(title: T, dimension: Dimension, child: C) -> Window<T::Output, impl Widget> {

        let child = child.into_widget();

        #[cfg(feature = "controls")]
        let child = carbide_controls::controls_overlay(child);

        let child = ZStack::new((
            Rectangle::new().fill(EnvironmentColor::SystemBackground),
            child
        ));

        let child = NavigationStack::new_root(child);

        let child = ThemeManager::new(child);

        let child = FontSizeManager::new(child);

        Window::UnInitialized {
            id: WidgetId::new(),
            title: title.into_read_state(),
            position: Default::default(),
            dimension,
            child,
            close_application_on_window_close: false,
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> Window<T, C> {
    pub fn close_application_on_window_close(mut self) -> Self {
        match &mut self {
            Window::UnInitialized { close_application_on_window_close, .. } => *close_application_on_window_close = true,
            Window::Initialized(initialized) => initialized.close_application_on_window_close = true,
            Window::Failed => {}
        }

        self
    }
}

impl<T: ReadState<T=String>, C: Widget> Clone for Window<T, C> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T: ReadState<T=String>, C: Widget> Debug for Window<T, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("position", &self.position())
            .field("dimension", &self.dimension())
            .field("child", match self {
                Window::UnInitialized { child, .. } => child,
                Window::Initialized (initialized) => &initialized.child,
                Window::Failed => &"Failed",
            })
            .finish()
    }
}