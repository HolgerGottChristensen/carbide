use carbide::controls::{ControlsExt, List};
use carbide::controls::list::PlainStyle;
use carbide::color::ColorExt;
use carbide::draw::Dimension;
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "List by content Example - Carbide",
        Dimension::new(400.0, 400.0),
        List::new_content((
            ZStack::new((
                Rectangle::new(),
                Text::new("0"),
            )).frame_fixed_height(20.0),

            ZStack::new((
                Rectangle::new(),
                Text::new("1"),
            )).frame_fixed_height(20.0),

            ZStack::new((
                Rectangle::new(),
                Text::new("2"),
            )).frame_fixed_height(20.0),

            ForEach::new(3..30, |i, _| {
                ZStack::new((
                    Rectangle::new(),
                    Text::new(i).color(EnvironmentColor::Label.color().invert())
                )).frame_fixed_height(20.0)
                    .accent_color(EnvironmentColor::Orange)
            })
        ))
            .border()
            .padding(50.0)
            .list_style(PlainStyle(1.0))
    ));

    application.launch();
}
