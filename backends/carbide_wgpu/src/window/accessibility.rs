use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
use smallvec::SmallVec;
use carbide_core::accessibility::{Accessibility, AccessibilityContext};
use carbide_core::state::ReadState;
use carbide_core::widget::{Identifiable, Widget, WidgetId};
use crate::Window;

impl<T: ReadState<T=String>, C: Widget> Accessibility for Window<T, C> {
    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        match self {
            Window::UnInitialized { .. } => {}
            Window::Initialized(initialized) => {
                let id = initialized.id;
                if ctx.parent_id.is_none() {
                    initialized.with_env(ctx.env, |env, initialized| {
                        initialized.accessibility_adapter.update_if_active(|| {
                            let mut tree_update = TreeUpdate {
                                nodes: vec![],
                                tree: Some(Tree {
                                    root: NodeId(id.as_u32() as u64),
                                    app_name: None,
                                    toolkit_name: Some("Carbide".to_string()),
                                    toolkit_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                                }),
                                focus: NodeId(id.as_u32() as u64),
                            };

                            let mut children = SmallVec::<[WidgetId; 8]>::new();

                            initialized.child.process_accessibility(&mut AccessibilityContext {
                                env,
                                nodes: &mut tree_update,
                                parent_id: Some(id),
                                children: &mut children,
                                hidden: false,
                                inherited_label: None,
                                inherited_hint: None,
                                inherited_value: None,
                                inherited_enabled: None,
                            });

                            let mut node_builder = Node::new(Role::Window);

                            node_builder.set_children(children.into_iter().map(|id| NodeId(id.as_u32() as u64)).collect::<Vec<_>>());

                            node_builder.set_label(initialized.title.value().clone());

                            tree_update.nodes.push((NodeId(id.as_u32() as u64), node_builder));

                            //println!("{:#?}", tree_update);

                            tree_update
                        })
                    });
                } else {
                    let mut children = SmallVec::<[WidgetId; 8]>::new();

                    initialized.with_env(ctx.env, |env, initialized| {
                        initialized.child.process_accessibility(&mut AccessibilityContext {
                            env,
                            nodes: ctx.nodes,
                            parent_id: Some(initialized.id),
                            children: &mut children,
                            hidden: false,
                            inherited_label: None,
                            inherited_hint: None,
                            inherited_value: None,
                            inherited_enabled: None,
                        })
                    });

                    let mut node_builder = Node::new(Role::Window);

                    node_builder.set_children(children.into_iter().map(|id| NodeId(id.as_u32() as u64)).collect::<Vec<_>>());

                    node_builder.set_label("Test window name");

                    ctx.nodes.push(self.id(), node_builder);
                }

                ctx.children.push(self.id());
            }
            Window::Failed => {}
        }
    }
}