use carbide_core::draw::{Dimension};
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {

    let mut application = Application::new()
        .with_asset_fonts();

    let (overlay, hierarchy) = ZStack::new(vec![
        Rectangle::new()
            .fill(EnvironmentColor::Blue)
            .frame(150.0, 150.0),
        Text::new("Overlay"),
    ]).on_click(|env: &mut Environment, _| {
        println!("Overlay clicked!")
    }).overlay();

    let overlay2 = overlay.clone();

    let widget = OverlaidLayer::new("overlay", *VStack::new(vec![
        Text::new("Click the rectangle to add element to overlay"),
        Box::new(ZStack::new(vec![
            Box::new(hierarchy),
            Rectangle::new()
                .fill(EnvironmentColor::Green)
                .frame(200.0, 200.0),
            Rectangle::new()
                .fill(EnvironmentColor::Red)
                .frame(100.0, 100.0),
        ]).on_click(move |env: &mut Environment, _| {
            let overlay = overlay.clone();
            if !env.contains_overlay("overlay", overlay.id()) {
                env.add_overlay("overlay", Box::new(overlay));
                println!("Added to overlay");
            }
        }).on_click_outside(move |env: &mut Environment, _| {
            let overlay = overlay2.clone();
            if env.contains_overlay("overlay", overlay.id()) {
                env.remove_overlay("overlay", overlay.id());
                println!("Removed from overlay")
            }
        })),
        Text::new("Click outside to remove the overlay"),
    ]));//.steal_events();


    application.set_scene(Window::new(
        "Overlay example - Carbide",
        Dimension::new(600.0, 450.0),
        Box::new(widget)
    ).close_application_on_window_close());

    application.launch();
}
