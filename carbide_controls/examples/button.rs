extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::Button;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
        String::from("Hejsa"),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Italic.ttf",
        FontWeight::Normal,
        FontStyle::Italic,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Bold.ttf",
        FontWeight::Bold,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    window.set_widgets(
        VStack::new(vec![
            Button::<(), String>::new(
                Text::new("Primary button").font_size(EnvironmentFontSize::Body),
            )
                .on_click(|_, _, _| {
                    println!("Clicked the primary button");
                })
                .frame(180.0, 26.0),
            Button::<(), String>::new(
                Text::new("Secondary button").font_size(EnvironmentFontSize::Body),
            )
                .secondary()
                .on_click(|_, _, _| {
                    println!("Clicked the secondary button");
                })
                .frame(180.0, 26.0),
        ])
            .spacing(10.0)
            .padding(40.0)
            .accent_color(EnvironmentColor::Green),
    );

    window.run_event_loop();
}
