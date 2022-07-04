use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_core::Color;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Materials example".to_string(),
        600,
        450,
        Some(icon_path.clone()),
    );

    let background = HStack::new(vec![
        Rectangle::new()
            .fill(Color::new_rgb(251, 61, 56))
            .frame(200.0, 800.0),
        Rectangle::new()
            .fill(Color::new_rgb(253, 148, 38))
            .frame(80.0, 800.0),
        Rectangle::new()
            .fill(Color::new_rgb(254, 203, 47))
            .frame(80.0, 800.0),
        Rectangle::new()
            .fill(Color::new_rgb(61, 198, 95))
            .frame(80.0, 800.0),
        Rectangle::new()
            .fill(Color::new_rgb(21, 126, 251))
            .frame(80.0, 800.0),
        Rectangle::new()
            .fill(Color::new_rgb(174, 89, 219))
            .frame(80.0, 800.0),
        Rectangle::new()
            .fill(Color::new_rgb(251, 61, 56))
            .frame(200.0, 800.0),
    ])
    .spacing(0.0)
    .rotation_effect(45.0);

    window.set_widgets(ZStack::new(vec![
        background,
        VStack::new(vec![
            HStack::new(vec![
                Rectangle::new()
                    .material(EnvironmentColor::UltraThickLight)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::ThickLight)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::RegularLight)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::ThinLight)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::UltraThinLight)
                    .frame(100.0, 100.0),
            ])
            .spacing(10.0),
            HStack::new(vec![
                Rectangle::new()
                    .material(EnvironmentColor::UltraThickDark)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::ThickDark)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::RegularDark)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::ThinDark)
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .material(EnvironmentColor::UltraThinDark)
                    .frame(100.0, 100.0),
            ])
            .spacing(10.0),
        ])
        .spacing(10.0),
    ]));

    window.launch();
}
