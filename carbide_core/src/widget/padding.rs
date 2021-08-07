use crate::prelude::*;
use crate::render::ChildRender;
use crate::widget::types::EdgeInsets;

#[derive(Debug, Clone, Widget)]
pub struct Padding {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions,
    edge_insets: EdgeInsets,
}

impl Padding {
    pub fn init(edge_insets: EdgeInsets, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Padding {
            id: Default::default(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            edge_insets,
        })
    }
}

impl CommonWidget for Padding {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }


    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl Layout for Padding {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        let dimensions = [requested_size[0] - self.edge_insets.left - self.edge_insets.right, requested_size[1] - self.edge_insets.top - self.edge_insets.bottom];

        let child_dimensions = self.child.calculate_size(dimensions, env);

        self.dimension = [child_dimensions[0] + self.edge_insets.left + self.edge_insets.right, child_dimensions[1] + self.edge_insets.top + self.edge_insets.bottom];

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = [self.position[0] + self.edge_insets.left, self.position[1] + self.edge_insets.top];
        let dimension = [self.dimension[0] - self.edge_insets.left - self.edge_insets.right, self.dimension[1] - self.edge_insets.top - self.edge_insets.bottom];

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl ChildRender for Padding {}

impl WidgetExt for Padding {}