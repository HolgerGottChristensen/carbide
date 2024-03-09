use carbide_macro::carbide_default_builder2;

use crate::draw::{Dimension, Position, Scalar};
use crate::flags::WidgetFlag;
use crate::layout::{Layout, LayoutContext};
use crate::widget::{AnyWidget, CommonWidget, Widget, WidgetExt, WidgetId};

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout)]
pub struct Spacer {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    max_size: Option<Scalar>,
}

impl Spacer {
    #[carbide_default_builder2]
    pub fn new() -> Self {
        Spacer {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            max_size: None,
       }
    }

    pub fn fixed(max: Scalar) -> Self {
        Spacer {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            max_size: Some(max),
        }
    }
}

impl Layout for Spacer {
    fn calculate_size(&mut self, requested_size: Dimension, _ctx: &mut LayoutContext) -> Dimension {
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

    fn flag(&self) -> WidgetFlag {
        if let Some(_) = self.max_size {
            WidgetFlag::EMPTY
        } else {
            WidgetFlag::SPACER
        }
    }

    fn foreach_child<'a>(&'a self, _f: &mut dyn FnMut(&'a dyn AnyWidget)) {}

    fn foreach_child_mut<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

    fn foreach_child_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

    fn foreach_child_direct<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}

    fn foreach_child_direct_rev<'a>(&'a mut self, _f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {}


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
