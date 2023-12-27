use carbide_controls::Button;
use carbide_core::a;
use carbide_core as carbide; // Required only in internal examples
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentFontSize;
use carbide_core::state::{LocalState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let switch = LocalState::new(false);

    application.set_scene(Window::new(
        "On change - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Text::new(switch.clone())
                .font_size(EnvironmentFontSize::Title),
            Button::new_primary("Change", a!(|_, _| { *$switch = !*$switch; }))
                .frame(96.0, 22.0),
        )).spacing(10.0)
            .on_change(switch.clone(),a!(|old, new| {
                println!("old: {:?}, new: {:?}", old, new);
            }))
    ).close_application_on_window_close());

    application.launch();
}