use carbide_core::draw::Dimension;
use carbide_core::state::AnyState;
use carbide_core::widget::*;
use carbide_icons::all_icon_names;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Lucide example - Carbide",
        Dimension::new(800.0, 600.0),
        Scroll::new(VGrid::new(ForEach::new(all_icon_names(), |name: &String, _| ZStack::new((
            Rectangle::new(),
            VStack::new((
                Image::system(name.clone()),
                Text::new(name.clone())
            )).cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(3.0)
                .padding(10.0)
        ))), vec![
            VGridColumn::Adaptive(130.0)
        ]).spacing(Dimension::new(5.0, 5.0))).padding(10.0),
    ));

    application.launch();
}
