use carbide::draw::Angle;
use carbide_core::widget::canvas::CanvasContext;
use carbide_macro::carbide_default_builder2;

use crate::color::WHITE;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::draw::stroke::LineCap;
use crate::environment::{EnvironmentColor};
use crate::state::AnimatedState;
use crate::time::*;
use crate::widget::{Circle, CommonWidget, Empty, Widget, WidgetExt, WidgetId, ZStack};
use crate::widget::canvas::Canvas;

#[derive(Debug, Clone, Widget)]
pub struct ProgressView<W> where W: Widget {
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
}

impl ProgressView<Empty> {
    #[carbide_default_builder2]
    pub fn new() -> ProgressView<impl Widget> {
        ProgressView::new_internal(30.0)
    }

    fn new_internal(size: f64) -> ProgressView<impl Widget> {
        let animation = AnimatedState::linear()
            .repeat()
            .duration(Duration::new(2, 0))
            .range(0.0, 360.0);

        let animation2 = AnimatedState::linear()
            .repeat()
            .duration(Duration::new(1, 0))
            .range(0.0, 360.0);

        let child = ZStack::new((
            Circle::new()
                .stroke(EnvironmentColor::Separator)
                .stroke_style(4.0),
            Canvas::new(|context: &mut CanvasContext| {
                context.move_to(2.0, context.dimension().height / 2.0);
                context.arc(
                    Position::new(context.dimension().width / 2.0, context.dimension().height / 2.0),
                    context.dimension().height / 2.0 - 2.0,
                    Angle::Degrees(0.0),
                    Angle::Degrees(60.0),
                );
                context.set_stroke_style(WHITE);
                context.set_line_width(4.0);
                context.set_line_cap(LineCap::Round);
                context.stroke();
            })
                .rotation_effect(animation),
            Canvas::new(|context: &mut CanvasContext| {
                context.move_to(2.0, context.dimension().height / 2.0);
                context.arc(
                    Position::new(context.dimension().width / 2.0, context.dimension().height / 2.0),
                    context.dimension().height / 2.0 - 2.0,
                    Angle::Degrees(0.0),
                    Angle::Degrees(120.0),
                );
                context.set_stroke_style(WHITE);
                context.set_line_width(4.0);
                context.set_line_cap(LineCap::Round);
                context.stroke();
            })
                .rotation_effect(animation2),
        ))
            .frame(size, size);

        ProgressView {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<W: Widget> ProgressView<W> {
    pub fn size(self, size: f64) -> ProgressView<impl Widget> {
        ProgressView::new_internal(size)
    }
}

impl<W: Widget> CommonWidget for ProgressView<W> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}