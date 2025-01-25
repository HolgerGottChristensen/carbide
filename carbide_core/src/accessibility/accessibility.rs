use crate::environment::{EnvironmentStack};
use crate::focus::{Focus, Focusable};
use crate::widget::{CommonWidget, WidgetId, WidgetSync};
use accesskit::{Action, Node, NodeId, Point, Rect, Role, Size, TreeUpdate};
use smallvec::SmallVec;
use carbide::accessibility::AccessibilityNode;
use crate::accessibility::AccessibilityAction;
use crate::scene::SceneManager;

pub trait Accessibility: Focusable + CommonWidget + WidgetSync {
    fn role(&self) -> Option<Role> { None }

    #[allow(unused_variables)]
    fn accessibility(&mut self, node: &mut Node) {}

    fn accessibility_create_node(&mut self, ctx: &mut AccessibilityContext) -> Option<Node> {
        if let Some(role) = self.role() {
            let mut node = Node::new(role);

            node.set_author_id(format!("{:?}", self.id()));

            let scale_factor = ctx.env_stack.get_mut::<SceneManager>()
                .map(|a| a.scale_factor())
                .unwrap_or(1.0);

            node.set_bounds(Rect::from_origin_size(
                Point::new(self.x() * scale_factor, self.y() * scale_factor),
                Size::new(self.width() * scale_factor, self.height() * scale_factor),
            ));

            if ctx.hidden {
                node.set_hidden();
            }

            if self.is_focusable() && self.get_focus() != Focus::Focused {
                node.add_action(AccessibilityAction::Focus);
            }

            if self.is_focusable() && self.get_focus() == Focus::Focused {
                node.add_action(AccessibilityAction::Blur);
            }

            if let Some(label) = ctx.inherited_label {
                node.set_label(label);
            }

            if let Some(hint) = ctx.inherited_hint {
                node.set_description(hint);
            }

            if let Some(hint) = ctx.inherited_value {
                node.set_value(hint);
            }

            if let Some(enabled) = ctx.inherited_enabled {
                if !enabled {
                    node.set_disabled();
                }
            }

            Some(node)
        } else {
            None
        }
    }

    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        self.sync(ctx.env_stack);

        /*if self.has_focus() {
            ctx.nodes.focus = NodeId(self.id().0 as u64);
        }*/

        // Check if this widget specifies a role, meaning it will
        // participate in the accessibility tree.
        if let Some(mut node) = self.accessibility_create_node(ctx) {
            let mut children = SmallVec::<[WidgetId; 8]>::new();

            let mut child_ctx = AccessibilityContext {
                env_stack: ctx.env_stack,
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

            self.accessibility(&mut node);

            node.set_children(
                children.into_iter()
                    .map(|a| NodeId(a.as_u32() as u64))
                    .collect::<Vec<_>>()
            );

            node.set_author_id(format!("{:?}", self.id()));

            ctx.nodes.push(self.id(), node);

            ctx.children.push(self.id());
        } else {
            // Delegate the accessibility to the children
            self.foreach_child_direct(&mut |child | {
                child.process_accessibility(ctx);
            })
        }
    }
}

pub struct AccessibilityContext<'a, 'b: 'a> {
    pub env_stack: &'a mut EnvironmentStack<'b>,
    pub nodes: &'a mut dyn AccessibilityUpdate,
    pub parent_id: Option<WidgetId>,
    pub children: &'a mut SmallVec<[WidgetId; 8]>,

    // TODO: The below could possibly all be in env stack
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
        self.nodes.push((NodeId(id.as_u32() as u64), node));
    }
}

impl AccessibilityUpdate for SmallVec<[AccessibilityNode; 1]> {
    fn push(&mut self, _id: WidgetId, node: AccessibilityNode) {
        self.push(node);
    }
}
