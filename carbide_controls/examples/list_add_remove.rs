use carbide_controls::{Button, List};
use carbide_controls::capture;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::environment::Environment;
use carbide_core::state::{LocalState, State, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "List Add/Remove Example - Carbide",
        800,
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let list_model = (1..10)
        .map(|i| format!("Number {}", i))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);

    fn delegate(item: StringState, _: UsizeState) -> Box<dyn Widget> {
        Rectangle::new(vec![
            Text::new(item)
        ])
            .fill(EnvironmentColor::Green)
            .frame(SCALE, 80.0)
    }

    window.set_widgets(
        VStack::new(vec![
            List::new(list_model_state.clone(), delegate)
                .clip()
                .frame(350.0, 450.0),
            HStack::new(vec![
                Button::new("Add element")
                    .on_click(capture!([list_model_state], |env: &mut Environment| {
                        let len = list_model_state.len();
                        list_model_state.push(format!("New element: {}", len + 1));
                    }))
                    .frame(150.0, 22.0),
                Button::new("Remove element")
                    .on_click(capture!([list_model_state], |env: &mut Environment| {
                        list_model_state.pop();
                    }))
                    .frame(150.0, 22.0)
                    .accent_color(EnvironmentColor::Red),
            ]).spacing(10.0),
            HStack::new(vec![
                Button::new("Add element to start")
                    .on_click(capture!([list_model_state], |env: &mut Environment| {
                        let len = list_model_state.len();
                        list_model_state.insert(0, format!("New element start: {}", len + 1));
                    }))
                    .frame(150.0, 22.0),
                Button::new("Remove first element")
                    .on_click(capture!([list_model_state], |env: &mut Environment| {
                        if list_model_state.len() > 0 {
                            list_model_state.remove(0);
                        }
                    }))
                    .frame(150.0, 22.0)
                    .accent_color(EnvironmentColor::Red),
            ]),
        ]).spacing(10.0),
    );

    window.launch();
}
