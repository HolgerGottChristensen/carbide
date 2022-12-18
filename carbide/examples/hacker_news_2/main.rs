// use carbide_core::draw::Dimension;
// use carbide_core::state::GlobalState;
// use carbide_core::text::FontFamily;
// use carbide_core::widget::{Rectangle, Text};
// use carbide_wgpu::{Application, Window};
//
// fn main() {
//     let mut application = Application::new();
//
//     let mut family = FontFamily::new_from_paths(
//         "NotoSans",
//         vec![
//             "fonts/NotoSans/NotoSans-Regular.ttf",
//             "fonts/NotoSans/NotoSans-Italic.ttf",
//             "fonts/NotoSans/NotoSans-Bold.ttf",
//         ],
//     );
//
//     application.add_font_family(family);
//
//     let env = application.environment_mut();
//
//     let global = GlobalState::new("Hejsa".to_string());
//
//     std::thread::spawn(|| {
//         let test = global.clone();
//         println!("{:?}", test)
//     });
//
//     application.set_scene(Window::new(
//         "Hacker news client 2",
//         Dimension::new(900.0, 500.0),
//         Text::new(global)
//     ).close_application_on_window_close());
//
//     application.launch()
// }
fn main() {}