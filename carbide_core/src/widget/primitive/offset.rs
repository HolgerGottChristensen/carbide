use crate::prelude::*;
use crate::widget::ChildRender;

#[derive(Debug, Clone, Widget)]
pub struct Offset {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions,
    #[state] offset_x: F64State,
    #[state] offset_y: F64State,
}

impl Offset {
    pub fn new(offset_x: F64State, offset_y: F64State, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Offset {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            offset_x,
            offset_y,
        })
    }
}

impl Layout for Offset {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        let mut child_position: Point = self.child.get_position();

        child_position[0] += *self.offset_x.value();
        child_position[1] += *self.offset_y.value();

        self.child.set_position(child_position);

        self.child.position_children();
    }
}

impl CommonWidget for Offset {
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
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl ChildRender for Offset {}

impl WidgetExt for Offset {}