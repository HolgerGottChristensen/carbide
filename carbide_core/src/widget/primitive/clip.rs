use crate::prelude::*;
use crate::render::primitive_kind::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[render(process_get_primitives)]
pub struct Clip {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions,
}

impl Clip {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Clip {
            id: Uuid::new_v4(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
        })
    }

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        primitives.push(Primitive {
            kind: PrimitiveKind::Clip,
            rect: OldRect::new(self.position, self.dimension),
        });

        for child in self.get_children_mut() {
            child.process_get_primitives(primitives, env);
        }

        primitives.push(Primitive {
            kind: PrimitiveKind::UnClip,
            rect: OldRect::new(self.position, self.dimension),
        });
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

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
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

impl Render for Clip {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        // Look in process_get_primitives
        return vec![];
    }
}

impl WidgetExt for Clip {}