use carbide_controls::{ControlsExt, EnabledState};
use carbide_controls::button::{BorderedProminentStyle, BorderedStyle, Button, PlainProminentStyle, PlainStyle};
use carbide_core::accessibility::AccessibilityExt;
use carbide_core::closure;
use carbide_core::draw::{AutomaticStyle, Dimension};
use carbide_core::event::{Key, ModifierKey};
use carbide_core::state::{LocalState, ReadStateExtNew, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter_state = LocalState::new(0);

    application.set_scene(Window::new(
        "Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Text::new(counter_state.map(|count: &i32| format!("Count: {}", count)))
                .font_size(32u32),

            Button::new("Add 1", closure!(|_| { *$counter_state += 1; }))
                .keyboard_shortcut("r", ModifierKey::ALT)
                .button_style(BorderedProminentStyle)
                .frame(90.0, 22.0),

            Button::new("Subtract 1", closure!(|_| { *$counter_state -= 1; }))
                .frame(90.0, 22.0),

            Button::new("Disabled", closure!(|_|{}))
                .enabled(false)
                .frame(90.0, 22.0),

            HStack::new((
                Button::new(
                    Image::new_icon("icons/chat-1-line.png")
                        .accessibility_label("Add 1"),
                    closure!(|_| {})
                )
                    .button_style(BorderedProminentStyle)
                    .frame(32.0, 32.0),
                Button::new(
                    Image::new_icon("icons/chat-1-line.png")
                        .accessibility_label("Subtract 1"),
                    closure!(|_| {})
                )
                    .frame(32.0, 32.0),
                Button::new(
                    Image::new_icon("icons/chat-1-line.png")
                        .accessibility_label("Disabled"),
                    closure!(|_| {})
                )
                    .enabled(false)
                    .frame(32.0, 32.0),
            )).spacing(10.0)
        ))
            .spacing(20.0)
            //.button_style(BorderedStyle)
            //.button_style(BorderedProminentStyle)
            //.button_style(PlainStyle)
            //.button_style(PlainProminentStyle)
            .button_style(AutomaticStyle)
    ));

    application.launch();
}
