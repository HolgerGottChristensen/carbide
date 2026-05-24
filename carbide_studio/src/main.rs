use carbide::{closure, Application, Window};
use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::draw::Dimension;
use carbide::environment::EnvironmentColor::{Green, Red};
use carbide::state::LocalState;
use carbide::widget::{HSplit, Image, Rectangle, Spacer, WidgetExt, ZStack};
use crate::widgets::{Toolbar, WidgetDetails, WidgetOutline, WidgetViewer};
use self::core::studio_widget_ext::StudioWidgetExt;
use self::core::any_studio_widget::AnyStudioWidget;

mod widgets;
mod core;
mod impls;

fn main() {
    let mut application = Application::new();

    let mut state = LocalState::new(Rectangle::new().studio_boxed());
    let mut editor = LocalState::new(Rectangle::new().editor());

    application.set_scene(Window::new(
        "Carbide Studio",
        Dimension::new(1100.0, 600.0),
        Toolbar::new(
            (
                Button::new(Image::system("image"), closure!(|_| {
                    //*$state = Image::new("images/landscape.png").resizeable().boxed();
                })).aspect_ratio(Dimension::new(1.0, 1.0)),
                Button::new(Image::system("square"), closure!(|_| {
                    *$state = Rectangle::new().studio_boxed();
                })).aspect_ratio(Dimension::new(1.0, 1.0)),
                Button::new(Image::system("rows-3"), closure!(|_| {})).aspect_ratio(Dimension::new(1.0, 1.0)),
                Button::new(Image::system("columns-3"), closure!(|_| {})).aspect_ratio(Dimension::new(1.0, 1.0)),
                Spacer::new()
            ),
            HSplit::new(
                ZStack::new((
                    Rectangle::new().fill(Green),
                    WidgetOutline::new(state.clone())
                )),
                HSplit::new(
                    WidgetViewer::new(state.clone())
                        .border()
                        .padding(30.0)
                        .border(),
                    ZStack::new((
                        Rectangle::new().fill(Red),
                        WidgetDetails::new(editor)
                    ))
                ).relative_to_end(LocalState::new(300.0))
            ).relative_to_start(LocalState::new(200.0)).boxed()
        )
            .button_style(BorderedProminentStyle)
    ));

    application.launch();
}
