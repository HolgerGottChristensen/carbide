use carbide_core::prelude::EnvironmentColor;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {

    let icon_path = Window::relative_path_to_assets("images/rust.png");

    let mut window = Window::new(
        "Materials example".to_string(),
        800,
        600,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf",
    ]);
    window.add_font_family(family);

    let lcabyg_icon_id = window.add_image_from_path("images/lcabyg.png");


    window.set_widgets(Image::new_icon(lcabyg_icon_id).resizeable());

    window.launch();
}