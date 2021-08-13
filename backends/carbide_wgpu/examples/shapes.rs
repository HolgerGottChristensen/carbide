use std::f64::consts::PI;

use carbide_core::draw::Position;
use carbide_core::environment::*;
use carbide_core::text::*;
use carbide_core::widget::*;
use carbide_core::widget::canvas::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Shapes example".to_string(), 1200, 1200, Some(icon_path));

    let landscape_id = window.add_image("images/landscape.png");

    window.set_widgets(VStack::new(vec![
        HStack::new(vec![
            Rectangle::new(vec![])
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0),
            Rectangle::new(vec![])
                .stroke(EnvironmentColor::Accent)
                .stroke_style(10.0)
                .frame(100.0, 100.0),
            Rectangle::new(vec![])
                .fill(EnvironmentColor::Accent)
                .stroke(EnvironmentColor::Red)
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(
                    Rectangle::new(vec![])
                        .fill(EnvironmentColor::Accent),
                )
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(
                    Rectangle::new(vec![])
                        .stroke(EnvironmentColor::Accent)
                        .stroke_style(10.0),
                )
                .frame(100.0, 100.0),
        ]),
        HStack::new(vec![
            RoundedRectangle::new(CornerRadii::all(25.0))
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0),
            RoundedRectangle::new(CornerRadii::all(25.0))
                .stroke(EnvironmentColor::Accent)
                .stroke_style(10.0)
                .frame(100.0, 100.0),
            RoundedRectangle::new(CornerRadii::all(25.0))
                .fill(EnvironmentColor::Accent)
                .stroke(EnvironmentColor::Red)
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(RoundedRectangle::new(CornerRadii::all(25.0)))
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(
                    RoundedRectangle::new(CornerRadii::all(25.0))
                        .stroke(EnvironmentColor::Accent)
                        .stroke_style(10.0),
                )
                .frame(100.0, 100.0),
        ]),
        HStack::new(vec![
            Circle::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0),
            Circle::new()
                .stroke(EnvironmentColor::Accent)
                .stroke_style(10.0)
                .frame(100.0, 100.0),
            Circle::new()
                .fill(EnvironmentColor::Accent)
                .stroke(EnvironmentColor::Red)
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(Circle::new())
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(
                    Circle::new()
                        .stroke(EnvironmentColor::Accent)
                        .stroke_style(10.0),
                )
                .frame(100.0, 100.0),
        ]),
        HStack::new(vec![
            Ellipse::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 50.0),
            Ellipse::new()
                .stroke(EnvironmentColor::Accent)
                .stroke_style(10.0)
                .frame(100.0, 50.0),
            Ellipse::new()
                .fill(EnvironmentColor::Accent)
                .stroke(EnvironmentColor::Red)
                .frame(100.0, 50.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(Ellipse::new())
                .frame(100.0, 50.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(
                    Ellipse::new()
                        .stroke(EnvironmentColor::Accent)
                        .stroke_style(10.0),
                )
                .frame(100.0, 50.0),
        ]),
        HStack::new(vec![
            Capsule::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 50.0),
            Capsule::new()
                .stroke(EnvironmentColor::Accent)
                .stroke_style(10.0)
                .frame(100.0, 50.0),
            Capsule::new()
                .fill(EnvironmentColor::Accent)
                .stroke(EnvironmentColor::Red)
                .frame(100.0, 50.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(Capsule::new())
                .frame(100.0, 50.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(
                    Capsule::new()
                        .stroke(EnvironmentColor::Accent)
                        .stroke_style(10.0),
                )
                .frame(100.0, 50.0),
        ]),
        HStack::new(vec![
            Canvas::new(|_, mut context| {
                context = draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                context.set_fill_style(EnvironmentColor::Accent);
                context.fill();
                context
            })
                .frame(100.0, 100.0),
            Canvas::new(|_, mut context| {
                context = draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                context.set_line_width(10.0);
                context.set_stroke_style(EnvironmentColor::Accent);
                context.stroke();
                context
            })
                .frame(100.0, 100.0),
            Canvas::new(|_, mut context| {
                context = draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                context.set_fill_style(EnvironmentColor::Accent);
                context.set_stroke_style(EnvironmentColor::Red);
                context.fill();
                context.stroke();
                context
            })
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(Canvas::new(|_, mut context| {
                    context = draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                    context.fill();
                    context
                }))
                .frame(100.0, 100.0),
            Image::new(landscape_id)
                .scaled_to_fill()
                .frame(200.0, 200.0)
                .clip_shape(Canvas::new(|_, mut context| {
                    context = draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                    context.set_line_width(10.0);
                    context.stroke();
                    context
                }))
                .frame(100.0, 100.0),
        ]),
    ]));

    window.run_event_loop();
}

fn draw_star(
    center: Position,
    number_of_spikes: u32,
    outer_radius: f64,
    inner_radius: f64,
    mut context: Context,
) -> Context {
    let mut rotation = PI / 2.0 * 3.0;

    let center_x = center.x();
    let center_y = center.y();

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
