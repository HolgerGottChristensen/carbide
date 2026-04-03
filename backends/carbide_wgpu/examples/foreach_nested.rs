use carbide_core::color::ColorExt;
use carbide_core::draw::{Color, Dimension};
use carbide_core::state::Map1;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "ForEach nested example - Carbide",
        Dimension::new(350.0, 600.0),
        VStack::new(
            ForEach::new(0..3, |a, b| {
                let group_color = Color::random();

                ForEach::new(0..3, move |c, d| {
                    let element_color = Map1::read_map(d, move |d| group_color.lightened(*d as f32 / 7.0));

                    Rectangle::new()
                        .fill(element_color)
                        .frame(100.0, 50.0)
                })
            })
        ),
    ));

    application.launch();
}
