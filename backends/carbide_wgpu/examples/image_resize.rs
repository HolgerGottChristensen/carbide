use std::path::PathBuf;
use carbide_core::prelude::EnvironmentColor;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {

    let icon_path = PathBuf::from("/Users/holgerchristensen/carbide/assets/images/rust.png");//Window::relative_path_to_assets("images/rust.png");
    println!("{:?}", icon_path);

    let mut window = Window::new(
        "Materials example".to_string(),
        800,
        600,
        Some(icon_path.clone()),
    );

    /*et family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf",
    ]);
    window.add_font_family(family);*/


    let dynamic_image = image::open("/Users/holgerchristensen/carbide/assets/images/lcabyg.png").expect("Couldn't load logo");
    let lcabyg_icon_id = window.add_image(dynamic_image);


    window.set_widgets(Image::new(lcabyg_icon_id).resizeable());

    window.launch();
}