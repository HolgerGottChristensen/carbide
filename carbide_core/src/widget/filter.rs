use carbide_core::render::RenderContext;
use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position, Rect};
use crate::environment::Environment;
use crate::render::Render;
use crate::widget::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Filter {
    id: WidgetId,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
    //blur_type: BlurType,
    filter: ImageFilter,
    filter_id: Option<FilterId>,
}

impl Filter {
    #[carbide_default_builder]
    pub fn new(filter: ImageFilter, child: Box<dyn AnyWidget>) -> Box<Self> {}

    pub fn new(filter: ImageFilter, child: Box<dyn AnyWidget>) -> Box<Self> {
        Box::new(Filter {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            //blur_type: BlurType::Gaussian(2.0),
            filter,
            filter_id: None,
        })
    }
}

impl CommonWidget for Filter {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach(f);
            return;
        }

        f(&self.child);
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_mut(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        if self.child.is_ignore() {
            return;
        }

        if self.child.is_proxy() {
            self.child.foreach_rev(f);
            return;
        }

        f(&mut self.child);
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        f(&mut self.child);
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
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
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        let filter_id = if let Some(filter_id) = self.filter_id {
            filter_id
        } else {
            let id = env.insert_filter(self.filter.clone());
            self.filter_id = Some(id);
            id
        };

        context.filter(filter_id, Rect::new(self.position, self.dimension), |this| {
            self.foreach_child_mut(&mut |child| {
                child.render(this, env);
            });
        });
    }
}

impl WidgetExt for Filter {}
