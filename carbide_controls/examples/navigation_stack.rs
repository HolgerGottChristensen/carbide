use carbide_controls::{Button, NavigationStack};
use carbide_core::Color;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, WidgetTransferAction};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn item(n: usize) -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(Color::random()),
            VStack::new(vec![
                Text::new(format!("Index: {}", n)),
                Button::new("Push")
                    .on_click(move |env: &mut Environment, _: _| {
                        env.transfer_widget(None, WidgetTransferAction::Push(item(n + 1)))
                    })
                    .frame(80, 22),
                Button::new("Push 2")
                    .on_click(move |env: &mut Environment, _: _| {
                        env.transfer_widget(
                            None,
                            WidgetTransferAction::PushVec(vec![item(n + 1), item(n + 2)]),
                        )
                    })
                    .frame(80, 22),
                Button::new("Pop")
                    .on_click(|env: &mut Environment, _: _| {
                        env.transfer_widget(None, WidgetTransferAction::Pop)
                    })
                    .frame(80, 22),
                Button::new("Pop 2")
                    .on_click(|env: &mut Environment, _: _| {
                        env.transfer_widget(None, WidgetTransferAction::PopN(2))
                    })
                    .frame(80, 22),
                Button::new("Pop all")
                    .on_click(|env: &mut Environment, _: _| {
                        env.transfer_widget(None, WidgetTransferAction::PopAll)
                    })
                    .frame(80, 22),
                Button::new("Replace")
                    .on_click(move |env: &mut Environment, _: _| {
                        env.transfer_widget(None, WidgetTransferAction::Replace(item(n)))
                    })
                    .frame(80, 22),
                Button::new("Replace all")
                    .on_click(move |env: &mut Environment, _: _| {
                        env.transfer_widget(None, WidgetTransferAction::ReplaceAll(item(0)))
                    })
                    .frame(80, 22),
            ])
            .spacing(10.0),
        ])
    }

    application.set_scene(Window::new(
        "NavigationStack Example - Carbide",
        Dimension::new(300.0, 300.0),
        NavigationStack::new(item(0))
    ).close_application_on_window_close());

    application.launch();
}
