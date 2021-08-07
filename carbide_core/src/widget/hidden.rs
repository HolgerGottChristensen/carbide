use crate::prelude::*;
use crate::render::ChildRender;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[render(process_get_primitives)]
pub struct Hidden {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions,
}

impl Hidden {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Hidden {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }

    fn process_get_primitives(&mut self, _: &mut Vec<Primitive>, _: &mut Environment) {}
}

impl Layout for Hidden {
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

        self.child.position_children();
    }
}

impl CommonWidget for Hidden {
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

impl ChildRender for Hidden {}

impl WidgetExt for Hidden {}