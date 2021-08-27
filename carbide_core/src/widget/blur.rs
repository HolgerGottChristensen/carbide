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
    filter_has_been_inserted: Option<u32>,
}

impl Blur {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Blur {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            filter_has_been_inserted: None,
        })
    }

    fn blur(radius: u32) -> ImageFilter {
        let radius = radius as i32;
        let div = (2 * radius + 1).pow(2);

        let mut entries = vec![];

        for radius_y in -radius..=radius {
            for radius_x in -radius..=radius {
                entries.push(ImageFilterValue::new(radius_x, radius_y, 1.0 / div as f32))
            }
        }

        ImageFilter { filter: entries }
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
        if self.filter_has_been_inserted == None {
            let filter_id = env.insert_filter(Blur::blur(7));
            self.filter_has_been_inserted = Some(filter_id);
        }

        /*[0.0, -3.0, -3.0, 1.0 / 4096.0],
                [0.0, -3.0, -2.0, 6.0 / 4096.0],
                [0.0, -3.0, -1.0, 15.0 / 4096.0],
                [0.0, -3.0, 0.0, 20.0 / 4096.0],
                [0.0, -3.0, 1.0, 15.0 / 4096.0],
                [0.0, -3.0, 2.0, 6.0 / 4096.0],
                [0.0, -3.0, 3.0, 1.0 / 4096.0],
                [0.0, -2.0, -3.0, 6.0 / 4096.0],
                [0.0, -2.0, -2.0, 36.0 / 4096.0],
                [0.0, -2.0, -1.0, 90.0 / 4096.0],
                [0.0, -2.0, 0.0, 120.0 / 4096.0],
                [0.0, -2.0, 1.0, 90.0 / 4096.0],
                [0.0, -2.0, 2.0, 36.0 / 4096.0],
                [0.0, -2.0, 3.0, 6.0 / 4096.0],
                [0.0, -1.0, -3.0, 15.0 / 4096.0],
                [0.0, -1.0, -2.0, 90.0 / 4096.0],
                [0.0, -1.0, -1.0, 225.0 / 4096.0],
                [0.0, -1.0, 0.0, 300.0 / 4096.0],
                [0.0, -1.0, 1.0, 225.0 / 4096.0],
                [0.0, -1.0, 2.0, 90.0 / 4096.0],
                [0.0, -1.0, 3.0, 15.0 / 4096.0],
                [0.0, 0.0, -3.0, 20.0 / 4096.0],
                [0.0, 0.0, -2.0, 120.0 / 4096.0],
                [0.0, 0.0, -1.0, 300.0 / 4096.0],
                [0.0, 0.0, 0.0, 400.0 / 4096.0],
                [0.0, 0.0, 1.0, 300.0 / 4096.0],
                [0.0, 0.0, 2.0, 120.0 / 4096.0],
                [0.0, 0.0, 3.0, 20.0 / 4096.0],
                [0.0, 1.0, -3.0, 15.0 / 4096.0],
                [0.0, 1.0, -2.0, 90.0 / 4096.0],
                [0.0, 1.0, -1.0, 225.0 / 4096.0],
                [0.0, 1.0, 0.0, 300.0 / 4096.0],
                [0.0, 1.0, 1.0, 225.0 / 4096.0],
                [0.0, 1.0, 2.0, 90.0 / 4096.0],
                [0.0, 1.0, 3.0, 15.0 / 4096.0],
                [0.0, 2.0, -3.0, 6.0 / 4096.0],
                [0.0, 2.0, -2.0, 36.0 / 4096.0],
                [0.0, 2.0, -1.0, 90.0 / 4096.0],
                [0.0, 2.0, 0.0, 120.0 / 4096.0],
                [0.0, 2.0, 1.0, 90.0 / 4096.0],
                [0.0, 2.0, 2.0, 36.0 / 4096.0],
                [0.0, 2.0, 3.0, 6.0 / 4096.0],
                [0.0, 3.0, -3.0, 1.0 / 4096.0],
                [0.0, 3.0, -2.0, 6.0 / 4096.0],
                [0.0, 3.0, -1.0, 15.0 / 4096.0],
                [0.0, 3.0, 0.0, 20.0 / 4096.0],
                [0.0, 3.0, 1.0, 15.0 / 4096.0],
                [0.0, 3.0, 2.0, 6.0 / 4096.0],
                [0.0, 3.0, 3.0, 1.0 / 4096.0],*/
        for child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }
        if let Some(filter_id) = self.filter_has_been_inserted {
            primitives.push(Primitive {
                kind: PrimitiveKind::Filter(filter_id),
                rect: Rect::new(self.position, self.dimension),
            });
        }
    }
}

impl WidgetExt for Blur {}
