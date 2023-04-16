use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::layout::Layout;
use crate::Scalar;
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout)]
pub struct Spacer {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    max_size: Option<Scalar>,
}

impl Spacer {
    #[carbide_default_builder]
    pub fn new() -> Box<Self> {}

    pub fn new() -> Box<Self> {
        Box::new(Spacer {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            max_size: None,
        })
    }

    pub fn fixed(max: Scalar) -> Box<Self> {
        Box::new(Spacer {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            max_size: Some(max),
        })
    }
}

impl Layout for Spacer {
    fn calculate_size(&mut self, requested_size: Dimension, _: &mut Environment) -> Dimension {
        if let Some(max) = self.max_size {
            self.dimension = Dimension::new(
                requested_size.width.min(max),
                requested_size.height.min(max),
            );
            self.dimension
        } else {
            self.dimension = requested_size;
            requested_size
        }
    }
}

impl CommonWidget for Spacer {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn flag(&self) -> Flags {
        if let Some(_) = self.max_size {
            Flags::EMPTY
        } else {
            Flags::SPACER
        }
    }

    fn foreach_child(&self, f: &mut dyn FnMut(&dyn Widget)) {}

    fn foreach_child_mut(&mut self, f: &mut dyn FnMut(&mut dyn Widget)) {}

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
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

impl WidgetExt for Spacer {}
