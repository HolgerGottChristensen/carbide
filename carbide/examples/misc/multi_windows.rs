use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::closure;
use carbide::draw::Dimension;
use carbide::environment::EnvironmentFontSize;
use carbide::state::LocalState;
use carbide::widget::{Text, WidgetExt};
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone()).font_size(EnvironmentFontSize::LargeTitle);

    let button = Button::new("Increase counter", closure!(|_| {
            *$counter += 1;
        }))
        .button_style(BorderedProminentStyle)
        .frame(130.0, 26.0);

    application.set_scenes((
        Window::new("Multiple windows example 1 - Carbide", Dimension::new(300.0, 200.0), text),
        Window::new("Multiple windows example 2 - Carbide", Dimension::new(300.0, 100.0), button),
    ));

    application.launch()
}