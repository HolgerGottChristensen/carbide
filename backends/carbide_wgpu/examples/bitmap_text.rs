use carbide_core::text::PolarBearMarkup;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Hello world 2".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    let mut noto_family = FontFamily::new("NotoSans");
    noto_family.add_font("fonts/NotoSans/NotoSans-Regular.ttf", FontWeight::Normal, FontStyle::Normal);
    noto_family.add_font("fonts/NotoSans/NotoSans-Italic.ttf", FontWeight::Normal, FontStyle::Italic);
    noto_family.add_font("fonts/NotoSans/NotoSans-Bold.ttf", FontWeight::Bold, FontStyle::Normal);
    window.add_font_family(noto_family);

    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_bitmap_font("/System/Library/Fonts/Apple Color Emoji.ttc", FontWeight::Normal, FontStyle::Normal);
    window.add_font_family(family);

    window.set_widgets(
        Text::new_with_generator("# Rich text\nHello *world*, this is /italic/, _underlined_ and -striked-. We can even show 😀, and we support a list of fallback fonts!", PolarBearMarkup::new())
            .border()
            .border_width(1)
            .color(EnvironmentColor::Green)
            .padding(EdgeInsets::all(40.0))
    );

    window.run_event_loop();
}

