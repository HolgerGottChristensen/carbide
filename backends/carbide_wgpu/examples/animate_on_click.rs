use std::time::Duration;

use carbide_core::color::{BLUE, GREEN, RED, Color};
use carbide_core::animation::{bounce_out, ease_in_out, linear, elastic_in_out};
use carbide_core::state::{LocalState, TState, ReadState, State};
use carbide_core::animate;
use carbide_controls::capture;
use carbide_controls::Button;
use carbide_core::environment::Environment;
use carbide_core::widget::*;
use carbide_core::draw::Dimension;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let offset_x = LocalState::new(-120.0);
    let color = LocalState::new(RED);

    application.set_scene(Window::new(
        "Animate on click - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Rectangle::new()
                .fill(color.clone())
                .frame(60.0, 60.0)
                .offset(offset_x.clone(), 0.0),
            HStack::new((
                Button::new_primary("Rgba to blue", capture!({color}, |env: &mut Environment| {
                    animate!(env, color => BLUE, duration: Duration::new(2, 0))
                }))
                    .frame(96.0, 22.0),
                Button::new_primary("Hsla to green", capture!({color}, |env: &mut Environment| {
                    animate!(env, color => GREEN, interpolation: Color::hsla_blend)
                }))
                    .frame(98.0, 22.0),
                Button::new_primary("Hsla to red", capture!({color}, |env: &mut Environment| {
                    animate!(env, color => RED, interpolation: Color::hsla_blend, duration: Duration::new(4, 0))
                }))
                    .frame(96.0, 22.0),
            )).spacing(10.0),
            animation_buttons(linear, "Linear", offset_x.clone()),
            animation_buttons(elastic_in_out, "Elastic", offset_x.clone()),
            animation_buttons(bounce_out, "Bounce out", offset_x.clone()),
            animation_buttons(ease_in_out, "Ease in-out", offset_x.clone()),
        )).spacing(10.0)
    ).close_application_on_window_close());


    application.launch();
}

fn animation_buttons(curve: fn(f64) -> f64, name: &str, offset: impl State<T=f64>) -> Box<dyn AnyWidget> {
    HStack::new((
        Button::new_primary(format!("{} left", name), capture!({ offset }, |env: &mut Environment| {
                if &*offset.value() > &119.0 {
                    animate!(env, offset := 120.0 => -120.0, curve: curve)
                }
            }))
            .frame(150.0, 22.0),
        Button::new_primary(format!("{} right", name), capture!({ offset }, |env: &mut Environment| {
                if &*offset.value() < &-119.0 {
                    animate!(env, offset := -120.0 => 120.0, curve: curve)
                }
            }))
            .frame(150.0, 22.0),
    ))
    .spacing(10.0)
        .boxed()
}
