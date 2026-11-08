// use carbide::controls::{ControlsExt, List};
// use carbide::draw::Dimension;
// use carbide::environment::EnvironmentColor;
// use carbide::state::{LocalState, Map1};
// use carbide::widget::*;
// use carbide::{Application, Window};
// use std::collections::HashSet;
// use carbide::controls::list::PlainStyle;
//
// fn main() {
//     let mut application = Application::new();
//
//     let selection = LocalState::new(HashSet::new());
//     let selection2 = selection.clone();
//
//     application.set_scene(Window::new(
//         "Multi-Selectable List Example - Carbide",
//         Dimension::new(400.0, 600.0),
//         List::new(0..10_000, move |item, _| {
//             let background_color = Map1::read_map(selection2.clone(), move |selection| {
//                 if selection.contains(&item)  {
//                     EnvironmentColor::Blue
//                 } else {
//                     EnvironmentColor::SystemFill
//                 }
//             });
//
//             ZStack::new((
//                 Rectangle::new().fill(background_color),
//                 Text::new(item),
//             )).frame_fixed_height(20.0)
//                 .tag(item)
//         }).selection(selection)
//             .padding(50.0)
//             .list_style(PlainStyle(1.0))
//     ));
//
//     application.launch();
// }
