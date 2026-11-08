use carbide::controls::identifiable::AnyIdentifiableWidget;
use carbide::controls::ControlsExt;
use carbide::draw::Dimension;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    let widgets = (
        Text::new("Test").tag(10u32),
        Text::new("Test").tag(11u32),

        ForEach::new(vec![12u32, 13u32], |a: &u32, _b| {
            Text::new("Test").tag_state(*a)
        })
    );

    application.set_scene(Window::new(
        "ForEach Widget example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(
            ForEach::custom_widget(widgets, |child: &dyn AnyIdentifiableWidget<T=u32>| {
                HStack::new((
                    Text::new(child.identifier().boxed()),
                    child.as_widget().boxed(),
                )).padding(10.0).border()
            })
        )
            .spacing(10.0),
    ));

    application.launch();
}
