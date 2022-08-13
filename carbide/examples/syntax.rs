use carbide_macro::{CarbideUI, gen_optionals};
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
            //let articles: Vec<String>
            let alignment: u32 = 53

            fn body() -> Widget {
                if alignment == 42 {
                    Text("Its nice")
                } else if alignment == 53 {
                    Text("Its default")
                } else {
                    Text("Not as nice")
                }

                /*if alignment == 42 {
                    Text("Its nice")
                } else if alignment == 53 {
                    Text("Its default")
                } else {
                    Text("Not as nice")
                }*/

                /*VStack {
                    HStack(spacing: 20.0) {
                        Text("Hejsa").bold()
                        Text("Verden").underline()
                        Text(alignment)
                    }
                    ZStack {
                        Rectangle.fill(EnvironmentColor::Red)
                        Circle.fill(EnvironmentColor::Green)
                            .frame(50.0, 50.0)
                    }.frame(100.0, 100.0)
                }*/

                /*HStack (articles: arts, aaa, optionally: 2 + 23) {

                }
                VStack {
                    VStack { hejsa in

                    }
                    VStack {

                    }
                }
                List(articles) { article in
                    HStack {
                        Image(article.front_image)
                        VStack(alignment: Alignment::Leading) {
                            Text(song.title)
                            Text(song.artist.name)
                                .foregroundStyle(Alignment::Leading)
                                .foregroundStyle(Alignment::Leading)
                        }
                    }
                }*/
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
        AlbumDetail::builder()
            .with_optional_alignment(42u32)
            .finish()
    ).close_application_on_window_close());

    application.launch();

}