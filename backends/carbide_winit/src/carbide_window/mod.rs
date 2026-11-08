mod common_widget;
mod initialize;
mod widget_sync;
mod event_handlers;
mod render;
mod initialized_window;
mod accessibility;

use crate::carbide_window::initialized_window::InitializedWindow;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{IntoReadState, ReadState};
use carbide_core::widget::managers::{FontSizeManager, ThemeManager};
use carbide_core::widget::{CommonWidget, Empty, IntoWidget, NavigationStack, Rectangle, Widget, WidgetId, ZStack};
use std::fmt::{Debug, Formatter};

/// A window scene. Construct it with [`Window::new`]; its lifecycle state is internal.
pub struct Window<T: ReadState<T=String>, C: Widget> {
    state: WindowState<T, C>,
}

pub(crate) enum WindowState<T: ReadState<T=String>, C: Widget> {
    UnInitialized {
        id: WidgetId,
        title: T,
        position: Position,
        dimension: Dimension,
        child: C,
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
            Rectangle::new()
                .fill(EnvironmentColor::SystemBackground),
            child
        ));

        let child = NavigationStack::new_root(child);

        let child = ThemeManager::new(child);

        let child = FontSizeManager::new(child);

        Window {
            state: WindowState::UnInitialized {
                id: WidgetId::new(),
                title: title.into_read_state(),
                position: Default::default(),
                dimension,
                child,
            }
        }
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
            .field("child", match &self.state {
                WindowState::UnInitialized { child, .. } => child,
                WindowState::Initialized (initialized) => &initialized.child,
                WindowState::Failed => &"Failed",
            })
            .finish()
    }
}