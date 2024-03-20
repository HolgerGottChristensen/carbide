use carbide::draw::{Color, Position, Rect};
use carbide::environment::Environment;
use carbide::layout::BasicLayouter;
use carbide::state::IntoReadState;
use carbide::widget::canvas::{Canvas, Context};
use carbide::widget::{Widget, WidgetExt};

pub struct BadgeSymbol;

impl BadgeSymbol {
    pub fn new(rotation: impl IntoReadState<f64>) -> impl Widget {
        Canvas::new(|rect: Rect, mut context: Context, env: &mut Environment| {
            let width = f64::min(rect.width(), rect.height());
            let height = width * 0.75;
            let spacing = width * 0.030;
            let middle = width * 0.5;
            let topWidth = width * 0.226;
            let topHeight = height * 0.488;

            context.add_lines([
                Position::new(middle, spacing),
                Position::new(middle - topWidth, topHeight - spacing),
                Position::new(middle, topHeight / 2.0 + spacing),
                Position::new(middle + topWidth, topHeight - spacing),
                Position::new(middle, spacing),
            ]);

            context.move_to(middle, topHeight / 2.0 + spacing * 3.0);

            context.add_lines([
                Position::new(middle - topWidth, topHeight + spacing),
                Position::new(spacing, height - spacing),
                Position::new(width - spacing, height - spacing),
                Position::new(middle + topWidth, topHeight + spacing),
                Position::new(middle, topHeight / 2.0 + spacing * 3.0),
            ]);

            context.set_fill_style(Color::new_rgba(79, 79, 191, 128));
            context.fill();

            context
        }).padding(-60.0).rotation_effect(rotation.into_read_state()).with_anchor(BasicLayouter::Bottom)
    }
}