use carbide_macro::CarbideUI;
use carbide::cursor::MouseCursor;
use carbide::state::{State, TState, UsizeState};
use carbide_controls::Button;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, ReadState, StateExt};
use carbide_core::text::{FontFamily, FontWeight, TextDecoration};
use carbide_core::widget::*;
use carbide_core::Widget;
use carbide_wgpu::{Application, Window};
use carbide_core::draw::Position;

fn main() {
    CarbideUI!{
        struct AlbumDetail {

            fn body() -> Widget {
                VStack {
                    HStack {
                        Text("Hejsa", weight: FontWeight::Bold)
                        Text("Verden", decoration: TextDecoration::Underline(vec![]))
                    }
                    Rectangle
                        .fill(EnvironmentColor::Red)
                        .frame(100.0, 100.0)
                }
            }
        }
    }

    let mut application = Application::new();

    let family =
        FontFamily::new_from_paths("NotoSans", vec![
            "fonts/NotoSans/NotoSans-Regular.ttf",
            "fonts/NotoSans/NotoSans-Bold.ttf",
        ]);
    application.add_font_family(family);

    application.set_scene(Window::new(
        "Carbide syntax example".to_string(),
        Dimension::new(400.0, 600.0),
        AlbumDetail::new()
    ).close_application_on_window_close());

    application.launch();

}