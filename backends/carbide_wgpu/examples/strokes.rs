use carbide_core::color::{GREEN, RED, WHITE};
use carbide_core::draw::{Alignment, Dimension, Position};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use carbide_core::widget::canvas::{Canvas, Context, LineCap, LineJoin};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Stroke example",
            Dimension::new(600.0, 600.0),
            Canvas::new(|_, mut context: Context, _: &mut Environment| {
                context.move_to(100.0, 100.0);
                /*context.line_to(500.0, 100.0);
                context.line_to(500.0, 500.0);*/

                /*context.line_to(300.0, 100.0);
                context.line_to(300.0, 300.0);
                context.line_to(500.0, 300.0);
                context.line_to(500.0, 500.0);*/

                context.line_to(500.0, 100.0);
                context.line_to(300.0, 500.0);

                /*context.bezier_curve_to(
                    Position::new(300.0, 100.0),
                    Position::new(300.0, 600.0),
                    Position::new(500.0, 500.0),
                );*/
                context.set_miter_limit(100.0);
                context.set_line_join(LineJoin::Miter);
                context.set_line_width(40.0);
                context.set_stroke_style(Gradient::linear(vec![RED, GREEN], Alignment::Leading, Alignment::Trailing));
                context.set_line_cap(LineCap::Butt);
                context.stroke();
                context
            })
        ).close_application_on_window_close()
    );

    application.launch()
}
