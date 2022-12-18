use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::{ReadState, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn delegate(item: TState<EnvironmentColor>, index: TState<usize>) -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(item.value().clone()),
            Text::new(index).font_size(EnvironmentFontSize::LargeTitle),
        ])
        .frame(100.0, 50.0)
    }

    application.set_scene(Window::new(
        "Foreach example",
        Dimension::new(600.0, 450.0),
        VStack::new(vec![ForEach::new(
            vec![
                EnvironmentColor::Red,
                EnvironmentColor::Orange,
                EnvironmentColor::Yellow,
                EnvironmentColor::Green,
                EnvironmentColor::Accent,
                EnvironmentColor::Purple,
            ],
            delegate,
        )])
            .spacing(10.0),
    ).close_application_on_window_close());

    application.launch();
}
