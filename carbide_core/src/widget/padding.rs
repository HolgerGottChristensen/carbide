use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::widget::types::EdgeInsets;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Padding {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    edge_insets: EdgeInsets,
}

impl Padding {
    pub fn init(edge_insets: EdgeInsets, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Padding {
            id: Default::default(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            edge_insets,
        })
    }
}

impl CommonWidget for Padding {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
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
        let dimensions = Dimension::new(requested_size.width - self.edge_insets.left - self.edge_insets.right, requested_size.height - self.edge_insets.top - self.edge_insets.bottom);

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = Dimension::new(child_dimensions.width + self.edge_insets.left + self.edge_insets.right, child_dimensions.height + self.edge_insets.top + self.edge_insets.bottom);

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.positioner();
        let position = Position::new(self.x() + self.edge_insets.left, self.y() + self.edge_insets.top);
        let dimension = Dimension::new(self.width() - self.edge_insets.left - self.edge_insets.right, self.height() - self.edge_insets.top - self.edge_insets.bottom);

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl WidgetExt for Padding {}