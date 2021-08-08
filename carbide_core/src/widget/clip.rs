use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
//#[render(process_get_primitives)]
pub struct Clip {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl Clip {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Clip {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }

    /*pub fn body(&mut self) -> Box<Self> {
        widget_body!(
            HStack (
                alignment: Alignment::Top,
                spacing: 10.0,
            ) {
                for i in $self.model {
                    Text("Item: {}, index: {}", $item, $index),
                }
            }
        )
    }*/
}

impl Layout for Clip {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.child.calculate_size(requested_size, env);
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        self.child.position_children();
    }
}

impl CommonWidget for Clip {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
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
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl Render for Clip {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        primitives.push(Primitive {
            kind: PrimitiveKind::Clip,
            rect: Rect::new(self.position, self.dimension),
        });

        for child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::UnClip,
            rect: Rect::new(self.position, self.dimension),
        });
    }
}

impl WidgetExt for Clip {}