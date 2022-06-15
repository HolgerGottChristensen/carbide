use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Filter {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    blur_type: BlurType,
    filter: ImageFilter,
    filter_id: Option<FilterId>,
}

impl Filter {
    pub fn new(filter: ImageFilter, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Filter {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            blur_type: BlurType::Gaussian(2.0),
            filter,
            filter_id: None,
        })
    }
}

impl CommonWidget for Filter {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
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

impl Render for Filter {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        if self.filter_id == None {
            self.filter_id = Some(env.insert_filter(self.filter.clone()));
        }

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }

        if let Some(filter_id) = self.filter_id {
            primitives.push(Primitive {
                kind: PrimitiveKind::Filter(filter_id),
                bounding_box: Rect::new(self.position, self.dimension),
            });
        }
    }
}

impl WidgetExt for Filter {}
