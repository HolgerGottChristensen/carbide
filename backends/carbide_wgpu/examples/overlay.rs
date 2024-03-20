use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {

    let mut application = Application::new()
        .with_asset_fonts();

    /*let state = LocalState::new(false);
    let state2 = state.clone();

    let overlay = ZStack::new((
        Rectangle::new()
            .fill(EnvironmentColor::Blue)
            .frame(150.0, 150.0),
        Text::new("Overlay"),
    )).on_click(|env: &mut Environment, _| {
        println!("Overlay clicked!")
    }).on_click_outside(move |env: &mut Environment, _| {
        state2.clone().set_value(false);
    }).overlay("overlay", state.clone());*/


    let widget = Overlay::new("overlay", VStack::new((
        Text::new("Click the rectangle to add element to overlay"),
        ZStack::new((
            Rectangle::new()
                .fill(EnvironmentColor::Green)
                .frame(200.0, 200.0),
            Rectangle::new()
                .fill(EnvironmentColor::Red)
                .frame(100.0, 100.0),
        )).on_click(move |env: &mut Environment, _| {
            println!("Open overlay");
            env.transfer_widget(Some("overlay".to_string()), WidgetTransferAction::Push(
                ZStack::new((
                    Rectangle::new()
                        .fill(EnvironmentColor::Blue)
                        .frame(150.0, 150.0),
                    Text::new("Overlay"),
                )).on_click(|env: &mut Environment, _| {
                    println!("Overlay clicked!")
                }).on_click_outside(move |env: &mut Environment, _| {
                    env.transfer_widget(Some("overlay".to_string()), WidgetTransferAction::Pop)
                }).boxed()
            ))
        }),
        Text::new("Click outside to remove the overlay"),
    ))).steal_events();


    application.set_scene(Window::new(
        "Overlay example - Carbide",
        Dimension::new(600.0, 450.0),
        widget
    ).close_application_on_window_close());

    application.launch();
}
