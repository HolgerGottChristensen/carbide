use carbide_core as carbide; // Required only in internal examples
use carbide_core::a;
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
        .on_click(a!(|_, _| {
            *$counter += 1;
        }))
        .frame(100.0, 30.0);

    application.set_scene(
        Window::new("Hello multiple windows", Dimension::new(300.0, 200.0), ZStack::new((
            text,
            *Window::new("Hello from window 2", Dimension::new(300.0, 100.0), button),
            //*Window::new("Hello from window 3", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Green)),
            //*Window::new("Hello from window 4", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Yellow)),
        )))
    );

    application.launch()
}