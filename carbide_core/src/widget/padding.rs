use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};
use crate::widget::types::EdgeInsets;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Padding {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    edge_insets: EdgeInsets,
}

impl Padding {
    #[carbide_default_builder]
    pub fn new(edge_insets: impl Into<EdgeInsets>, child: Box<dyn Widget>) -> Box<Self> {}

    pub fn new(edge_insets: impl Into<EdgeInsets>, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Padding {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            edge_insets: edge_insets.into(),
        })
    }
}

impl CommonWidget for Padding {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }


    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        Dimension::new(self.dimension.width.abs(), self.dimension.height.abs())
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl Layout for Padding {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let dimensions = Dimension::new(
            requested_size.width - self.edge_insets.left - self.edge_insets.right,
            requested_size.height - self.edge_insets.top - self.edge_insets.bottom,
        );

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = Dimension::new(
            child_dimensions.width + self.edge_insets.left + self.edge_insets.right,
            child_dimensions.height + self.edge_insets.top + self.edge_insets.bottom,
        );

        self.dimension
    }

    fn position_children(&mut self, env: &mut Environment) {
        let positioning = BasicLayouter::Center.positioner();
        let position = Position::new(
            self.x() + self.edge_insets.left,
            self.y() + self.edge_insets.top,
        );
        let dimension = Dimension::new(
            self.width() - self.edge_insets.left - self.edge_insets.right,
            self.height() - self.edge_insets.top - self.edge_insets.bottom,
        );

        positioning(position, dimension, &mut self.child);
        self.child.position_children(env);
    }
}

impl WidgetExt for Padding {}
