use std::time::Instant;
use carbide_core::color::{GREEN, RED, WHITE};
use carbide_core::draw::{Alignment, Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, CanvasContext, LineCap, LineJoin};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let start = Instant::now();

    application.set_scene(
        Window::new(
            "Stroke example",
            Dimension::new(600.0, 600.0),
            Canvas::new(move |context: &mut CanvasContext| {
                let mouse_position = context.env().mouse_position();
                context.request_animation_frame();
                context.move_to(100.0, 100.0);
                /*context.line_to(500.0, 100.0);
                context.line_to(500.0, 500.0);*/

                /*context.line_to(300.0, 100.0);
                context.line_to(300.0, 300.0);
                context.line_to(500.0, 300.0);
                context.line_to(500.0, 500.0);*/

                /*context.line_to(env.mouse_position().x, env.mouse_position().y);
                context.line_to(300.0, 500.0);
                context.line_to(100.0, 500.0);*/

                context.bezier_curve_to(
                    Position::new(300.0, 100.0),
                    Position::new(mouse_position.x, mouse_position.y),
                    Position::new(500.0, 500.0),
                );

                context.set_miter_limit(100.0);
                context.set_line_join(LineJoin::Miter);
                context.set_line_width(3.0);
                //context.set_dash_offset(9.0);
                context.set_dash_offset(Instant::now().duration_since(start).as_secs_f64() * 30.0);
                context.set_dash_pattern(Some(vec![2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]));
                //context.set_stroke_style(WHITE);
                context.set_stroke_style(Gradient::linear(vec![RED, GREEN], Alignment::TopLeading, Alignment::BottomTrailing));
                context.set_line_cap(LineCap::Butt);
                context.stroke();
            })
        )
    );

    application.launch()
}


