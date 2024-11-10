use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{MouseArea, Rectangle, Text, WidgetExt, ZStack};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone()).font_size(EnvironmentFontSize::LargeTitle);

    let button = MouseArea::new(Rectangle::new().fill(EnvironmentColor::Yellow))
        .on_click(closure!(|_| {
            *$counter += 1;
        }))
        .frame(100.0, 30.0);

    application.set_scenes((
        Window::new("Multiple windows example 1 - Carbide", Dimension::new(300.0, 200.0), text),
        Window::new("Multiple windows example 2 - Carbide", Dimension::new(300.0, 100.0), button),
    ));

    application.launch()
}