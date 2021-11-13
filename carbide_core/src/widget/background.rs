use crate::draw::{Dimension, Position};
use crate::prelude::*;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout, Render)]
pub struct Background {
    id: Uuid,
    child: Box<dyn Widget>,
    background: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    alignment: Box<dyn Layouter>,
}

impl Background {
    pub fn new(child: Box<dyn Widget>, background: Box<dyn Widget>) -> Box<Background> {
        Box::new(Background {
            id: Uuid::new_v4(),
            child,
            background,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            alignment: Box::new(BasicLayouter::Center),
        })
    }

    /*pub fn with_alignment(mut self, layouter: BasicLayouter) -> Box<Self> {
        self.alignment = Box::new(layouter);
        Box::new(self)
    }*/
}

impl Layout for Background {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let child_size = self.child.calculate_size(requested_size, env);
        self.background.calculate_size(child_size, env);
        self.dimension = child_size;
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = self.alignment.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, self.child.deref_mut());
        positioning(position, dimension, self.background.deref_mut());
        self.child.position_children();
        self.background.position_children();
    }
}

impl Render for Background {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        self.background.process_get_primitives(primitives, env);
        self.child.process_get_primitives(primitives, env);
    }
}

impl CommonWidget for Background {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn alignment(&self) -> Box<dyn Layouter> {
        self.alignment.clone()
    }

    fn set_alignment(&mut self, alignment: Box<dyn Layouter>) {
        self.alignment = alignment;
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

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl WidgetExt for Background {}
