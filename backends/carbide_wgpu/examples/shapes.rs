use std::f64::consts::PI;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_core::widget::canvas::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let landscape = "images/landscape.png";

    let rectangles = HStack::new((
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .frame(100.0, 100.0),
        Rectangle::new()
            .stroke(EnvironmentColor::Accent)
            .stroke_style(10.0)
            .frame(100.0, 100.0),
        Rectangle::new()
            .fill(EnvironmentColor::Accent)
            .stroke(EnvironmentColor::Red)
            .frame(100.0, 100.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(Rectangle::new().fill(EnvironmentColor::Accent))
            .frame(100.0, 100.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(
                Rectangle::new()
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0),
            )
            .frame(100.0, 100.0),
    ));

    let rounded_rectangles = HStack::new((
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
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(RoundedRectangle::new(CornerRadii::all(25.0)))
            .frame(100.0, 100.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(
                RoundedRectangle::new(CornerRadii::all(25.0))
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0),
            )
            .frame(100.0, 100.0),
    ));

    let circles = HStack::new((
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
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(Circle::new())
            .frame(100.0, 100.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(
                Circle::new()
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0),
            )
            .frame(100.0, 100.0),
    ));

    let ellipsis = HStack::new((
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
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(Ellipse::new())
            .frame(100.0, 50.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(
                Ellipse::new()
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0),
            )
            .frame(100.0, 50.0),
    ));

    let capsules = HStack::new((
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
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(Capsule::new())
            .frame(100.0, 50.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(
                Capsule::new()
                    .stroke(EnvironmentColor::Accent)
                    .stroke_style(10.0),
            )
            .frame(100.0, 50.0),
    ));

    let stars = HStack::new((
        Canvas::new(|_, context: &mut Context, _: &mut Environment| {
            draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
            context.set_fill_style(EnvironmentColor::Accent);
            context.fill();
        }).frame(100.0, 100.0),
        Canvas::new(|_, context: &mut Context, _: &mut Environment| {
            draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
            context.set_line_width(10.0);
            context.set_stroke_style(EnvironmentColor::Accent);
            context.stroke();
        }).frame(100.0, 100.0),
        Canvas::new(|_, context: &mut Context, _: &mut Environment| {
            draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
            context.set_fill_style(EnvironmentColor::Accent);
            context.set_stroke_style(EnvironmentColor::Red);
            context.fill();
            context.stroke();
        }).frame(100.0, 100.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(Canvas::new(|_, context: &mut Context, _: &mut Environment| {
                draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                context.fill();
            }))
            .frame(100.0, 100.0),
        Image::new(landscape)
            .scaled_to_fill()
            .frame(200.0, 200.0)
            .clip_shape(Canvas::new(|_, context: &mut Context, _: &mut Environment| {
                draw_star(Position::new(50.0, 50.0), 5, 45.0, 20.0, context);
                context.set_line_width(10.0);
                context.stroke();
            }))
            .frame(100.0, 100.0),
    ));

    application.set_scene(
        Window::new(
            "Shapes example - Carbide",
            Dimension::new(600.0, 600.0),
            VStack::new((
                rectangles,
                rounded_rectangles,
                circles,
                ellipsis,
                capsules,
                stars,
            )),
        ).close_application_on_window_close()
    );

    application.launch()
}

fn draw_star(
    center: Position,
    number_of_spikes: u32,
    outer_radius: f64,
    inner_radius: f64,
    context: &mut Context,
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
