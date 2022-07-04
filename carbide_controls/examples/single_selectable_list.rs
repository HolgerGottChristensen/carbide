use carbide_controls::List;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{
    LocalState, Map2, ReadState, State, StateExt, StringState, TState, UsizeState,
};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_core::{lens, Color};
use carbide_wgpu::window::Window;
use std::collections::HashSet;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Single-Selectable List Example - Carbide",
        400,
        600,
        Some(icon_path),
    );

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let list_model = (1..20)
        .map(|i| (format!("Number {}", i), WidgetId::new()))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);
    let selected_item: TState<Option<WidgetId>> = LocalState::new(None);

    fn id_function(item: &(String, WidgetId)) -> WidgetId {
        item.1
    }

    let selected_item_delegate = selected_item.clone();

    let delegate = move |item: TState<(String, WidgetId)>, _: UsizeState| -> Box<dyn Widget> {
        let selected = Map2::read_map(
            selected_item_delegate.clone(),
            item.clone(),
            |selection: &Option<WidgetId>, item: &(String, WidgetId)| {
                selection == &Some(id_function(item))
            },
        )
        .ignore_writes();

        let background_color: TState<Color> = selected
            .choice(
                EnvironmentColor::Blue.state(),
                EnvironmentColor::SystemFill.state(),
            )
            .ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(lens!((String, WidgetId); |item| {item.0.clone()})),
        ])
        .frame(0.0, 80.0)
        .expand_width()
    };

    window.set_widgets(
        List::new(list_model_state, delegate)
            .selectable(id_function, selected_item)
            .frame(350.0, 500.0),
    );

    window.launch();
}
