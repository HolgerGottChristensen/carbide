use std::f64::consts::PI;

use carbide_core::Point;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Shapes example".to_string(), 800, 1200, Some(icon_path));

    let mut family = FontFamily::new("NotoSans");
    family.add_font("fonts/NotoSans/NotoSans-Regular.ttf", FontWeight::Normal, FontStyle::Normal);
    family.add_font("fonts/NotoSans/NotoSans-Italic.ttf", FontWeight::Normal, FontStyle::Italic);
    family.add_font("fonts/NotoSans/NotoSans-Bold.ttf", FontWeight::Bold, FontStyle::Normal);
    window.add_font_family(family);

    window.set_widgets(
        VStack::initialize(vec![
            HStack::initialize(vec![
                Rectangle::initialize(vec![])
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0),
                Rectangle::initialize(vec![])
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0)
                    .frame(100.0, 100.0),
                Rectangle::initialize(vec![])
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Red)
                    .frame(100.0, 100.0)
            ]),
            HStack::initialize(vec![
                RoundedRectangle::initialize(CornerRadii::all(25.0))
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0),
                RoundedRectangle::initialize(CornerRadii::all(25.0))
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0)
                    .frame(100.0, 100.0),
                RoundedRectangle::initialize(CornerRadii::all(25.0))
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Red)
                    .frame(100.0, 100.0)
            ]),
            HStack::initialize(vec![
                Ellipse::new()
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0),
                Ellipse::new()
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0)
                    .frame(100.0, 100.0),
                Ellipse::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Red)
                    .frame(100.0, 100.0)
            ]),
            HStack::initialize(vec![
                Capsule::initialize()
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 50.0)
                    .frame(100.0, 100.0),
                Capsule::initialize()
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0)
                    .frame(100.0, 50.0)
                    .frame(100.0, 100.0),
                Capsule::initialize()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Red)
                    .frame(100.0, 50.0)
                    .frame(100.0, 100.0)
            ]),
            HStack::initialize(vec![
                Canvas::initialize(|_, mut context| {
                    context = draw_star([50.0, 50.0], 5, 45.0, 20.0, context);
                    context.set_fill_style(EnvironmentColor::Accent);
                    context.fill();
                    context
                }).frame(100.0, 100.0),
                Canvas::initialize(|_, mut context| {
                    context = draw_star([50.0, 50.0], 5, 45.0, 20.0, context);
                    context.set_line_width(10.0);
                    context.set_stroke_style(EnvironmentColor::Accent);
                    context.stroke();
                    context
                }).frame(100.0, 100.0),
                Canvas::initialize(|_, mut context| {
                    context = draw_star([50.0, 50.0], 5, 45.0, 20.0, context);
                    context.set_fill_style(EnvironmentColor::Accent);
                    context.set_stroke_style(EnvironmentColor::Red);
                    context.fill();
                    context.stroke();
                    context
                }).frame(100.0, 100.0),
            ]),
        ])
    );

    window.run_event_loop();
}

fn draw_star(center: Point, number_of_spikes: u32, outer_radius: f64, inner_radius: f64, mut context: Context) -> Context {
    let mut rotation = PI / 2.0 * 3.0;

    let center_x = center[0];
    let center_y = center[1];

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

    context
}