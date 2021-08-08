use futures::executor::block_on;
use serde::Deserialize;
use serde::Serialize;

use carbide_controls::PlainTextInput;
use carbide_controls::PopUpButton;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

use self::Day::{Friday, Monday, Saturday, Sunday, Thursday, Tuesday, Wednesday};

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

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Focus behavior example - Carbide".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let text_state = CommonState::new_local_with_key(&"Hello World!".to_string());
    let text_state2 = CommonState::new_local_with_key(&"Hej Verden!".to_string());
    let text_state3 = CommonState::new_local_with_key(&"Hallo Welt!".to_string());
    let text_state4 = CommonState::new_local_with_key(&"Ciao mondo!".to_string());
    //let text_state5 = CommonState::new_local_with_key(&"Bonjour monde!".to_string());
    //let text_state6 = CommonState::new_local_with_key(&"Hola mundo!".to_string());

    let selected_index = CommonState::new_local_with_key(&0).into_box();

    let selected_model = CommonState::new_local_with_key(&vec![
        Monday,
        Tuesday,
        Wednesday,
        Thursday,
        Friday,
        Saturday,
        Sunday,
    ]).into_box();

    window.set_widgets(
        VStack::new(vec![
            PlainTextInput::new(text_state)
                .font_size(40)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(50.0)),
            PlainTextInput::new(text_state2)
                .font_size(40)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(50.0)),
            PlainTextInput::new(text_state3)
                .font_size(40)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(50.0)),
            PlainTextInput::new(text_state4)
                .font_size(40)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(50.0)),
            /*PlainTextInput::new(text_state5)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(30.0)),
            PlainTextInput::new(text_state6)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(30.0)),*/
            PopUpButton::new(selected_model, selected_index)
                .padding(EdgeInsets::all(50.0)),
        ]).spacing(20.0)
    );

    window.run_event_loop();
}