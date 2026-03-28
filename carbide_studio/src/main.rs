use carbide::{closure, Application, Window};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::draw::Dimension;
use carbide::environment::EnvironmentColor::{Green, Red, Yellow};
use carbide::state::LocalState;
use carbide::widget::{HSplit, Image, Rectangle, Spacer, WidgetExt, ZStack};
use crate::widgets::{Toolbar, WidgetOutline, WidgetViewer};

mod widgets;

fn main() {
    let mut application = Application::new();

    let mut state = LocalState::new(Rectangle::new().boxed());

    application.set_scene(Window::new(
        "Carbide Studio",
        Dimension::new(900.0, 600.0),
        Toolbar::new(
            (
                Button::new(Image::system("image"), closure!(|_| {
                    *$state = Image::new("images/landscape.png").resizeable().boxed();
                })).aspect_ratio(Dimension::new(1.0, 1.0)),
                Button::new(Image::system("rows-3"), closure!(|_| {})).aspect_ratio(Dimension::new(1.0, 1.0)),
                Button::new(Image::system("columns-3"), closure!(|_| {})).aspect_ratio(Dimension::new(1.0, 1.0)),
                Spacer::new()
            ),
            HSplit::new(
                WidgetViewer::new(state.clone())
                    .border()
                    .padding(30.0)
                    .border(),
                ZStack::new((
                    Rectangle::new().fill(Red),
                    WidgetOutline::new(state)
                ))
            ).relative_to_end(LocalState::new(300.0))
                .boxed()
        )
            .button_style(BorderedProminentStyle)
    ));

    application.launch();
}
