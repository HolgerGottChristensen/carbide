use instant::Duration;

use crate::color::WHITE;
use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::prelude::canvas::LineCap;
use crate::render::PrimitiveKind;
use crate::widget::canvas::Canvas;

#[derive(Debug, Clone, Widget)]
pub struct ProgressView {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl ProgressView {
    pub fn new() -> Box<Self> {
        ProgressView::new_internal(30.0)
    }

    pub fn size(mut self, size: f64) -> Box<Self> {
        ProgressView::new_internal(size)
    }

    fn new_internal(size: f64) -> Box<Self> {
        let animation = AnimatedState::linear(None)
            .repeat()
            .duration(Duration::new(2, 0))
            .range(0.0, 360.0);

        let animation2 = AnimatedState::linear(None)
            .repeat()
            .duration(Duration::new(1, 0))
            .range(0.0, 360.0);

        let child = ZStack::new(vec![
            Circle::new()
                .stroke(EnvironmentColor::Separator)
                .stroke_style(4.0),
            Canvas::new(|rect, mut context| {
                context.move_to(2.0, rect.height() / 2.0);
                context.arc(rect.width() / 2.0, rect.height() / 2.0, rect.height() / 2.0 - 2.0, 0.0, 60.0);
                context.set_stroke_style(WHITE);
                context.set_line_width(4.0);
                context.set_line_cap(LineCap::Round);
                context.stroke();
                context
            }).rotation_effect(animation),
            Canvas::new(|rect, mut context| {
                context.move_to(2.0, rect.height() / 2.0);
                context.arc(rect.width() / 2.0, rect.height() / 2.0, rect.height() / 2.0 - 2.0, 0.0, 120.0);
                context.set_stroke_style(WHITE);
                context.set_line_width(4.0);
                context.set_line_cap(LineCap::Round);
                context.stroke();
                context
            }).rotation_effect(animation2),
        ]).frame(size, size);


        Box::new(ProgressView {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl CommonWidget for ProgressView {
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

impl WidgetExt for ProgressView {}
