// use carbide_core::draw::Dimension;
// use carbide_core::environment::{Environment, EnvironmentFontSize};
// use carbide_core::event::{HotKey, Key, ModifierKey};
// use carbide_core::prelude::{EnvironmentColor, MenuItem, Rectangle};
// use carbide_core::state::LocalState;
// use carbide_core::text::FontFamily;
// use carbide_core::widget::{Menu, MouseArea, Text, WidgetExt, ZStack};
// use carbide_wgpu::{Application, Window};
//
// fn main() {
//     let mut application = Application::new();
//
//     let menu1 = vec![
//         Menu::new("Test1")
//             .item(MenuItem::new("Test1.1", true)
//                 .action(Box::new(|_| { println!("1.1 clicked");}))
//                 .hotkey(HotKey::new(Key::At, ModifierKey::GUI))
//             )
//             .item(MenuItem::new("Test1.2", true).disabled())
//             .separator()
//             .item(MenuItem::new("Test1.3", true)
//                 .hotkey(HotKey::new(Key::N, ModifierKey::ALT_GUI))),
//         Menu::new("Test2")
//             .item(MenuItem::new("Test2.1", true)
//                 .hotkey(HotKey::new(Key::Up, ModifierKey::GUI)))
//             .item(MenuItem::new("Test2.2", true))
//             .item(MenuItem::new("Test2.3", true)),
//     ];
//
//     let menu2 = vec![
//         Menu::new("Test3")
//             .item(MenuItem::new("Test3.1", true))
//             .item(MenuItem::new("Test3.2", true))
//             .item(MenuItem::new("Test3.3", true)),
//         Menu::new("File")
//             .sub_menu(
//                 Menu::new("New")
//                     .item(MenuItem::new("Project", true))
//                     .item(MenuItem::separator())
//                     .item(MenuItem::new("Module", true))
//             )
//             .item(MenuItem::new("Test4.2", true))
//             .item(MenuItem::new("Test4.3", true)),
//     ];
//
//     application.set_scene(
//         Window::new("Look at the window menu", Dimension::new(300.0, 200.0),ZStack::new(vec![
//             Rectangle::new().fill(EnvironmentColor::Yellow),
//             Window::new(
//                 "Different menus for different windows",
//                 Dimension::new(300.0, 100.0),
//                 Rectangle::new()
//             ).menu(menu1),
//         ])).menu(menu2)
//     );
//
//     application.launch()
// }
fn main() {}