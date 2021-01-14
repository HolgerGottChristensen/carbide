use conrod_wgpu::window::Window;
use futures::executor::block_on;
use conrod_core::window::TWindow;
use conrod_core::state::state::State;
use conrod_core::widget::primitive::v_stack::VStack;
use conrod_core::widget::{Text, Image, Rectangle, HStack, SCALE, Oval};
use conrod_core::widget::complex::SyncTest;
use conrod_core::color::{GREEN, LIGHT_BLUE, RED};
use conrod_core::widget::primitive::widget::WidgetExt;
use conrod_core::widget::primitive::spacer::{Spacer, SpacerDirection};
use conrod_core::widget::primitive::edge_insets::EdgeInsets;
use conrod_core::widget::primitive::overlaid_layer::OverlaidLayer;
use conrod_core::widget::primitive::scroll::Scroll;
use conrod_core::widget::types::scroll_direction::ScrollDirection;
use conrod_core::widget::types::scale_mode::ScaleMode;
use conrod_core::widget::primitive::clip::Clip;

fn main() {
    env_logger::init();
    let mut window = block_on(Window::new("Hello world 2".to_string(), 800, 800, String::from("Hejsa")));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_press.png").unwrap();

    let sync_state = State::new_local("K", &"Hello".to_string());

    window.set_widgets(
        OverlaidLayer::new (
        VStack::initialize(vec![
            Text::initialize("Hello".into(), vec![]),
            Text::initialize("world! \nHvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst".into(), vec![]),
            Image::new(rust_image,  vec![]),
            Rectangle::initialize(vec![
                SyncTest::new(sync_state)
            ]).fill(GREEN),
            HStack::initialize(vec![
                Image::new(rust_image,  vec![]),
                Rectangle::initialize(vec![
                    Clip::new(
                        Scroll::new(
                            Image::new(rust_image,  vec![])
                                .resizeable()
                                .aspect_ratio(ScaleMode::Fill)
                                .frame(800.0, 500.0)
                        ).set_scroll_direction(ScrollDirection::Both)
                    ),
                ]).fill(LIGHT_BLUE),
                    //.frame(SCALE, 120.0),
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
        ]),
        "overlay_test"
        ),
    );

    window.run_event_loop();

}