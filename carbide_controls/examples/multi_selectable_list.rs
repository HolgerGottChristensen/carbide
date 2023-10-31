use std::collections::HashSet;
use carbide_controls::{List};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{AnyState, LocalState, Map2};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let list_model = (1..100)
        .map(|i| format!("Number {}", i))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);

    let selected_item: LocalState<HashSet<String>> = LocalState::new(HashSet::new());
    let selected_item_clone = selected_item.clone();

    let delegate = move |item: Box<dyn AnyState<T=String>>, _| -> Box<dyn AnyWidget> {
        let background_color = Map2::read_map(item.clone(), selected_item_clone.clone(), |item, selected| {
            if selected.contains(item) {
                EnvironmentColor::Blue
            } else {
                EnvironmentColor::SystemFill
            }
        });

        ZStack::new((
            Rectangle::new().fill(background_color),
            Text::new(item),
        ))
            .frame_fixed_height(80.0)
            .boxed()
    };

    application.set_scene(Window::new(
        "Multi-Selectable List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(list_model_state, delegate)
            .selectable(selected_item)
            .clip()
            .padding(50.0)
    ).close_application_on_window_close());

    application.launch();
}
