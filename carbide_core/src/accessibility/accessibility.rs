use crate::environment::Environment;
use crate::focus::{Focus, Focusable};
use crate::widget::{CommonWidget, WidgetId, WidgetSync};
use accesskit::{Action, NodeBuilder, NodeId, Point, Rect, Role, Size, TreeUpdate};
use smallvec::SmallVec;
use carbide::accessibility::AccessibilityNode;

pub trait Accessibility: Focusable + CommonWidget + WidgetSync {
    fn role(&self) -> Option<Role> { None }

    #[allow(unused_variables)]
    fn accessibility(&mut self, builder: &mut NodeBuilder, env: &mut Environment) {}

    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env);

        /*if self.has_focus() {
            ctx.nodes.focus = NodeId(self.id().0 as u64);
        }*/

        // Check if this widget specifies a role, meaning it will
        // participate in the accessibility tree.
        if let Some(role) = self.role() {
            let mut children = SmallVec::<[WidgetId; 8]>::new();

            let mut child_ctx = AccessibilityContext {
                env: ctx.env,
                nodes: ctx.nodes,
                parent_id: Some(self.id()),
                children: &mut children,
                hidden: ctx.hidden,
                inherited_label: None,
                inherited_hint: None,
                inherited_value: None,
                inherited_enabled: None,
            };

            // Process the accessibility of the children
            self.foreach_child_direct(&mut |child | {
                child.process_accessibility(&mut child_ctx);
            });

            let mut builder = NodeBuilder::new(role);

            builder.set_bounds(Rect::from_origin_size(
                Point::new(self.x() * ctx.env.scale_factor(), self.y() * ctx.env.scale_factor()),
                Size::new(self.width() * ctx.env.scale_factor(), self.height() * ctx.env.scale_factor()),
            ));

            if ctx.hidden {
                builder.set_hidden();
            }

            if self.is_focusable() {
                builder.add_action(Action::Focus);
            }

            if self.get_focus() == Focus::Focused {
                builder.add_action(Action::Blur);
            }

            if let Some(label) = ctx.inherited_label {
                builder.set_name(label);
            }

            if let Some(hint) = ctx.inherited_hint {
                builder.set_description(hint);
            }

            if let Some(hint) = ctx.inherited_value {
                builder.set_value(hint);
            }

            if let Some(enabled) = ctx.inherited_enabled {
                if !enabled {
                    builder.set_disabled();
                }
            }

            self.accessibility(&mut builder, ctx.env);

            builder.set_children(
                children.into_iter()
                    .map(|a| NodeId(a.0 as u64))
                    .collect::<Vec<_>>()
            );

            builder.set_author_id(format!("{:?}", self.id()));

            ctx.nodes.push(self.id(), builder.build());

            ctx.children.push(self.id());
        } else {
            // Delegate the accessibility to the children
            self.foreach_child_direct(&mut |child | {
                child.process_accessibility(ctx);
            })
        }
    }
}

pub struct AccessibilityContext<'a> {
    pub env: &'a mut Environment,
    pub nodes: &'a mut dyn AccessibilityUpdate,
    pub parent_id: Option<WidgetId>,
    pub children: &'a mut SmallVec<[WidgetId; 8]>,
    pub hidden: bool,
    pub inherited_label: Option<&'a str>,
    pub inherited_hint: Option<&'a str>,
    pub inherited_value: Option<&'a str>,
    pub inherited_enabled: Option<bool>,
}

pub trait AccessibilityUpdate {
    fn push(&mut self, id: WidgetId, node: AccessibilityNode);
}

impl AccessibilityUpdate for TreeUpdate {
    fn push(&mut self, id: WidgetId, node: AccessibilityNode) {
        self.nodes.push((NodeId(id.0 as u64), node));
    }
}

impl AccessibilityUpdate for SmallVec<[AccessibilityNode; 1]> {
    fn push(&mut self, _id: WidgetId, node: AccessibilityNode) {
        self.push(node);
    }
}
