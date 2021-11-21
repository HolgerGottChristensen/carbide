use carbide_controls::{Button, capture, List, NavigationStack};
use carbide_core::Color;
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide_core::prelude::WidgetTransferAction;
use carbide_core::state::{LocalState, State, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "NavigationStack Example - Carbide",
        600,
        600,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    fn item(n: usize) -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(Color::random()),
            VStack::new(vec![
                Text::new(format!("Index: {}", n)),
                Button::new("Push")
                    .on_click(move |env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::Push(
                            item(n + 1)
                        ))
                    })
                    .frame(80, 22),
                Button::new("Push 2")
                    .on_click(move |env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::PushVec(
                            vec![item(n + 1), item(n + 2)]
                        ))
                    })
                    .frame(80, 22),
                Button::new("Pop")
                    .on_click(|env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::Pop)
                    })
                    .frame(80, 22),
                Button::new("Pop 2")
                    .on_click(|env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::PopN(2))
                    })
                    .frame(80, 22),
                Button::new("Pop all")
                    .on_click(|env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::PopAll)
                    })
                    .frame(80, 22),
                Button::new("Replace")
                    .on_click(move |env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::Replace(
                            item(n)
                        ))
                    })
                    .frame(80, 22),
                Button::new("Replace all")
                    .on_click(move |env: &mut Environment| {
                        env.transfer_widget(None, WidgetTransferAction::ReplaceAll(
                            item(0)
                        ))
                    })
                    .frame(80, 22),
            ]).spacing(10.0),
        ])
    }

    window.set_widgets(
        NavigationStack::new(item(0)),
    );

    window.launch();
}
