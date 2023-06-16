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
    let selected_item: TState<Option<WidgetId>> = LocalState::new(None);

    fn id_function(item: &(String, WidgetId)) -> WidgetId {
        item.1
    }

    let selected_item_delegate = selected_item.clone();

    let delegate = move |item: TState<(String, WidgetId)>, _: TState<usize>| -> Box<dyn Widget> {
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
        "Single-Selectable List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(list_model_state, delegate)
            .selectable(id_function, selected_item)
            .frame(350.0, 500.0),
    ).close_application_on_window_close());

    application.launch();
}
