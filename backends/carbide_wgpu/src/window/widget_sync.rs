use accesskit::{NodeBuilder, NodeId, Role, Tree, TreeUpdate};
use smallvec::SmallVec;
use carbide_core::accessibility::{Accessibility, AccessibilityContext};
use carbide_core::draw::Dimension;
use carbide_core::event::{AccessibilityEventHandler, EventHandler, KeyboardEventHandler, MouseEventHandler, OtherEventHandler, WindowEventHandler};
use carbide_core::focus::Focusable;
use carbide_core::layout::{Layout, LayoutContext};
use carbide_core::lifecycle::{Update, UpdateContext};
use carbide_core::Scene;
use carbide_core::state::ReadState;
use carbide_core::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId, WidgetSync};
use crate::window::Window;

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

impl<T: ReadState<T=String>, C: Widget> Accessibility for Window<T, C> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        match self {
            Window::UnInitialized { .. } => {}
            Window::Initialized(initialized) => {
                let mut tree = TreeUpdate {
                    nodes: vec![],
                    tree: Some(Tree {
                        root: NodeId(initialized.id.0 as u64),
                        app_name: None,
                        toolkit_name: Some("Carbide".to_string()),
                        toolkit_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    }),
                    focus: NodeId(initialized.id.0 as u64),
                };

                let mut children = SmallVec::<[WidgetId; 8]>::new();

                initialized.child.process_accessibility(&mut AccessibilityContext {
                    env: ctx.env,
                    tree: &mut tree,
                    parent_id: initialized.id,
                    children: &mut children,
                    hidden: false,
                });

                let mut node_builder = NodeBuilder::new(Role::Window);

                node_builder.set_children(children.into_iter().map(|id| NodeId(id.0 as u64)).collect::<Vec<_>>());

                node_builder.set_name(initialized.title.value().clone());
            }
            Window::Failed => {}
        }
    }
}

impl<T: ReadState<T=String>, C: Widget> AnyWidget for Window<T, C> {}

impl<T: ReadState<T=String>, C: Widget> WidgetExt for Window<T, C> {}

impl<T: ReadState<T=String>, C: Widget> Scene for Window<T, C> {
    /// Request the window to redraw next frame
    fn request_redraw(&self) {
        match self {
            Window::UnInitialized { .. } => {}
            Window::Initialized(initialized) => {
                initialized.inner.request_redraw();
            }
            Window::Failed => {}
        }
    }
}