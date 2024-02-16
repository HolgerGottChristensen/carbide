use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::{ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new().with_asset_fonts();

    fn delegate(_: impl State<T=u32>, _: impl ReadState<T=usize>) -> impl Widget {
        Proxy::new((
            Rectangle::new().fill(EnvironmentColor::Blue).frame(100.0, 50.0),
            Rectangle::new().fill(EnvironmentColor::Orange).frame(100.0, 25.0),
        ))
    }

    application.set_scene(Window::new(
        "Proxy example",
        Dimension::new(600.0, 450.0),
        VStack::new(ForEach::new(1..5, delegate)).spacing(10.0),
    ).close_application_on_window_close());

    application.launch();
}
