extern crate carbide_wgpu;
extern crate futures;
extern crate env_logger;
#[macro_use]
extern crate carbide_core;

use carbide_core::widget::*;
use carbide_wgpu::window::Window;
use futures::executor::block_on;
use carbide_controls::PopUpButton;
use carbide_core::color::RED;
use carbide_core::state::environment_color::EnvironmentColor;
use serde::Serialize;
use serde::Deserialize;
use self::Day::{Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Default for Day {
    fn default() -> Self {
        Monday
    }
}

fn main() {
    env_logger::init();

    let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");

    let mut window = block_on(Window::new("Pop up Button Example - Carbide".to_string(), 800, 1200,Some(icon_path), 0));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();


    let selected_index = CommonState::new_local_with_key(&0);

    let selected_model = CommonState::new_local_with_key(&vec![
        Monday,
        Tuesday,
        Wednesday,
        Thursday,
        Friday,
        Saturday,
        Sunday,
    ]);

    window.set_widgets(
        PopUpButton::new(Box::new(selected_model), Box::new(selected_index))
            .frame(235.0.into(), 100.0.into())
    );

    window.run_event_loop();

}