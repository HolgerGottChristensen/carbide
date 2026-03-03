use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::widget::{Rectangle, WidgetExt};
use carbide_table::Table;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Table example - Carbide",
            Dimension::new(600.0, 600.0),
            Table::new()
                //.border()
                //.padding(50.0)

        )
    );

    application.launch()
}