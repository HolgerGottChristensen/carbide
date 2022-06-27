use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "ZStack example".to_string(),
        400,
        600,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    window.set_widgets(
        ZStack::new(vec![
            RoundedRectangle::new(10.0),
            Text::new("Hello world!"),
        ]).padding(40.0)
    );

    window.launch();
}
