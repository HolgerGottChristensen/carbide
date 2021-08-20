use crate::draw::{Dimension, Position, Rect};
use crate::focus::Focus;
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Blur {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl Blur {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Blur {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl CommonWidget for Blur {
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

    fn flexibility(&self) -> u32 {
        0
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl Render for Blur {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        for child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::Filter,
            rect: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for Blur {}
