use carbide_controls::PlainButton;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::focus::Focus;
use carbide_core::state::{
    LocalState, State, StateExt,
};
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

    let counter_text = counter_state
        .map(|count: &i32| format!("Count: {}", count))
        .ignore_writes();
    let focus_text = focus_state
        .map(|focus: &Focus| format!("Focus: {:?}", focus))
        .ignore_writes();
    let hover_text = hover_state
        .map(|hover: &bool| format!("Hovered: {}", hover))
        .ignore_writes();
    let pressed_text = pressed_state
        .map(|press: &bool| format!("Pressed: {}", press))
        .ignore_writes();

    application.set_scene(Window::new(
        "Plain Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            PlainButton::new(Rectangle::new().fill(EnvironmentColor::Accent))
                .on_click(move |_: &mut _, _: _| {
                    let mut temp = button_counter_state.clone();
                    *temp.value_mut() += 1;
                })
                .hovered(hover_state.clone())
                .pressed(pressed_state.clone())
                .focused(focus_state.clone())
                .border()
                .clip()
                .frame(120.0, 70.0),
            Text::new(counter_text).font_size(40),
            Text::new(hover_text).font_size(40),
            Text::new(pressed_text).font_size(40),
            Text::new(focus_text).font_size(40),
        ])
            .spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
