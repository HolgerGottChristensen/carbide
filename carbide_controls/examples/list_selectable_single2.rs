use carbide_controls::{ControlsExt, List};
use carbide_controls::list::PlainStyle;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, Map1};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let selection = LocalState::new(None);
    let selection2 = selection.clone();

    application.set_scene(Window::new(
        "Single-Selectable List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new_selectable(0..10_000, selection, move |item, _| {
            let background_color = Map1::read_map(selection2.clone(), move |selection| {
                if let Some(selected) = selection && *selected == item  {
                    EnvironmentColor::Blue
                } else {
                    EnvironmentColor::SystemFill
                }
            });

            ZStack::new((
                Rectangle::new().fill(background_color),
                Text::new(item),
            )).frame_fixed_height(20.0)
        })
            .padding(50.0)
            .list_style(PlainStyle(1.0))
    ));

    application.launch();
}
