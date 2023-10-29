use carbide_controls::{Button, TextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor};
use carbide_core::{a, matches_case};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide; // Required only in internal examples

fn main() {

    let mut application = Application::new()
        .with_asset_fonts();

    let integer_state = LocalState::new(0);

    let middle = ZStack::new((
        Rectangle::new().fill(EnvironmentColor::Yellow),
        TextInput::new(LocalState::new("Hello world!".to_string())).padding(30.0),
    )).boxed();


    application.set_scene(Window::new(
        "Match example",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Match::new(integer_state.clone())
                .case((
                    |a| matches!(a, 0),
                    Rectangle::new().fill(EnvironmentColor::Blue).boxed(),
                ))
                .case((|a| matches!(a, 1), middle))
                .case(matches_case!(integer_state, x, x => Rectangle::new().fill(EnvironmentColor::Red).boxed())),
            Button::new_primary("Click to change the view above", a!(|_, _| {
                *$integer_state = (*$integer_state + 1) % 3;
            })).frame_fixed_height(45.0),
        ))
    ).close_application_on_window_close());


    application.launch();
}