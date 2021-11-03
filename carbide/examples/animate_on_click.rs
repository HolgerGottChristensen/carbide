use std::time::Duration;

use carbide::animation::Animation;
use carbide::color::{BLUE, GREEN, RED};
use carbide::prelude::elastic_in_out;
use carbide::state::{bounce_in_out, linear, ValueState};
use carbide_controls::{Button, List};
use carbide_controls::capture;
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
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let offset_x = LocalState::new(-120.0);
    let color = LocalState::new(RED);
    let color1 = color.clone();
    let color2 = color.clone();
    let color3 = color.clone();

    fn hsla_blend(from: &Color, to: &Color, percentage: f64) -> Color {
        let from_hsla = from.to_hsl();
        let to_hsla = to.to_hsl();

        let from_h = from_hsla.0 * 180.0 / std::f32::consts::PI;
        let to_h = to_hsla.0 * 180.0 / std::f32::consts::PI;

        let d = to_h - from_h;
        let delta = d + if d.abs() > 180.0 {
            if d < 0.0 {
                360.0
            } else {
                -360.0
            }
        } else {
            0.0
        };

        let mut new_angle_deg = from_h as f64 + (percentage * delta as f64);

        if new_angle_deg < 0.0 {
            new_angle_deg = 360.0 + new_angle_deg;
        } else if new_angle_deg >= 360.0 {
            new_angle_deg = new_angle_deg - 360.0
        }

        Color::Hsla(
            (new_angle_deg as f32).to_radians().abs(),
            from_hsla.1.interpolate(&to_hsla.1, percentage),
            from_hsla.2.interpolate(&to_hsla.2, percentage),
            from_hsla.3.interpolate(&to_hsla.3, percentage),
        )
    }

    window.set_widgets(
        VStack::new(vec![
            Rectangle::new(vec![])
                .fill(color.clone())
                .frame(60.0, 60.0)
                .offset(offset_x.clone(), 0.0),
            Button::new("Rgba to blue")
                .on_click(move |env: &mut Environment| {
                    let color = color1.clone();
                    let color_value = *color.value();

                    fn rgba_blend(from: &Color, to: &Color, percentage: f64) -> Color {
                        let from_rgba = from.to_rgb();
                        let to_rgba = to.to_rgb();
                        Color::Rgba(
                            from_rgba.0.interpolate(&to_rgba.0, percentage),
                            from_rgba.1.interpolate(&to_rgba.1, percentage),
                            from_rgba.2.interpolate(&to_rgba.2, percentage),
                            from_rgba.3.interpolate(&to_rgba.3, percentage),
                        )
                    }

                    let animation = Animation::new_custom(
                        color.to_boxed_state(),
                        color_value,
                        BLUE,
                        rgba_blend,
                    ).duration(Duration::new(2, 0));
                    env.insert_animation(animation);
                })
                .frame(150.0, 22.0),
            Button::new("Hsla to green")
                .on_click(move |env: &mut Environment| {
                    let color = color2.clone();
                    let color_value = *color.value();

                    let animation = Animation::new_custom(
                        color.to_boxed_state(),
                        color_value,
                        GREEN,
                        hsla_blend,
                    ).curve(elastic_in_out).duration(Duration::new(2, 0));
                    env.insert_animation(animation);
                })
                .frame(150.0, 22.0),
            Button::new("Hsla to red")
                .on_click(move |env: &mut Environment| {
                    let color = color3.clone();
                    let color_value = *color.value();

                    let animation = Animation::new_custom(
                        color.to_boxed_state(),
                        color_value,
                        RED,
                        hsla_blend,
                    ).duration(Duration::new(2, 0));
                    env.insert_animation(animation);
                })
                .frame(150.0, 22.0),
            animation_buttons(linear, "Linear", &offset_x),
            animation_buttons(elastic_in_out, "Elastic", &offset_x),
        ]).spacing(10.0),
    );

    window.launch();
}

fn animation_buttons(curve: fn(f64) -> f64, name: &str, offset: &TState<f64>) -> Box<dyn Widget> {
    let offset_x1 = offset.clone();
    let offset_x2 = offset.clone();
    HStack::new(vec![
        Button::new(format!("{} left", name))
            .on_click(move |env: &mut Environment| {
                let offset = offset_x2.clone();
                if &*offset.value() > &119.0 {
                    let animation = Animation::new(
                        offset.to_boxed_state(),
                        120.0,
                        -120.0,
                    ).curve(curve);
                    env.insert_animation(animation);
                }
            })
            .frame(150.0, 22.0),
        Button::new(format!("{} right", name))
            .on_click(move |env: &mut Environment| {
                let offset = offset_x1.clone();
                if &*offset.value() < &-119.0 {
                    let animation = Animation::new(
                        offset.to_boxed_state(),
                        -120.0,
                        120.0,
                    ).curve(curve);
                    env.insert_animation(animation);
                }
            })
            .frame(150.0, 22.0),
    ]).spacing(10.0)
}
