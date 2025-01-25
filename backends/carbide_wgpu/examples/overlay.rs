use carbide_core::color::ColorExt;
use carbide_core::draw::{Color, Dimension};
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let overlay = ZStack::new((
        Rectangle::new()
            .frame(200.0, 200.0),
        Text::new("Overlay"),
    )).on_click(|_| {
        println!("Overlay clicked!")
    }).on_click_outside(move |ctx| {
        OverlayManager::get::<OverlayKey>(ctx.env, |manager| {
            manager.clear();
        })
    });

    let widget = VStack::new((
        Text::new("Click the rectangle to add an element to the overlay"),
        Rectangle::new()
            .fill(EnvironmentColor::Red)
            .frame(150.0, 50.0)
            .on_click(move |ctx| {
                OverlayManager::get::<OverlayKey>(ctx.env, |manager| {
                    manager.insert(overlay.clone().accent_color(Color::random().with_lightness(0.3)));
                })
            }),
        Text::new("Click outside the overlay to remove it"),
    )).spacing(30.0);

    application.set_scene(Window::new(
        "Overlay example - Carbide",
        Dimension::new(400.0, 300.0),
        widget.overlay::<OverlayKey>().steal_events()
    ));

    application.launch();
}

#[derive(Copy, Clone, Debug)]
struct OverlayKey;
impl EnvironmentKey for OverlayKey {
    type Value = OverlayManager;
}