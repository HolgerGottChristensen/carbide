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

fn main() {
    env_logger::init();
    let mut window = block_on(Window::new("Hello world 2".to_string(), 800, 800, String::from("Hejsa")));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_press.png").unwrap();

    let sync_state = State::new("K", &"Hello".to_string());

    window.set_widgets(
        VStack::initialize(vec![
            Text::initialize("Hello".into(), vec![]),
            Text::initialize("world! \nHvad sker der i denne verden og vil den layoute rigtigt når der er en lang tekst".into(), vec![]),
            Image::new(rust_image, [100.0,100.0], vec![]),
            Rectangle::initialize(vec![
                SyncTest::new(sync_state)
            ]).fill(GREEN),
            HStack::initialize(vec![
                Image::new(rust_image, [100.0, 100.0], vec![]),
                //ZStack::initialize(vec![
                Rectangle::initialize(vec![])
                    .fill(LIGHT_BLUE)
                    .frame(SCALE, 120.0),
                //Image::new(rust_image, [100.0,100.0], vec![])
                // ])
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
        ])
    );

    window.run_event_loop();

}