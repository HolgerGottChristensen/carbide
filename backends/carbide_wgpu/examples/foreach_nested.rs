use carbide_core::color::{ColorExt, RED};
use carbide_core::draw::{Color, Dimension};
use carbide_core::environment::*;
use carbide_core::state::{AnyReadState, AnyState, IndexState, LocalState, ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "ForEach nested example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(
            ForEach::new(0..3, |a, b| {
                ForEach::new(0..2, |c, d| {
                    Rectangle::new()
                        .fill(Color::random())
                        .frame(100.0, 50.0)
                })
            })
        ),
    ));

    application.launch();
}
