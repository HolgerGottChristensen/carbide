use carbide_core::application::ApplicationManager;
use carbide_core::closure;
use carbide_core::draw::{Color, Dimension};
use carbide_core::scene::SceneManager;
use carbide_core::state::ReadState;
use carbide_core::widget::{MouseArea, Rectangle, Text, VStack, Widget, WidgetExt};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn window() -> Window<impl ReadState<T=String>, impl Widget> {
        Window::new(
            "ApplicationManager example - Carbide",
            Dimension::new(300.0, 300.0),
            VStack::new((
                Text::new("Click below to open a new window"),
                MouseArea::new(Rectangle::new().fill(Color::random()))
                    .on_click(closure!(|ctx| {
                        ApplicationManager::get(ctx.env_stack, |manager| {
                            manager.add_scene(window())
                        })
                    }))
                    .frame(100.0, 30.0),
                Text::new("Click below to open a new sub window"),
                MouseArea::new(Rectangle::new().fill(Color::random()))
                    .on_click(closure!(|ctx| {
                        SceneManager::get(ctx.env_stack, |manager| {
                            manager.add_sub_scene(window())
                        })
                    }))
                    .frame(100.0, 30.0),
            ))
        )
    }

    application.set_scene(window());

    application.launch()
}