use carbide_controls::identifiable::AnyIdentifiableWidget;
use carbide_controls::ControlsExt;
use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use carbide_core::utils::clone_box;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn delegate(child: &dyn AnyIdentifiableWidget<u32>) -> impl Widget {
        HStack::new((
            Text::new(clone_box(child.identifier())),
            clone_box(child),
        )).padding(10.0).border()
    }

    application.set_scene(Window::new(
        "ForEach Widget example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(ForEach::custom_widget(
            (
                Text::new("Test").tag(10u32),
                Text::new("Test").tag(11u32),

                ForEach::new(vec![12u32, 13u32], |a, b| {
                    Text::new("Test").tag(a)
                })
            ),
            delegate,
        ))
            .spacing(10.0),
    ));

    application.launch();
}
