use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::flags::Flags;
use crate::render::{Primitive, PrimitiveKind, Render};
use crate::widget::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Filter {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    blur_type: BlurType,
    filter: ImageFilter,
    filter_id: Option<FilterId>,
}

impl Filter {
    #[carbide_default_builder]
    pub fn new(filter: ImageFilter, child: Box<dyn Widget>) -> Box<Self> {}

    pub fn new(filter: ImageFilter, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Filter {
            id: WidgetId::new(),
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

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_child_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn Widget)) {
        f(&mut self.child);
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

        self.foreach_child_mut(&mut |child| {
            child.process_get_primitives(primitives, env);
        });

        if let Some(filter_id) = self.filter_id {
            primitives.push(Primitive {
                kind: PrimitiveKind::Filter(filter_id),
                bounding_box: Rect::new(self.position, self.dimension),
            });
        }
    }
}

impl WidgetExt for Filter {}
