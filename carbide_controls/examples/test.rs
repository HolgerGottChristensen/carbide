use dyn_clone::clone_box;
use carbide_controls::ControlsExt;
use carbide_controls::identifiable::{AnyIdentifiableWidget, IdentifiableWidget};
use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn delegate(child: impl IdentifiableWidget<u32>) -> impl Widget {
        HStack::new((
            Text::new(clone_box(child.identifier())),
            child,
        )).padding(10.0).border()
    }

    application.set_scene(Window::new(
        "ForEach Widget example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(ForEach::widget_custom(
            (
                Text::new("Test").tag(10u32),
                //Text::new("Test").tag(11u32),
                ForEach::new(vec![11u32, 12u32], |a, b| {
                    Text::new("Test").tag(a)
                })
            ),
            delegate,
        ))
            .spacing(10.0),
    ));

    application.launch();
}
