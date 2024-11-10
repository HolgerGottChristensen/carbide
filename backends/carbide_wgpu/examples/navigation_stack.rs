use carbide_controls::Button;
use carbide_core::draw::Color;
use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn item(n: usize) -> impl Widget {
        ZStack::new((
            Rectangle::new().fill(Color::random()),
            VStack::new((
                Text::new(format!("Index: {}", n)),

                Button::new_primary("Push", move |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.push(item(n + 1));
                    });
                }).frame(80.0, 22.0),

                Button::new_primary("Push 2", move |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.extend(vec![item(n + 1), item(n + 2)])
                    })
                }).frame(80.0, 22.0),

                Button::new_primary("Pop", |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.pop();
                    })
                }).frame(80.0, 22.0),

                Button::new_primary("Pop 2", |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.pop_n(2);
                    })
                }).frame(80.0, 22.0),

                Button::new_primary("Pop all", |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.pop_all();
                    })
                }).frame(80.0, 22.0),

                Button::new_primary("Replace", move |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.replace(item(n));
                    })
                }).frame(80.0, 22.0),

                Button::new_primary("Replace all", move |ctx| {
                    NavigationManager::get(ctx.env_stack, |manager| {
                        manager.replace_all(item(0));
                    })
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
