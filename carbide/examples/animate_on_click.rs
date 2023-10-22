use std::time::Duration;

use carbide::color::{BLUE, GREEN, RED};
use carbide::animation::{bounce_out, ease_in_out, linear, elastic_in_out};
use carbide::state::{LocalState, TState, ReadState};
use carbide::animate;
use carbide::controls::capture;
use carbide::controls::Button;
use carbide::environment::Environment;
use carbide::widget::*;
use carbide::Color;
use carbide::draw::Dimension;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let offset_x = LocalState::new(-120.0);
    let color = LocalState::new(RED);

    application.set_scene(Window::new(
        "Animate on click - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new(vec![
            Rectangle::new()
                .fill(color.clone())
                .frame(60.0, 60.0)
                .offset(offset_x.clone(), 0.0),
            HStack::new(vec![
                Button::new("Rgba to blue")
                    .on_click(capture!({color}, |env: &mut Environment| {
                    animate!(env, color => BLUE, duration: Duration::new(2, 0))
                }))
                    .frame(96.0, 22.0),
                Button::new("Hsla to green")
                    .on_click(capture!({color}, |env: &mut Environment| {
                    animate!(env, color => GREEN, interpolation: Color::hsla_blend)
                }))
                    .frame(98.0, 22.0),
                Button::new("Hsla to red")
                    .on_click(capture!({color}, |env: &mut Environment| {
                    animate!(env, color => RED, interpolation: Color::hsla_blend, duration: Duration::new(4, 0))
                }))
                    .frame(96.0, 22.0),
            ]).spacing(10.0),
            animation_buttons(linear, "Linear", &offset_x),
            animation_buttons(elastic_in_out, "Elastic", &offset_x),
            animation_buttons(bounce_out, "Bounce out", &offset_x),
            animation_buttons(ease_in_out, "Ease in-out", &offset_x),
        ]).spacing(10.0)
    ).close_application_on_window_close());


    application.launch();
}

fn animation_buttons(curve: fn(f64) -> f64, name: &str, offset: &TState<f64>) -> Box<dyn Widget> {
    HStack::new(vec![
        Button::new(format!("{} left", name))
            .on_click(capture!({ offset }, |env: &mut Environment| {
                if &*offset.value() > &119.0 {
                    animate!(env, offset := 120.0 => -120.0, curve: curve)
                }
            }))
            .frame(150.0, 22.0),
        Button::new(format!("{} right", name))
            .on_click(capture!({ offset }, |env: &mut Environment| {
                if &*offset.value() < &-119.0 {
                    animate!(env, offset := -120.0 => 120.0, curve: curve)
                }
            }))
            .frame(150.0, 22.0),
    ])
    .spacing(10.0)
        .boxed()
}
