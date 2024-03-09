use carbide_controls::Button;
use carbide_core::draw::Color;
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, WidgetTransferAction};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn item(n: usize) -> Box<dyn AnyWidget> {
        ZStack::new((
            Rectangle::new().fill(Color::random()),
            VStack::new((
                Text::new(format!("Index: {}", n)),

                Button::new_primary("Push", move |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::Push(item(n + 1)))
                }).frame(80.0, 22.0),

                Button::new_primary("Push 2", move |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::PushVec(vec![item(n + 1), item(n + 2)]))
                }).frame(80.0, 22.0),

                Button::new_primary("Pop", |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::Pop)
                }).frame(80.0, 22.0),

                Button::new_primary("Pop 2", |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::PopN(2))
                }).frame(80.0, 22.0),

                Button::new_primary("Pop all", |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::PopAll)
                }).frame(80.0, 22.0),

                Button::new_primary("Replace", move |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::Replace(item(n)))
                }).frame(80.0, 22.0),

                Button::new_primary("Replace all", move |env: &mut Environment, _: _| {
                    env.transfer_widget(None, WidgetTransferAction::ReplaceAll(item(0)))
                }).frame(80.0, 22.0),
            ))
            .spacing(10.0),
        )).boxed()
    }

    application.set_scene(Window::new(
        "NavigationStack - Carbide",
        Dimension::new(300.0, 300.0),
        NavigationStack::new(item(0))
    ).close_application_on_window_close());

    application.launch();
}
