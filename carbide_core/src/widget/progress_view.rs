use instant::Duration;

use crate::color::WHITE;
use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::prelude::canvas::LineCap;
use crate::widget::canvas::Canvas;
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
pub struct ProgressView {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl ProgressView {
    pub fn new() -> Box<Self> {
        ProgressView::new_internal(30.0)
    }

    pub fn size(self, size: f64) -> Box<Self> {
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
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

CommonWidgetImpl!(ProgressView, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for ProgressView {}
