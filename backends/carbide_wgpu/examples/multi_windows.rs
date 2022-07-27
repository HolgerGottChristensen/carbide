use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::prelude::{EnvironmentColor, Rectangle};
use carbide_core::state::LocalState;
use carbide_core::text::FontFamily;
use carbide_core::widget::{MouseArea, Text, WidgetExt, ZStack};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    application.add_font_family(family);

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone()).font_size(EnvironmentFontSize::LargeTitle);

    let button = MouseArea::new(Rectangle::new().fill(EnvironmentColor::Yellow))
        .on_click({
            let counter = counter.clone();

            move |_env: &mut Environment, modifier: carbide_core::event::ModifierKey| {
                use carbide_core::prelude::State;
                let mut counter = counter.clone();

                {
                    let mut counter = counter.value_mut();
                    {
                        *counter = *counter + 1;
                        println!("{}", *counter);
                    }
                }
                counter.update_dependent();
            }
        })
        .frame(100.0, 30.0);

    application.set_scene(
        Window::new("Hello multiple windows", Dimension::new(300.0, 200.0),ZStack::new(vec![
            text,
            Window::new("Hello from window 2", Dimension::new(300.0, 100.0), button),
            //Window::new("Hello from window 3", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Green)),
            //Window::new("Hello from window 4", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Yellow)),
        ]))
    );

    application.launch()
}