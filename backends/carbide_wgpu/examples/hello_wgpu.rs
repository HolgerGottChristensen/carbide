use carbide_wgpu::window::Window;
use futures::executor::block_on;
use carbide_core::window::TWindow;
use carbide_core::state::state::State;
use carbide_core::widget::primitive::v_stack::VStack;
use carbide_core::widget::{Text, Image, Rectangle, HStack, SCALE, Oval};
use carbide_core::widget::complex::SyncTest;
use carbide_core::color::{GREEN, LIGHT_BLUE, RED};
use carbide_core::widget::primitive::widget::WidgetExt;
use carbide_core::widget::primitive::spacer::{Spacer};
use carbide_core::widget::primitive::edge_insets::EdgeInsets;
use carbide_core::widget::primitive::overlaid_layer::OverlaidLayer;
use carbide_core::widget::primitive::scroll::Scroll;
use carbide_core::widget::types::scroll_direction::ScrollDirection;
use carbide_core::widget::types::scale_mode::ScaleMode;
use carbide_core::widget::types::spacer_direction::SpacerDirection;
use carbide_core::widget::primitive::shape::rounded_rectangle::RoundedRectangle;
use carbide_core::widget::primitive::canvas::canvas::Canvas;
use carbide_core::widget::primitive::canvas::context::Context;
use carbide_core::widget::primitive::canvas::context::ContextAction;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = block_on(Window::new("Hello world 2".to_string(), 800, 1200,Some(icon_path), String::from("Hejsa")));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_press.png").unwrap();

    let sync_state = State::new_local("K", &"Hello".to_string());

    window.set_widgets(
        OverlaidLayer::new ("overlay_test",
        VStack::initialize(vec![
            Text::initialize("Hello world!".into()),
            Text::initialize("Hvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst og der ikke er nok plads til at det hele kan være på en linje".into()),
            Image::new(rust_image),
            Rectangle::initialize(vec![
                SyncTest::new(sync_state)
            ]).fill(GREEN),
            HStack::initialize(vec![
                RoundedRectangle::initialize(vec![]).frame(100.0, 100.0),
                Canvas::initialize(Context { actions: vec![
                    ContextAction::MoveTo([75.0, 40.0]),
                    ContextAction::CubicBezierTo{ctrl1: [75.0, 37.0], ctrl2: [70.0, 25.0], to: [50.0, 25.0]},
                    ContextAction::CubicBezierTo{ctrl1: [20.0, 25.0], ctrl2: [20.0, 62.5], to: [20.0, 62.5]},
                    ContextAction::CubicBezierTo{ctrl1: [20.0, 80.0], ctrl2: [40.0, 102.0], to: [75.0, 120.0]},
                    ContextAction::CubicBezierTo{ctrl1: [110.0, 102.0], ctrl2: [130.0, 80.0], to: [130.0, 62.5]},
                    ContextAction::CubicBezierTo{ctrl1: [130.0, 62.5], ctrl2: [130.0, 25.0], to: [100.0, 25.0]},
                    ContextAction::CubicBezierTo{ctrl1: [85.0, 25.0], ctrl2: [75.0, 37.0], to: [75.0, 40.0]},
                    ContextAction::Close
                    ]}).frame(150.0, 150.0),
                Rectangle::initialize(vec![
                    Scroll::new(
                        Image::new(rust_image)
                            .resizeable()
                            .aspect_ratio(ScaleMode::Fill)
                            .frame(800.0, 500.0)
                    ).set_scroll_direction(ScrollDirection::Both)
                        .clip()
                ]).fill(LIGHT_BLUE).frame(SCALE, 200.0),
            ]),
            HStack::initialize(vec![
                Spacer::new(SpacerDirection::Horizontal),
                Oval::initialize(vec![])
                    .fill(RED)
                    .padding(EdgeInsets::all(10.0))
                    .frame(150.0, 150.0),
                Spacer::new(SpacerDirection::Horizontal),
                Spacer::new(SpacerDirection::Horizontal)
            ]),
        ])),
    );

    window.run_event_loop();

}