use accesskit::{Action, Node, NodeBuilder, NodeId, Point, Rect, Role, Size, TreeUpdate};
use smallvec::SmallVec;
use carbide::environment::Environment;
use carbide::focus::Focus;
use carbide::widget::{CommonWidget, WidgetId, WidgetSync};
use crate::focus::Focusable;

pub trait Accessibility: Focusable + CommonWidget + WidgetSync {
    fn role(&self) -> Option<Role> { None }

    #[allow(unused_variables)]
    fn accessibility(&mut self, builder: &mut NodeBuilder, env: &mut Environment) {}

    fn process_accessibility(&mut self, ctx: &mut AccessibilityContext) {
        let needs_accessibility = ctx.env.accessibility_needs_update(self.id());
        let parent_needs_accessibility = ctx.env.accessibility_needs_update(ctx.parent_id);

        if self.has_focus() {
            ctx.tree.focus = NodeId(self.id().0 as u64);
        }

        // Check if this widget specifies a role, meaning it will
        // participate in the accessibility tree.
        if let Some(role) = self.role() {
            let mut children = SmallVec::<[WidgetId; 8]>::new();

            let mut child_ctx = AccessibilityContext {
                env: ctx.env,
                tree: ctx.tree,
                parent_id: self.id(),
                children: &mut children,
                hidden: ctx.hidden,
            };

            // Process the accessibility of the children
            self.foreach_child_direct(&mut |child | {
                child.process_accessibility(&mut child_ctx);
            });

            // If this widget requested accessibility
            if needs_accessibility {
                let mut builder = NodeBuilder::new(role);

                builder.set_bounds(Rect::from_origin_size(
                    Point::new(self.x(), self.y()),
                    Size::new(self.width(), self.height()),
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

                builder.set_children(
                    children.into_iter()
                        .map(|a| NodeId(a.0 as u64))
                        .collect::<Vec<_>>()
                );

                ctx.tree.nodes.push((NodeId(self.id().0 as u64), builder.build()));
            }

            if parent_needs_accessibility {
                ctx.children.push(self.id());
            }
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
    pub tree: &'a mut TreeUpdate,
    pub parent_id: WidgetId,
    pub children: &'a mut SmallVec<[WidgetId; 8]>,
    pub hidden: bool,
}