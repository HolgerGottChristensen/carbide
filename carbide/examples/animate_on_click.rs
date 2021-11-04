use std::time::Duration;

use carbide::animation::Animation;
use carbide::color::{BLUE, GREEN, RED};
use carbide::prelude::elastic_in_out;
use carbide::state::{bounce_in, bounce_in_out, bounce_out, ease_in_out, linear, ValueState};
use carbide_controls::{Button, List};
use carbide_controls::capture;
use carbide_core::animate;
use carbide_core::animation::Animatable;
use carbide_core::Color;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::environment::Environment;
use carbide_core::state::{LocalState, State, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Animate on click - Carbide",
        800,
        600,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let offset_x = LocalState::new(-120.0);
    let color = LocalState::new(RED);


    window.set_widgets(
        VStack::new(vec![
            Rectangle::new(vec![])
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
        ]).spacing(10.0),
    );

    window.launch();
}

fn animation_buttons(curve: fn(f64) -> f64, name: &str, offset: &TState<f64>) -> Box<dyn Widget> {
    HStack::new(vec![
        Button::new(format!("{} left", name))
            .on_click(capture!({offset}, |env: &mut Environment| {
                if &*offset.value() > &119.0 {
                    animate!(env, offset := 120.0 => -120.0, curve: curve)
                }
            }))
            .frame(150.0, 22.0),
        Button::new(format!("{} right", name))
            .on_click(capture!({offset}, |env: &mut Environment| {
                if &*offset.value() < &-119.0 {
                    animate!(env, offset := -120.0 => 120.0, curve: curve)
                }
            }))
            .frame(150.0, 22.0),
    ]).spacing(10.0)
}
