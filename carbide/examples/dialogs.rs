use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use carbide::animation::Animation;
use carbide::color::{BLUE, GREEN, RED};
use carbide::dialog::color_dialog::ColorDialog;
use carbide::dialog::emoji_dialog::EmojiDialog;
use carbide::dialog::FileSpecification;
use carbide::dialog::open_dialog::OpenDialog;
use carbide::dialog::save_dialog::SaveDialog;
use carbide::platform::mac::{open_open_panel, open_save_panel};
use carbide::prelude::elastic_in_out;
use carbide::SpawnTask;
use carbide::state::{bounce_in, bounce_in_out, bounce_out, ease_in_out, linear, ValueState};
use carbide_controls::{Button, List, TextInput};
use carbide_controls::capture;
use carbide_core::animate;
use carbide_core::animation::Animatable;
use carbide_core::Color;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::environment::Environment;
use carbide_core::state::{LocalState, State, StringState, TState, UsizeState};
use carbide_core::task;
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::TryFutureExt;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Dialogs - Carbide",
        600,
        400,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_bitmap_font_with_hints(
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let text_state = LocalState::new("Hello world!".to_string());
    let color = LocalState::new(GREEN);
    let color_dialog = color.clone();

    window.set_widgets(
        VStack::new(vec![
            Button::new("Open Save-dialog")
                .on_click(|env: &mut Environment| {
                    SaveDialog::new().open(env)
                        .spawn(env, |res: Option<PathBuf>, _| {
                            println!("Received save path: {:?}", res);
                        });
                })
                .frame(200.0, 22.0),
            Button::new("Open Open-dialog")
                .on_click(|env: &mut Environment| {
                    OpenDialog::new()
                        .message("Hejsa, det er en besked".to_string())
                        .default_type(FileSpecification::new("Gif", &["gif"]))
                        .button_text("Åben".to_string())
                        .open(env)
                        .spawn(env, |res: Option<Vec<PathBuf>>, _| {
                            println!("Received open path: {:?}", res);
                        });
                })
                .frame(200.0, 22.0),
            Button::new("Open Emoji-dialog")
                .on_click(|env: &mut Environment| {
                    EmojiDialog::new()
                        .open()
                })
                .frame(200.0, 22.0),
            TextInput::new(text_state)
                .frame(200.0, 22.0),
            Button::new("Open Color-dialog")
                .on_click(move |env: &mut Environment| {
                    let color = color_dialog.clone();
                    ColorDialog::new()
                        .show_alpha()
                        .open(env, move |col, env| {
                            let mut color = color.clone();
                            color.set_value(col);
                            //println!("Color: {:?}", col);
                            false
                        })
                })
                .accent_color(color)
                .frame(200.0, 22.0),
        ]).spacing(10.0)
    );

    window.launch();
}