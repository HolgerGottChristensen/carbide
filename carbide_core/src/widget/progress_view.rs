use std::time::Duration;
use carbide_core::draw::Rect;
use carbide_core::widget::canvas::Context;


use carbide_macro::{carbide_default_builder2};

use crate::color::WHITE;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::{Environment, EnvironmentColor};
use crate::state::AnimatedState;
use crate::widget::{Circle, CommonWidget, AnyWidget, WidgetExt, WidgetId, ZStack, Widget};
use crate::widget::canvas::Canvas;
use crate::widget::canvas::LineCap;

#[derive(Debug, Clone, Widget)]
pub struct ProgressView {
    id: WidgetId,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
}

impl ProgressView {
    #[carbide_default_builder2]
    pub fn new() -> Self {
        ProgressView::new_internal(30.0)
    }

    pub fn size(self, size: f64) -> Self {
        ProgressView::new_internal(size)
    }

    fn new_internal(size: f64) -> Self {
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
                .stroke_style(4.0)
                .boxed(),
            Canvas::new(|rect: Rect, mut context: Context, _env: &mut Environment| {
                context.move_to(2.0, rect.height() / 2.0);
                context.arc(
                    rect.width() / 2.0,
                    rect.height() / 2.0,
                    rect.height() / 2.0 - 2.0,
                    0.0,
                    60.0,
                );
                context.set_stroke_style(WHITE);
                context.set_line_width(4.0);
                context.set_line_cap(LineCap::Round);
                context.stroke();
                context
            })
            .rotation_effect(animation)
                .boxed(),
            Canvas::new(|rect: Rect, mut context: Context, _env: &mut Environment| {
                context.move_to(2.0, rect.height() / 2.0);
                context.arc(
                    rect.width() / 2.0,
                    rect.height() / 2.0,
                    rect.height() / 2.0 - 2.0,
                    0.0,
                    120.0,
                );
                context.set_stroke_style(WHITE);
                context.set_line_width(4.0);
                context.set_line_cap(LineCap::Round);
                context.stroke();
                context
            })
            .rotation_effect(animation2)
                .boxed(),
        ])
        .frame(size, size)
            .boxed();

        ProgressView {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl CommonWidget for ProgressView {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl WidgetExt for ProgressView {}
