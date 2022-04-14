use carbide_core::prelude::EnvironmentColor;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Icon example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let image_id = window.add_image_from_path("images/rust.png");

    window.set_widgets(
        VStack::new(vec![
            Image::new_icon(image_id),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(50, 50),
        ]).accent_color(EnvironmentColor::Red)
            .menu(vec![
                Menu::new("File".to_string())
                    .item(MenuItem::new("Item 1".to_string(), None, true, false))
                    .item(MenuItem::separator())
                    .item(MenuItem::new("Item 2".to_string(), None, true, false)),
                Menu::new("Edit".to_string())
                    .item(MenuItem::new("Item 1".to_string(), None, true, false))
                    .item(Menu::new("Hello".to_string()).item(MenuItem::new("Item 78".to_string(), None, true, false)).sub_menu())
                    .item(MenuItem::new("Item 2".to_string(), None, true, false)),
                Menu::new("Very long menu item".to_string())
                    .item(MenuItem::new("Item 1".to_string(), None, true, false))
                    .item(MenuItem::new("Item 2".to_string(), None, true, false)),
            ])
    );

    window.set_menu(vec![
        Menu::new("Test 1".to_string())
            .item(MenuItem::new("Item 1".to_string(), None, true, false))
            .item(MenuItem::new("Item 2".to_string(), None, true, false))
            .item(MenuItem::separator())
            .item(MenuItem::new("Item 3".to_string(), None, false, false))
            .item(MenuItem::separator())
            .item(MenuItem::new("Item 4".to_string(), None, false, false))
            .item(MenuItem::new("Item 4".to_string(), None, false, false))
            .item(MenuItem::separator())
            .item(MenuItem::new("Item 4".to_string(), None, false, false)),

        Menu::new("Test 2".to_string())
            .item(MenuItem::new("Item 5".to_string(), None, true, false))
            .item(MenuItem::new("Item 6".to_string(), None, true, false))
            .item(MenuItem::new("Item 7".to_string(), None, false, false))
            .item(MenuItem::separator())
            .item(MenuItem::new("Item 8".to_string(), None, false, false))
            .item(
                Menu::new("Sub-menu".to_string())
                    .item(MenuItem::new("Item 9".to_string(), None, true, false))
                    .item(MenuItem::new("Item 10".to_string(), None, true, false))
                    .item(MenuItem::new("Item 11".to_string(), None, true, false))
                    .sub_menu()
            ).kind(MenuKind::Help)
    ]);

    window.launch();
}
