use carbide::draw::Dimension;
use carbide::widget::WidgetExt;
use carbide::table::Table;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Table example - Carbide",
            Dimension::new(600.0, 600.0),
            Table::new()
                //.border()
                .clip()
                .padding(50.0)

        )
    );

    application.launch()
}