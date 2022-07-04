mod message_bubble;

use crate::message_bubble::{Message, MessageBubble};
use carbide_controls::{capture, Button, List, TextInput};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::prelude::{State, TState};
use carbide_core::state::{LocalState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::{Circle, Ellipse, HStack, Text, VStack, Widget, WidgetExt, ZStack};
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    let mut window = Window::new("Messaging Service", 400, 300, None);

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    fn list_delegate(item: TState<Message>, index: UsizeState) -> Box<dyn Widget> {
        MessageBubble::new(item, "Mark".to_string())
    }

    let model = LocalState::new(vec![
        Message::new("Hello Carl".to_string(), "Mark".to_string()),
        Message::new("Hi Mark".to_string(), "Carl".to_string()),
        Message::new(
            "Carl you made an error in the file".to_string(),
            "Mark".to_string(),
        ),
    ]);

    let mut list = List::new(model.clone(), list_delegate).spacing(20.0);

    let text_state = LocalState::new("Write Message".to_string());

    let mut textfield_container = HStack::new(vec![
        TextInput::new(text_state.clone()),
        Button::new("Send")
            .on_click(capture!([text_state, model], |env: &mut Environment| {
                println!("{}", text_state);
                model.push(Message::new(text_state.to_string(), "Mark".to_string()));
                *text_state = "".to_string()
            }))
            .frame(70, 22),
    ]);

    let mut root = VStack::new(vec![list, textfield_container]);

    window.set_widgets(root);
    window.launch();
}
