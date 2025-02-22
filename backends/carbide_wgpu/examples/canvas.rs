use std::f64::consts::PI;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::*;
use carbide_core::widget::canvas::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {

    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Canvas example - Carbide",
            Dimension::new(600.0, 600.0),
            Canvas::new(|context: &mut CanvasContext| {
                draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                context.set_fill_style(EnvironmentColor::Accent);
                context.fill();
            }).frame(100.0, 100.0),
        )
    );

    application.launch()
}

fn draw_star(
    center: Position,
    number_of_spikes: u32,
    outer_radius: f64,
    inner_radius: f64,
    context: &mut CanvasContext,
) {
    let mut rotation = PI / 2.0 * 3.0;

    let center_x = center.x;
    let center_y = center.y;

    let mut x;
    let mut y;

    let step = PI / number_of_spikes as f64;

    context.begin_path();

    context.move_to(center_x, center_y - outer_radius);

    for _ in 0..number_of_spikes {
        x = center_x + rotation.cos() * outer_radius;
        y = center_y + rotation.sin() * outer_radius;

        context.line_to(x, y);
        rotation += step;

        x = center_x + rotation.cos() * inner_radius;
        y = center_y + rotation.sin() * inner_radius;
        context.line_to(x, y);
        rotation += step;
    }

    context.line_to(center_x, center_y - outer_radius);
    context.close_path();
}
