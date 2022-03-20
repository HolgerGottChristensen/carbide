use std::collections::HashSet;
use carbide_controls::List;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::{Color, lens};
use carbide_core::state::{LocalState, Map2, ReadState, State, StateExt, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Single-Selectable List Example - Carbide",
        800,
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let list_model = (1..20)
        .map(|i| (format!("Number {}", i), Id::new_v4()))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);
    let selected_item: TState<Option<Id>> = LocalState::new(None);

    fn id_function(item: &(String, Id)) -> Id { item.1 }

    let selected_item_delegate = selected_item.clone();

    let delegate = move |item: TState<(String, Id)>, _: UsizeState| -> Box<dyn Widget> {

        let selected = Map2::read_map(selected_item_delegate.clone(), item.clone(),
          |selection: &Option<Id>, item: &(String, Id)| {
              selection == &Some(id_function(item))
          }).ignore_writes();

        let background_color: TState<Color> = selected
            .choice(EnvironmentColor::Blue.state(), EnvironmentColor::SystemFill.state())
            .ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(lens!((String, Id); |item| {item.0.clone()})),
        ]).frame(0.0, 80.0)
            .expand_width()
    };

    window.set_widgets(
        List::new(list_model_state, delegate)
            .selectable(id_function, selected_item)
            .frame(350.0, 500.0),
    );

    window.launch();
}
