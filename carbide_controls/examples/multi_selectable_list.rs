use std::collections::HashSet;

use carbide_controls::List;
use carbide_core::{Color, lens};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{
    LocalState, Map2, StateExt, TState,
};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model = (1..20)
        .map(|i| (format!("Number {}", i), WidgetId::new()))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);
    let selected_items: TState<HashSet<WidgetId>> = LocalState::new(HashSet::new());

    fn id_function(item: &(String, WidgetId)) -> WidgetId {
        item.1
    }

    let selected_items_delegate = selected_items.clone();

    let delegate = move |item: TState<(String, WidgetId)>, _: TState<usize>| -> Box<dyn Widget> {
        let selected = Map2::read_map(
            selected_items_delegate.clone(),
            item.clone(),
            |map: &HashSet<WidgetId>, item: &(String, WidgetId)| map.contains(&id_function(item)),
        )
        .ignore_writes();

        let background_color: TState<Color> = selected
            .choice(
                EnvironmentColor::Blue.color(),
                EnvironmentColor::SystemFill.color(),
            )
            .ignore_writes();

        ZStack::new(vec![
            Rectangle::new().fill(background_color),
            Text::new(lens!((String, WidgetId); |item| {item.0.clone()})),
        ])
        .frame(0.0, 80.0)
        .expand_width()
    };

    application.set_scene(Window::new(
        "Multi-Selectable List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(list_model_state, delegate)
            .selectable(id_function, selected_items)
            .clip()
            .frame(350.0, 500.0),
    ).close_application_on_window_close());

    application.launch();
}
