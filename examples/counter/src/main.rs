use carbide::{Application, Window, closure};
use carbide::draw::Dimension;
use carbide::state::LocalState;
use carbide::widget::{Text, VStack, WidgetExt};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::environment::EnvironmentFontSize::LargeTitle;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone()).font_size(LargeTitle);

    let button = Button::new("Increase counter", closure!(|_| {
        *$counter += 1;
    })).frame(200.0, 30.0);

    application.set_scene(Window::new(
        "My first counter",
        Dimension::new(300.0, 235.0),
        VStack::new((
            text,
            button
        )).button_style(BorderedProminentStyle)
    ));

    application.launch();
}
