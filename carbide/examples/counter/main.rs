use carbide::{Application, Window};
use carbide::draw::Dimension;
use carbide::environment::{EnvironmentFontSize};
use carbide::state::LocalState;
use carbide::widget::{Text, VStack, WidgetExt};
use carbide_controls::{Button};
use carbide_core::a;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone())
        .font_size(EnvironmentFontSize::LargeTitle);

    let button = Button::new_primary("Increase counter", a!(|_, _| {
        *$counter += 1;
    }))
        .frame(200.0, 30.0)
        .boxed();

    application.set_scene(Window::new(
        "My first counter",
        Dimension::new(235.0, 300.0),
        VStack::new(vec![
            text,
            button,
        ])
    ).close_application_on_window_close());

    application.launch()
}
