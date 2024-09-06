use carbide::draw::{Alignment, Color, Dimension, Position, Rect};
use carbide::environment::Environment;
use carbide::widget::{Gradient, Widget, WidgetExt};
use carbide::widget::canvas::{Canvas, CanvasContext};

use crate::hexagon_parameters::HexagonParameters;

pub struct BadgeBackground;

impl BadgeBackground {
    pub fn new() -> impl Widget {
        let hexagon = HexagonParameters::new();

        Canvas::new(move |rect: Rect, context: &mut CanvasContext, _: &mut Environment| {
            let mut width = rect.width().min(rect.height());
            let height = width;

            let x_scale = 0.832;
            let x_offset = (width * (1.0 - x_scale)) / 2.0;
            width *= x_scale;

            context.move_to(
                width * 0.95 + x_offset,
                height * (0.2 + &hexagon.adjustment)
            );

            for segment in &hexagon.segments {
                context.line_to(
                    width * segment.line.x + x_offset,
                    height * segment.line.y
                );

                context.quadratic_curve_to(
                    Position::new(
                        width * segment.control.x + x_offset,
                        height * segment.control.y,
                    ),
                    Position::new(
                        width * segment.curve.x + x_offset,
                        height * segment.curve.y,
                    )
                );
            }

            context.set_fill_style(Gradient::linear(
                vec![
                    Color::new_rgb(239, 120, 221),
                    Color::new_rgb(239, 172, 120),
                ],
                Alignment::Custom(0.5, 0.0),
                Alignment::Custom(0.5, 0.6)
            ));
            context.fill();
        }).aspect_ratio(Dimension::new(1.0, 1.0))
            .scale_to_fit()
    }
}