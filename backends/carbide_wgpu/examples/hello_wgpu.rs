use carbide_core::draw::Position;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::widget::*;
use carbide_core::widget::canvas::Context;
use carbide_wgpu::window::*;
use hello::sync_test::SyncTest;

mod hello;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Hello world 2".to_string(),
        400,
        600,
        Some(icon_path),
    );

    window
        .add_font("fonts/NotoSans/NotoSans-Regular.ttf")
        .unwrap();
    let rust_image = window.add_image_from_path("images/rust_press.png").unwrap();

    let sync_state = CommonState::new_local_with_key(&"Hello".to_string());

    window.set_widgets(
        OverlaidLayer::new("overlay_test",
                           VStack::new(vec![
                               Text::new("Hello world!")
                                   .font_size(EnvironmentFontSize::Title)
                                   .color(EnvironmentColor::Green)
                                   .padding(EdgeInsets::all(10.0)),
                               Text::new("Hvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst og der ikke er nok plads til at det hele kan være på en linje")
                                   .padding(EdgeInsets::all(10.0)).border(),
                               Image::new(rust_image),
                               ZStack::new(vec![
                                   Rectangle::new().fill(EnvironmentColor::SecondarySystemBackground),
                                   SyncTest::new(sync_state),
                               ]),
                               HStack::new(vec![
                                   RoundedRectangle::new(25.0)
                                       .frame(100.0, 100.0),
                                   Canvas::initialize(draw_heart)
                                       .frame(150.0, 150.0),
                                   ZStack::new(vec![
                                       Rectangle::new().fill(EnvironmentColor::SecondarySystemBackground),
                                       Scroll::new(
                                           Image::new(rust_image)
                                               .resizeable()
                                               .aspect_ratio(ScaleMode::Fill)
                                               .frame(800.0, 500.0)
                                       ).with_scroll_direction(ScrollDirection::Both)
                                           .clip(),
                                   ]).frame(0.0, 200.0)
                                       .expand_width(),
                               ]).padding(EdgeInsets::all(10.0)),
                               HStack::new(vec![
                                   Spacer::new(),
                                   Ellipse::new()
                                       .padding(EdgeInsets::all(10.0))
                                       .frame(150.0, 150.0),
                                   Spacer::new(),
                                   Spacer::new(),
                               ]),
                           ])),
    );

    window.launch();
}

fn draw_heart<GS: GlobalStateContract>(_: OldRect, mut context: Context) -> Context {
    context.move_to(75.0, 40.0);
    context.bezier_curve_to(Position::new(75.0, 37.0), Position::new(70.0, 25.0), Position::new(50.0, 25.0));
    context.bezier_curve_to(Position::new(20.0, 25.0), Position::new(20.0, 62.5), Position::new(20.0, 62.5));
    context.bezier_curve_to(Position::new(20.0, 80.0), Position::new(40.0, 102.0), Position::new(75.0, 120.0));
    context.bezier_curve_to(Position::new(110.0, 102.0), Position::new(130.0, 80.0), Position::new(130.0, 62.5));
    context.bezier_curve_to(Position::new(130.0, 62.5), Position::new(130.0, 25.0), Position::new(100.0, 25.0));
    context.bezier_curve_to(Position::new(85.0, 25.0), Position::new(75.0, 37.0), Position::new(75.0, 40.0));
    context.close_path();
    context.set_fill_style(EnvironmentColor::Accent);
    context.fill();

    context
}
