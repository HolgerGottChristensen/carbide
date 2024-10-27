use carbide::{Application, Window, closure};
use carbide::accessibility::AccessibilityExt;
use carbide::draw::Dimension;
use carbide::state::LocalState;
use carbide::widget::{MouseAreaActionContext, Text, VStack, WidgetExt};
use carbide::controls::{Button};
use carbide::environment::EnvironmentFontSize::LargeTitle;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone()).font_size(LargeTitle);

    let button = Button::new_primary("Increase counter", closure!(|_| {
        *$counter += 1;
    }))
        .frame(200.0, 30.0);

    application.set_scene(Window::new(
        "My first counter",
        Dimension::new(300.0, 235.0),
        VStack::new((
            text,
            button
        ))
    ).close_application_on_window_close());

    application.launch();
}
