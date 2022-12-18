use carbide_controls::Button;
use carbide_core::draw::Dimension;
use carbide_core::focus::Focus;
use carbide_core::state::{LocalState, State, StateExt};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let hover_state = LocalState::new(false);
    let pressed_state = LocalState::new(false);
    let focus_state = LocalState::new(Focus::Focused);
    let counter_state = LocalState::new(0);
    let button_counter_state = counter_state.clone();

    application.set_scene(Window::new(
        "Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            Button::new("Add 1")
                .on_click(move |_: &mut _, _: _| {
                    let mut temp = button_counter_state.clone();
                    *temp.value_mut() += 1;
                })
                .hover(hover_state.clone())
                .pressed(pressed_state.clone())
                .focused(focus_state.clone())
                .frame(60.0, 22.0),
            Text::new(counter_state.mapped(|count: &i32| format!("Count: {}", count)))
                .font_size(40),
            Text::new(hover_state.mapped(|hover: &bool| format!("Hovered: {}", hover)))
                .font_size(40),
            Text::new(pressed_state.mapped(|press: &bool| format!("Pressed: {}", press)))
                .font_size(40),
            Text::new(focus_state.mapped(|focus: &Focus| format!("Focus: {:?}", focus)))
                .font_size(40),
        ])
            .spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
