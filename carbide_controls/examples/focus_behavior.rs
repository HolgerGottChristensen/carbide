use carbide_controls::{PlainDatePicker, PlainTextInput};
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use crate::Month::{
    April, December, February, January, July, June, March, May, November, October, September,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Default for Month {
    fn default() -> Self {
        January
    }
}

fn main() {
    let text_state = LocalState::new("Hello World!".to_string());
    let text_state2 = LocalState::new("Hej Verden!".to_string());
    let text_state3 = LocalState::new("Hallo Welt!".to_string());
    let text_state4 = LocalState::new("Ciao mondo!".to_string());
    //let text_state5 = LocalState::new("Bonjour monde!".to_string());
    //let text_state6 = LocalState::new("Hola mundo!".to_string());

    let selected = LocalState::new(January);

    let model = LocalState::new(vec![
        January, February, March, April, May, June, July, September, October, November, December,
    ]);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Focus behavior example - Carbide",
        Dimension::new(400.0, 600.0),
        *VStack::new(vec![
            PlainTextInput::new(text_state)
                .font_size(40u32)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(20.0))
                .boxed(),
            PlainTextInput::new(text_state2)
                .font_size(40u32)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(20.0))
                .boxed(),
            PlainTextInput::new(text_state3)
                .font_size(40u32)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(20.0))
                .boxed(),
            PlainTextInput::new(text_state4)
                .font_size(40u32)
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .padding(EdgeInsets::all(20.0))
                .boxed(),
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
            //PopUpButton::new(model, selected).padding(EdgeInsets::all(50.0)),
            PlainDatePicker::new(selected, model)
                .padding(50.0)
                .boxed(),
        ])
            .spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
